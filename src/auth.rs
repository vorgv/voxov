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
    pub fn handle(&self, q: &Query) -> Reply {
        Reply::Unknown
    }
}
