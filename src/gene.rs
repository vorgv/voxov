//! Genes are just functions.

use serde::Serialize;
use tokio::time::{Duration, Instant};

use crate::config::Config;
use crate::database::namespace::UID2CREDIT;
use crate::database::{ns, Database};
use crate::error::Error;
use crate::meme::Meme;
use crate::message::{Costs, Id, Query, Reply, Uint};

mod file;
mod info;

pub struct Gene {
    meme: &'static Meme,
    db: &'static Database,
    metas: &'static Vec<GeneMeta>,
    time_cost: Uint,
    traffic_cost: Uint,
}

impl Gene {
    pub fn new(config: &Config, db: &'static Database, meme: &'static Meme) -> Gene {
        Gene {
            meme,
            db,
            metas: config.gene_metas,
            time_cost: config.time_cost,
            traffic_cost: config.traffic_cost,
        }
    }

    pub async fn handle(
        &self,
        query: &Query,
        uid: &Id,
        mut change: Costs,
        deadline: Instant,
    ) -> Result<Reply, Error> {
        /// Subtract traffic from change based on $s.len().
        macro_rules! traffic {
            ($s: expr) => {
                // Traffic cost is server-to-client for now.
                let traffic = $s.len() as Uint * self.traffic_cost;
                if traffic > change.traffic {
                    return Err(Error::CostTraffic);
                } else {
                    change.traffic -= traffic;
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
                    change.time = remaining.as_millis() as Uint * self.time_cost;
                }
            };
        }

        /// Refund current change.
        macro_rules! refund {
            () => {
                let u2c = ns(UID2CREDIT, uid);
                self.db.incrby(&u2c[..], change.sum()).await?;
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
                if id >= &self.metas.len() {
                    return Err(Error::GeneInvalidId);
                }
                let meta = serde_json::to_string(&self.metas[*id]).unwrap();
                traffic_time_refund!(meta);
                Ok(Reply::GeneMeta { change, meta })
            }

            Query::GeneCall { head, id, arg } => {
                let result = match id {
                    0 => info::v1().await,
                    1 => file::v1(head, arg).await,
                    _ => return Err(Error::GeneInvalidId),
                };
                traffic_time_refund!(result);
                Ok(Reply::GeneCall { change, result })
            }

            Query::MemeMeta { head: _, hash } => {
                let meta = self.meme.get_meta(uid, hash, deadline).await?;
                traffic_time_refund!(meta);
                Ok(Reply::MemeMeta { change, meta })
            }

            Query::MemeRawPut {
                head: _,
                hash: _,
                raw: _,
            } => Ok(Reply::Unimplemented),

            Query::MemeRawGet { head: _, hash: _ } => Ok(Reply::Unimplemented),

            _ => Err(Error::Logical), // This arm should be unreachable.
        }
    }
}

#[derive(Serialize)]
pub struct GeneMeta {
    name: String,
    /// Increment on breaking changes.
    version: usize,
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
                name: "file".into(),
                version: 1,
                description: "User file system.".into(),
            },
        ]
    }
}
