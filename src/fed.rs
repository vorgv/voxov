use crate::config::Config;
use crate::database::Database;
use crate::gene::Gene;
use crate::message::{Id, Int, Query, Reply};

pub struct Fed {
    gene: &'static Gene,
    db: &'static Database,
}

impl Fed {
    pub fn new(_config: &Config, db: &'static Database, gene: &'static Gene) -> Fed {
        Fed { gene, db }
    }
    pub async fn handle(&self, uid: &Id, space: &Int, query: &Query) -> Reply {
        Reply::Unimplemented
    }
}
