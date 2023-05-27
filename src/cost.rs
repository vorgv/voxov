use crate::config::{self, Config};
use crate::database::Database;
use crate::fed::Fed;
use crate::message::{Error, Id, Query, Reply, Uint};
use tokio_util::sync::CancellationToken;

pub struct Cost {
    fed: &'static Fed,
    db: &'static Database,
    time_cost: Uint,
}

impl Cost {
    pub fn new(config: &Config, db: &'static Database, fed: &'static Fed) -> Cost {
        Cost {
            fed,
            db,
            time_cost: config.time_cost,
        }
    }
    pub async fn handle(&self, query: &Query, uid: &Id) -> Reply {
        match query {
            Query::Pay { access: _, vendor } => Reply::Pay {
                uri: format!("Not implemented: {}, {}", vendor, uid),
            },
            _ => {
                let costs = query.get_costs();
                let token = CancellationToken::new();
                let cloned_token = token.clone();
                let deadline = costs.time * self.time_cost;
                tokio::spawn(async move {
                    tokio::time::sleep(std::time::Duration::from_millis(deadline)).await;
                    token.cancel();
                });
                tokio::select! {
                    r = async {
                        self.fed.handle(query, uid, costs, cloned_token.clone()).await
                    } => {r}
                    _ = cloned_token.cancelled() => Reply::Error { error: Error::Cost }
                }
            }
        }
    }
}
