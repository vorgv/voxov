//! The cost layer checks balance and does cancellation on timeout.
//! Payment is also handled here. Anything behind this is paid.

use crate::config::Config;
use crate::database::namespace::UID2CREDIT;
use crate::database::{ns, Database};
use crate::error::Error;
use crate::fed::Fed;
use crate::message::{Id, Int, Query, Reply, Uint};
use tokio::time::{Duration, Instant};

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

    pub async fn handle(&self, query: Query, uid: &Id) -> Result<Reply, Error> {
        match query {
            Query::CostPay { access: _, vendor } => Ok(Reply::CostPay {
                uri: format!("Not implemented: {}, {}", vendor, uid),
            }),
            _ => {
                // Check if cost exceeds credit.
                let costs = query.get_costs();
                let u2p = ns(UID2CREDIT, uid);
                let credit = self.db.get::<&[u8], Int>(&u2p).await?;
                if costs.sum() as Int > credit {
                    return Err(Error::CostInsufficientCredit);
                } else {
                    // Decrement then refund to prevent double pay.
                    let u2c = ns(UID2CREDIT, uid);
                    self.db.decrby(&u2c[..], costs.sum()).await?;
                }

                // Set limits.
                let deadline = Instant::now() + Duration::from_millis(costs.time / self.time_cost);
                self.fed.handle(query, uid, costs, deadline).await
            }
        }
    }
}
