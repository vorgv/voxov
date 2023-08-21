//! Genes are just functions.

use crate::config::Config;
use crate::database::namespace::UID2CREDIT;
use crate::database::{ns, Database};
use crate::error::Error;
use crate::meme::Meme;
use crate::message::{Costs, Id, Query, Reply, Uint};
use crate::Result;
use mongodb::bson::doc;
use serde::Serialize;
use tokio::time::{Duration, Instant};

mod chan;
mod info;
mod map;

pub struct Gene {
    meme: &'static Meme,
    config_json: String,
    db: &'static Database,
    metas: &'static Vec<GeneMeta>,
    time_cost: Uint,
    space_cost_doc: Uint,
    traffic_cost: Uint,
}

impl Gene {
    pub fn new(c: &'static Config, db: &'static Database, meme: &'static Meme) -> Gene {
        Gene {
            meme,
            config_json: serde_json::to_string_pretty(c).unwrap_or_default(),
            db,
            metas: c.gene_metas,
            time_cost: c.time_cost,
            space_cost_doc: c.space_cost_doc,
            traffic_cost: c.traffic_cost,
        }
    }

    pub async fn handle(
        &self,
        query: Query,
        uid: &Id,
        mut changes: Costs,
        deadline: Instant,
    ) -> Result<Reply> {
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
    ) -> Result<Reply> {
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
                    changes.time = 0;
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
            Query::GeneMeta { head: _, gid } => {
                if gid >= self.metas.len() {
                    return Err(Error::GeneInvalidId);
                }
                let meta = serde_json::to_string(&self.metas[gid]).unwrap();
                traffic_time_refund!(meta);
                Ok(Reply::GeneMeta { changes, meta })
            }

            Query::GeneCall { head: _, gid, arg } => {
                traffic!(arg);
                let result = match gid {
                    0 => info::v1(uid, &arg, self.config_json.clone()).await,
                    1 => {
                        map::v1(
                            uid,
                            &arg,
                            &mut changes,
                            deadline,
                            self.space_cost_doc,
                            self.traffic_cost,
                            self.db,
                        )
                        .await?
                    }
                    2 => chan::v1().await?,
                    _ => {
                        return Err(Error::GeneInvalidId);
                    }
                };
                traffic_time_refund!(result);
                Ok(Reply::GeneCall { changes, result })
            }

            Query::MemeMeta { head: _, hash } => {
                let meta = self.meme.get_meta(uid, deadline, &hash).await?;
                traffic_time_refund!(meta);
                Ok(Reply::MemeMeta { changes, meta })
            }

            Query::MemePut { head: _, days, raw } => {
                let reply = self
                    .meme
                    .put_meme(uid, &mut changes, deadline, days, raw)
                    .await;
                refund!();
                reply
            }

            Query::MemeGet {
                head: _,
                hash,
                public,
            } => {
                let reply = self
                    .meme
                    .get_meme(uid, &mut changes, deadline, hash, public)
                    .await;
                refund!();
                reply
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
                description: "Infomantion about this server.".into(),
            },
            // 1
            GeneMeta {
                name: "map".into(),
                version: 1,
                description: "Mapping abstraction backed by MongoDB.".into(),
            },
            // 2
            GeneMeta {
                name: "chan".into(),
                version: 1,
                description: "Messaging another user".into(),
            },
        ]
    }
}
