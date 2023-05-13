use crate::config::Config;
use crate::database::Database;
use crate::fed::Fed;
use crate::message::{Id, Query, Reply};

pub struct Cost {
    fed: &'static Fed,
    db: &'static Database,
}

impl Cost {
    pub fn new(_config: &Config, db: &'static Database, fed: &'static Fed) -> Cost {
        Cost { fed, db }
    }
    pub fn handle(&self, _uid: &Id, _query: &Query) -> Reply {
        Reply::Unimplemented
    }
}
