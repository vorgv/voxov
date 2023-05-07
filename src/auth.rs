use crate::config::Config;
use crate::cost::Cost;
use crate::database::Database;
use crate::message::{Error, Id, Query, Reply};

pub struct Auth {
    cost: Cost,
    db: &'static Database,
}

impl Auth {
    pub fn new(_config: &Config, db: &'static Database, cost: Cost) -> Auth {
        Auth { cost, db }
    }
    pub async fn handle(&self, query: &Query) -> Reply {
        match query {
            // Session management
            Query::AuthSessionStart => {
                let access = [1; 16];
                let refresh = [1; 16];
                let value = 2;
                let seconds = 3;
                if let Ok(()) = self.db.set(&access, value, seconds).await {
                    if let Ok(()) = self.db.set(&refresh, value, seconds).await {
                        return Reply::AuthSessionStart {
                            access: Id(access),
                            refresh: Id(refresh),
                        };
                    }
                }
                Reply::AuthError {
                    error: Error::Redis,
                }
            }
            Query::AuthSessionRefresh { refresh } => Reply::Unimplemented,
            Query::AuthSessionEnd { access, refresh } => Reply::Unimplemented,
            Query::AuthSmsSendTo { access } => Reply::Unimplemented,
            Query::AuthSmsSent { access } => Reply::Unimplemented,
            // Authenticate and pass to next layer
            q => {
                //TODO authenticate
                self.cost.handle(q)
            }
        }
    }
}
