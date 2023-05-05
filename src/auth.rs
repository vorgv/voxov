use crate::config::Config;
use crate::cost::Cost;
use crate::database::Database;
use crate::message::{Query, Reply};

pub struct Auth {
    cost: Cost,
    db: &'static Database,
}

impl Auth {
    pub fn new(_config: &Config, db: &'static Database, cost: Cost) -> Auth {
        Auth { cost, db }
    }
    pub fn handle(&self, query: &Query) -> Reply {
        match query {
            // Session management
            Query::AuthSessionStart => Reply::Unimplemented,
            Query::AuthSessionRefresh { refresh } => Reply::Unimplemented,
            Query::AuthSessionEnd { access, refresh } => Reply::Unimplemented,
            Query::AuthSmsSendTo { access } => Reply::Unimplemented,
            Query::AuthSmsSent { access } => Reply::Unimplemented,
            // Authenticate and pass to next layer
            q => {
                //TODO authenticate
                self.cost.handle(&q)
            },
        }
    }
}
