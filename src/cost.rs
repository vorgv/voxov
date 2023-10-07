//! The cost layer checks balance and does cancellation on timeout.
//! Payment is also handled here. Anything behind this is paid.

use crate::config::Config;
use crate::database::namespace::{UID2CHECKIN, UID2CREDIT};
use crate::database::{ns, Database};
use crate::fed::Fed;
use crate::ir::{Id, Query, Reply};
use crate::{Error, Result};
use tokio::time::{Duration, Instant};

pub struct Cost {
    fed: &'static Fed,
    db: &'static Database,
    time_cost: i64,
    check_in_award: i64,
    check_in_refresh: i64,
}

impl Cost {
    pub fn new(config: &Config, db: &'static Database, fed: &'static Fed) -> Cost {
        Cost {
            fed,
            db,
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
                        self.db
                            .incr_credit(uid, self.check_in_award, "CostCheckIn")
                            .await?;
                        Ok(Reply::CostCheckIn {
                            award: self.check_in_award,
                        })
                    }
                    _ => Err(Error::CostCheckInTooEarly),
                }
            }

            _ => {
                // Entry-refund to prevent double pay.
                let costs = query.get_costs();
                self.db.decr_credit(uid, costs.sum(), "CostEntry").await?;

                // Set limits.
                let deadline = Instant::now()
                    + Duration::from_millis((costs.time / self.time_cost).try_into()?);
                self.fed.handle(query, uid, costs, deadline).await
            }
        }
    }
}

pub mod macros {
    #[macro_export]
    macro_rules! cost_macros {
        ($self: expr, $uid: expr, $changes: expr, $deadline: expr) => {
            /// Subtract traffic from changes based on $s.len().
            macro_rules! traffic {
                ($s: expr) => {
                    // Traffic cost is server-to-client for now.
                    let traffic = $s.len() as i64 * $self.traffic_cost;
                    if traffic > $changes.traffic {
                        return Err(Error::CostTraffic);
                    } else {
                        $changes.traffic -= traffic;
                    }
                };
            }

            /// Update changes.time by the closeness to deadline.
            macro_rules! time {
                () => {
                    let now = Instant::now();
                    if now > $deadline {
                        $changes.time = 0;
                        return Err(Error::CostTime);
                    } else {
                        let remaining: Duration = $deadline - now;
                        $changes.time = remaining.as_millis() as i64 * $self.time_cost;
                    }
                };
            }

            /// Refund current changes.
            macro_rules! refund {
                () => {
                    $self
                        .db
                        .incr_credit($uid, $changes.sum(), "CostRefund")
                        .await?;
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
        };
    }
}
