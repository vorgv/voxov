//! Genes are just functions.

use mongodb::bson::doc;
use mongodb::options::FindOneOptions;
use serde::Serialize;
use tokio::time::{Duration, Instant};

use crate::config::Config;
use crate::database::namespace::UID2CREDIT;
use crate::database::{ns, Database};
use crate::error::Error;
use crate::meme::{Meme, Putter};
use crate::message::{Costs, Id, Query, Reply, Uint};

mod info;
mod map;

pub struct Gene {
    meme: &'static Meme,
    c: &'static Config,
    db: &'static Database,
    metas: &'static Vec<GeneMeta>,
    time_cost: Uint,
    space_cost_doc: Uint,
    space_cost_obj: Uint,
    traffic_cost: Uint,
}

impl Gene {
    pub fn new(c: &'static Config, db: &'static Database, meme: &'static Meme) -> Gene {
        Gene {
            meme,
            c,
            db,
            metas: c.gene_metas,
            time_cost: c.time_cost,
            space_cost_doc: c.space_cost_doc,
            space_cost_obj: c.space_cost_obj,
            traffic_cost: c.traffic_cost,
        }
    }

    pub async fn handle(
        &self,
        query: Query,
        uid: &Id,
        mut changes: Costs,
        deadline: Instant,
    ) -> Result<Reply, Error> {
        // The same as the two combined in self.handle_ignore_error().
        macro_rules! time_refund {
            () => {
                let now = Instant::now();
                if now > deadline {
                    return Err(Error::CostTime);
                } else {
                    let remaining: Duration = deadline - now;
                    changes.time = remaining.as_millis() as Uint * self.time_cost;
                }
                let u2c = ns(UID2CREDIT, uid);
                self.db.incrby(&u2c[..], changes.sum()).await?;
            };
        }

        let reply = self
            .handle_ignore_error(query, uid, changes, deadline)
            .await;
        if reply.is_err() {
            time_refund!();
        }
        reply
    }

    /// Refund Ok(_)s, and leave Err(_)s to be refunded in the upstream.
    async fn handle_ignore_error(
        &self,
        query: Query,
        uid: &Id,
        mut changes: Costs,
        deadline: Instant,
    ) -> Result<Reply, Error> {
        /// Subtract traffic from changes based on $s.len().
        macro_rules! traffic {
            ($s: expr) => {
                // Traffic cost is server-to-client for now.
                let traffic = $s.len() as Uint * self.traffic_cost;
                if traffic > changes.traffic {
                    return Err(Error::CostTraffic);
                } else {
                    changes.traffic -= traffic;
                }
            };
        }

        /// Update changes.time by the closeness to deadline.
        macro_rules! time {
            () => {
                let now = Instant::now();
                if now > deadline {
                    return Err(Error::CostTime);
                } else {
                    let remaining: Duration = deadline - now;
                    changes.time = remaining.as_millis() as Uint * self.time_cost;
                }
            };
        }

        /// Refund current changes.
        macro_rules! refund {
            () => {
                let u2c = ns(UID2CREDIT, uid);
                self.db.incrby(&u2c[..], changes.sum()).await?;
            };
        }

        /// Three in one.
        macro_rules! traffic_time_refund {
            ($s: expr) => {
                traffic!($s);
                time!();
                refund!();
            };
        }

        match query {
            Query::GeneMeta { head: _, id } => {
                if id >= self.metas.len() {
                    return Err(Error::GeneInvalidId);
                }
                let meta = serde_json::to_string(&self.metas[id]).unwrap();
                traffic_time_refund!(meta);
                Ok(Reply::GeneMeta { changes, meta })
            }

            Query::GeneCall { head: _, id, arg } => {
                traffic!(arg);
                let result = match id {
                    0 => info::v1(uid, &arg, self.c).await,
                    1 => map::v1(uid, &arg, &mut changes, self.space_cost_doc, deadline).await,
                    _ => {
                        return Err(Error::GeneInvalidId);
                    }
                };
                traffic_time_refund!(result);
                Ok(Reply::GeneCall { changes, result })
            }

            Query::MemeMeta { head: _, hash } => {
                let meta = self.meme.get_meta(uid, &hash, deadline).await?;
                traffic_time_refund!(meta);
                Ok(Reply::MemeMeta { changes, meta })
            }

            Query::MemeRawPut { head: _, days, raw } => {
                // keep at least 1 day.
                if days < 1 {
                    return Err(Error::MemeRawPut);
                }
                // Check if fund is enough for the first poll.
                const MAX_FRAME_BYTES: usize = 16_777_215;
                if changes.traffic < MAX_FRAME_BYTES as u64 * self.space_cost_obj {
                    return Err(Error::CostTraffic);
                }
                // AsyncRead from Incoming
                let mut putter = Putter::new(days, raw, changes, deadline, self.space_cost_obj);
                self.meme.put_meme(uid, changes, days, &mut putter).await?;
                // Refund
                let hash: [u8; 32] = putter.get_hash().into();
                let changes = putter.into_changes();
                traffic_time_refund!(hash);
                Ok(Reply::MemeRawPut { changes, hash })
            }

            Query::MemeRawGet {
                head: _,
                hash,
                public,
            } => {
                let hash = hex::encode(hash);
                // Filter
                let filter = match public {
                    true => doc! {
                        "public": true,
                        "hash": hash.clone(),
                    },
                    false => doc! {
                        "uid": uid.to_string(),
                        "hash": hash.clone(),
                    },
                };
                // Sort by tips
                let options = FindOneOptions::builder()
                    .projection(doc! { "uid": 1, "hash": 1, "size": 1, "tips": 1, "_id": 0 })
                    .sort(doc! { "tips": 1 })
                    .build();
                let mm = &self.db.mm;
                let meta = mm
                    .find_one(filter, options)
                    .await
                    .map_err(|_| Error::MemeRawGet)?;
                if meta.is_none() {
                    return Err(Error::MemeNotFound);
                }
                let meta = meta.unwrap();
                // Is fund enough for the file size
                let cost =
                    self.space_cost_obj * meta.get_i64("size").map_err(|_| Error::Logical)? as u64;
                if cost > changes.space {
                    return Err(Error::CostSpace);
                }
                changes.space -= cost;
                // Pay tips
                if public {
                    let tips = meta.get_i64("tips").map_err(|_| Error::Logical)? as u64;
                    if tips > changes.tips {
                        return Err(Error::CostTips);
                    }
                    changes.tips -= tips;
                    let uid = meta.get_str("uid").map_err(|_| Error::Logical)?;
                    use std::str::FromStr;
                    let uid = Id::from_str(uid)?;
                    let u2c = ns(UID2CREDIT, &uid);
                    self.db.incrby(&u2c[..], tips).await?;
                }
                // Stream object
                let mr = &self.db.mr;
                let stream = Box::pin(mr.get_object_stream(hash).await.map_err(|_| Error::S3)?);
                // Check costs
                Ok(Reply::MemeRawGet {
                    changes,
                    raw: stream,
                })
            }

            _ => Err(Error::Logical), // This arm should be unreachable.
        }
    }
}

#[derive(Serialize)]
pub struct GeneMeta {
    /// Naming convention: snake_case
    name: String,

    /// Increment on breaking changes.
    version: usize,

    /// Man page.
    description: String,
}

impl GeneMeta {
    pub fn new_vec() -> Vec<GeneMeta> {
        vec![
            // 0
            GeneMeta {
                name: "info".into(),
                version: 1,
                description: "Return infomantion about this server.".into(),
            },
            // 1
            GeneMeta {
                name: "map".into(),
                version: 1,
                description: "Mapping over document data backed by MongoDB.".into(),
            },
        ]
    }
}
