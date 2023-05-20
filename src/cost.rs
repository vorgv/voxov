use crate::config::Config;
use crate::database::Database;
use crate::fed::Fed;
use crate::message::{Id, Query, Reply};
use tokio_util::sync::CancellationToken;

pub struct Cost {
    fed: &'static Fed,
    db: &'static Database,
}

impl Cost {
    pub fn new(_config: &Config, db: &'static Database, fed: &'static Fed) -> Cost {
        Cost { fed, db }
    }
    pub async fn handle(&self, uid: &Id, query: &Query) -> Reply {
        match query {
            Query::Pay { access: _, vendor } => Reply::Pay {
                uri: format!("Not implemented: {}, {}", vendor, uid),
            },
            _ => {
                let cost = query.get_cost();
                //TODO: timeout context
                let token = CancellationToken::new();
                tokio::select! {
                    r = async { self.fed.handle(uid, &cost.space, query).await } => {r}
                }
            }
        }
    }
}
