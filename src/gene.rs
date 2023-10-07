//! Genes are just functions.

use crate::config::Config;
use crate::database::Database;
use crate::ir::{Costs, Id, Query, Reply};
use crate::meme::Meme;
use crate::{cost_macros, Error, Result};
use mongodb::bson::doc;
use serde::Serialize;
use std::collections::HashMap;
use tokio::time::{Duration, Instant};

mod info;
mod map;
mod msg;

pub struct Gene {
    meme: &'static Meme,
    config_json: String,
    db: &'static Database,
    metas: &'static HashMap<String, GeneMeta>,
    time_cost: i64,
    space_cost_doc: i64,
    traffic_cost: i64,
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

    #[allow(unused_macros)]
    pub async fn handle(
        &self,
        query: Query,
        uid: &Id,
        mut changes: Costs,
        deadline: Instant,
    ) -> Result<Reply> {
        cost_macros!(self, uid, changes, deadline);

        let reply = self
            .handle_ignore_error(query, uid, changes, deadline)
            .await;
        if reply.is_err() {
            time!();
            refund!();
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
        cost_macros!(self, uid, changes, deadline);

        match query {
            Query::GeneMeta { head: _, gid } => {
                let meta =
                    serde_json::to_string(&self.metas.get(&gid).ok_or(Error::GeneInvalidId)?)
                        .unwrap();
                traffic_time_refund!(meta);
                Ok(Reply::GeneMeta { changes, meta })
            }

            Query::GeneCall { head: _, gid, arg } => {
                macro_rules! map_1_cx {
                    () => {
                        map::V1Context {
                            uid,
                            arg: &arg,
                            changes: &mut changes,
                            deadline,
                            space_cost: self.space_cost_doc,
                            traffic_cost: self.traffic_cost,
                            db: self.db,
                        }
                    };
                }

                let result = match gid.as_str() {
                    "info_v1" => info::v1(uid, &arg, self.config_json.clone()).await,
                    "map_v1" => map::v1(map_1_cx!(), false).await?,
                    "msg_v1" => msg::v1(map_1_cx!()).await?,
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
    pub fn new_map() -> HashMap<String, GeneMeta> {
        HashMap::from([
            (
                "info_1".into(),
                GeneMeta {
                    name: "info".into(),
                    version: 1,
                    description: "Infomantion about this server.".into(),
                },
            ),
            (
                "map_1".into(),
                GeneMeta {
                    name: "map".into(),
                    version: 1,
                    description: "Mapping abstraction backed by MongoDB.".into(),
                },
            ),
            (
                "msg_1".into(),
                GeneMeta {
                    name: "msg".into(),
                    version: 1,
                    description: "Messaging another user.".into(),
                },
            ),
        ])
    }
}
