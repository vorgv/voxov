//! The cost layer checks balance and does cancellation on timeout.
//! Payment is also handled here. Anything behind this is paid.

use crate::config::Config;
use crate::database::namespace::{UID2CHECKIN, UID2CREDIT};
use crate::database::{ns, Database};
use crate::error::Error;
use crate::fed::Fed;
use crate::ir::{Id, Query, Reply};
use crate::Result;
use tokio::time::{Duration, Instant};

pub struct Cost {
    fed: &'static Fed,
    db: &'static Database,
    credit_limit: i64,
    time_cost: i64,
    check_in_award: i64,
    check_in_refresh: i64,
}

impl Cost {
    pub fn new(config: &Config, db: &'static Database, fed: &'static Fed) -> Cost {
        Cost {
            fed,
            db,
            credit_limit: config.credit_limit,
            time_cost: config.time_cost,
            check_in_award: config.check_in_award,
            check_in_refresh: config.check_in_refresh,
        }
    }

    pub async fn handle(&self, query: Query, uid: &Id) -> Result<Reply> {
        match query {
            Query::CostPay { access: _, vendor } => Ok(Reply::CostPay {
                uri: format!("Not implemented: {}, {}", vendor, uid),
            }),

            Query::CostGet { access: _ } => {
                let u2p = ns(UID2CREDIT, uid);
                let credit = self.db.get::<&[u8], i64>(&u2p).await?;
                Ok(Reply::CostGet { credit })
            }

            Query::CostCheckIn { access: _ } => {
                // checked in today?
                let u2ci = ns(UID2CHECKIN, uid);
                let last_check_in = self.db.incr(&u2ci[..]).await?;
                self.db.expire_xx(&u2ci[..], self.check_in_refresh).await?;
                match last_check_in {
                    1 => {
                        let u2c = ns(UID2CREDIT, uid);
                        self.db.incrby(&u2c[..], self.check_in_award).await?;
                        Ok(Reply::CostCheckIn {
                            award: self.check_in_award,
                        })
                    }
                    _ => Err(Error::CostCheckInTooEarly),
                }
            }

            _ => {
                // Check if cost exceeds credit.
                let costs = query.get_costs();
                let u2p = ns(UID2CREDIT, uid);
                let credit = self.db.get::<&[u8], i64>(&u2p).await?;
                if costs.sum() > credit - self.credit_limit {
                    return Err(Error::CostInsufficientCredit);
                } else {
                    // Decrement then refund to prevent double pay.
                    let u2c = ns(UID2CREDIT, uid);
                    self.db.decrby(&u2c[..], costs.sum()).await?;
                }

                // Set limits.
                let deadline = Instant::now()
                    + Duration::from_millis((costs.time / self.time_cost).try_into()?);
                self.fed.handle(query, uid, costs, deadline).await
            }
        }
    }
}
