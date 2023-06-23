//! Genes are just functions.

use std::task::Poll;

use http_body_util::BodyExt;
use serde::Serialize;
use tokio::io::{AsyncRead, BufStream};
use tokio::time::{Duration, Instant};
use blake3;

use crate::config::Config;
use crate::database::namespace::UID2CREDIT;
use crate::database::{ns, Database};
use crate::error::Error;
use crate::meme::Meme;
use crate::message::{Costs, Id, Query, Reply, Uint};

mod info;
mod map;

pub struct Gene {
    meme: &'static Meme,
    db: &'static Database,
    metas: &'static Vec<GeneMeta>,
    time_cost: Uint,
    space_cost_doc: Uint,
    space_cost_obj: Uint,
    traffic_cost: Uint,
}

impl Gene {
    pub fn new(config: &Config, db: &'static Database, meme: &'static Meme) -> Gene {
        Gene {
            meme,
            db,
            metas: config.gene_metas,
            time_cost: config.time_cost,
            space_cost_doc: config.space_cost_doc,
            space_cost_obj: config.space_cost_obj,
            traffic_cost: config.traffic_cost,
        }
    }

    pub async fn handle(
        &self,
        query: Query,
        uid: &Id,
        mut changes: Costs,
        deadline: Instant,
    ) -> Result<Reply, Error> {
        /// Subtract traffic from change based on $s.len().
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

        /// Update change.time by the closeness to deadline.
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

        /// Refund current change.
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
                Ok(Reply::GeneMeta { change: changes, meta })
            }

            Query::GeneCall { head: _, id, arg } => {
                traffic!(arg);
                let result = match id {
                    0 => info::v1(uid, &arg).await,
                    1 => map::v1(uid, &arg, &mut changes, self.space_cost_doc, deadline).await,
                    _ => return Err(Error::GeneInvalidId),
                };
                traffic_time_refund!(result);
                Ok(Reply::GeneCall { change: changes, result })
            }

            Query::MemeMeta { head: _, hash } => {
                let meta = self.meme.get_meta(uid, &hash, deadline).await?;
                traffic_time_refund!(meta);
                Ok(Reply::MemeMeta { change: changes, meta })
            }

            Query::MemeRawPut { head: _, mut raw } => {
                // check if fund is enough for the first round
                const MAX_FRAME_BYTES: usize = 16_777_215;
                if changes.traffic < MAX_FRAME_BYTES as u64 * self.space_cost_obj {
                    return Err(Error::CostTraffic)
                }
                // create tmp object with a random name
                let mut putter = Putter {};
                let mut rng = rand::thread_rng();
                let tmp_id = Id::rand(&mut rng)?;
                self.db.mr.put_object_stream(&mut putter, tmp_id.to_string());
                // create hash
                let mut hasher = blake3::Hasher::new();
                while let Some(result) = raw.frame().await {
                    match result {
                        Ok(frame) => {
                            if frame.is_data() {
                                let bytes = frame.into_data().map_err(|_| Error::MemeRawPut)?;
                                // update hash
                                hasher.update(&bytes);
                                // append to object
                                // check costs
                            }
                        }
                        Err(_) => {
                            // remove tmp object
                            return Err(Error::MemeRawPut)
                        },
                    };
                }
                // if object does not exist, update object name by hash
                //  create meta-data
                // else append extra life to the object
                //  update meta-data
                Ok(Reply::Unimplemented)
            }

            Query::MemeRawGet { head: _, hash: _ } => {
                // check if fund is enough for the file size
                // get object handle
                loop {
                    // get from handle
                    // append to body
                    // check costs
                    break;
                }
                Ok(Reply::Unimplemented)
            }

            _ => Err(Error::Logical), // This arm should be unreachable.
        }
    }
}

struct Putter {}

impl AsyncRead for Putter {
    fn poll_read(
            self: std::pin::Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
            buf: &mut tokio::io::ReadBuf<'_>,
        ) -> std::task::Poll<std::io::Result<()>> {
        Poll::Pending
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
