use serde::Serialize;
use serde_json;
use tokio_util::sync::CancellationToken;

use crate::config::Config;
use crate::database::namespace::UID2CREDIT;
use crate::database::{ns, Database};
use crate::error::Error;
use crate::meme::Meme;
use crate::message::{Costs, Id, Query, Reply, Uint};

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
        costs: &Costs,
        token: CancellationToken,
    ) -> Reply {
        macro_rules! check {
            ($id:expr) => {
                if $id >= &self.metas.len() {
                    return Reply::Error {
                        error: Error::GeneInvalidId,
                    };
                }
            };
        }
        match query {
            Query::GeneMeta { head, id } => {
                check!(id);
                let meta = serde_json::to_string(&self.metas[*id]).unwrap();
                let costs = &head.costs;
                let traffic = meta.len() as Uint * self.traffic_cost;
                if traffic > costs.traffic {
                    return Reply::Error {
                        error: Error::CostTraffic,
                    };
                } else {
                    let u2c = ns(UID2CREDIT, uid);
                    if let Err(error) = self.db.decrby(&u2c[..], traffic).await {
                        return Reply::Error { error };
                    }
                }
                Reply::GeneMeta {
                    cost: Costs {
                        time: 0,
                        space: 0,
                        traffic,
                        tips: 0,
                    },
                    meta: Ok(meta),
                }
            }
            Query::GeneCall { head, id, arg } => {
                check!(id);
                Reply::Unimplemented
            }
            Query::MemeMeta { head, key } => Reply::Unimplemented,
            Query::MemeRawPut { head, key, raw } => Reply::Unimplemented,
            Query::MemeRawGet { head, key } => Reply::Unimplemented,
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
}

impl GeneMeta {
    pub fn new_vec() -> Vec<GeneMeta> {
        vec![
            GeneMeta {
                name: "info".to_string(),
                version: 1,
            },
            GeneMeta {
                name: "file".to_string(),
                version: 1,
            },
        ]
    }
}
