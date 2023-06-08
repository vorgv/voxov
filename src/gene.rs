//! Genes are just functions.

use serde::Serialize;

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
    traffic_cost: Uint,
}

impl Gene {
    pub fn new(config: &Config, db: &'static Database, meme: &'static Meme) -> Gene {
        Gene {
            meme,
            db,
            metas: config.gene_metas,
            traffic_cost: config.traffic_cost,
        }
    }
    pub async fn handle(
        &self,
        query: &Query,
        uid: &Id,
        mut change: Costs,
        deadline: tokio::time::Instant,
    ) -> Reply {
        macro_rules! check_index {
            ($id:expr) => {
                if $id >= &self.metas.len() {
                    return Reply::Error {
                        error: Error::GeneInvalidId,
                    };
                }
            };
        }
        macro_rules! traffic {
            ($change: expr, $s: expr) => {
                // Traffic cost is server-to-client for now.
                let traffic = $s.len() as Uint * self.traffic_cost;
                if traffic > $change.traffic {
                    return Reply::Error {
                        error: Error::CostTraffic,
                    };
                } else {
                    $change.traffic -= traffic;
                    let u2c = ns(UID2CREDIT, uid);
                    if let Err(error) = self.db.decrby(&u2c[..], traffic).await {
                        return Reply::Error { error };
                    }
                }
            };
        }
        macro_rules! time {
            ($change: expr, $deadline: expr) => {};
        }
        match query {
            Query::GeneMeta { head: _, id } => {
                check_index!(id);
                let meta = serde_json::to_string(&self.metas[*id]).unwrap();
                traffic!(change, meta);
                time!(change, deadline);
                Reply::GeneMeta { change, meta }
            }
            Query::GeneCall { head, id, arg } => {
                check_index!(id);
                let result = match id {
                    0 => info::v1().await,
                    1 => file::v1(head, arg).await,
                    _ => {
                        return Reply::Error {
                            error: Error::Logical,
                        }
                    }
                };
                traffic!(change, result);
                time!(change, deadline);
                Reply::GeneCall { change, result }
            }
            Query::MemeMeta { head: _, key } => match self.meme.get_meta(uid, key).await {
                Ok(meta) => Reply::MemeMeta { change, meta },
                Err(error) => Reply::Error { error },
            },
            Query::MemeRawPut {
                head: _,
                key: _,
                raw: _,
            } => Reply::Unimplemented,
            Query::MemeRawGet { head: _, key: _ } => Reply::Unimplemented,
            _ => Reply::Error {
                error: crate::error::Error::Logical,
            },
        }
    }
}

#[derive(Serialize)]
pub struct GeneMeta {
    name: String,
    /// Incremen on breaking change.
    version: usize,
    description: String,
}

impl GeneMeta {
    //TODO: generate from macros.
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
