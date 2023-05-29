use crate::config::Config;
use crate::database::namespace::UID2CREDIT;
use crate::database::{ns, Database};
use crate::error::Error;
use crate::fed::Fed;
use crate::message::{Id, Int, Query, Reply, Uint};
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
            Query::CostPay { access: _, vendor } => Reply::CostPay {
                uri: format!("Not implemented: {}, {}", vendor, uid),
            },
            _ => {
                // Check if cost exceeds credit.
                let costs = query.get_costs();
                let u2p = ns(UID2CREDIT, uid);
                let credit = match self.db.get::<&[u8], Int>(&u2p).await {
                    Ok(i) => i,
                    Err(error) => return Reply::Error { error },
                };
                if costs.sum() as Int > credit {
                    return Reply::Error {
                        error: Error::CostInsufficientCredit,
                    };
                }

                // Set time limit as connection costs.
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
                    _ = cloned_token.cancelled() => Reply::Error { error: Error::CostTimeout }
                }
            }
        }
    }
}
