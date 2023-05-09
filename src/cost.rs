use crate::config::Config;
use crate::database::Database;
use crate::fed::Fed;
use crate::message::{Id, Query, Reply};

pub struct Cost {
    fed: Fed,
    db: &'static Database,
}

impl Cost {
    pub fn new(_config: &Config, db: &'static Database, fed: Fed) -> Cost {
        Cost { fed, db }
    }
    pub fn handle(&self, uid: &Id, query: &Query) -> Reply {
        Reply::Unimplemented
    }
}
