use crate::config::Config;
use crate::database::Database;
use crate::gene::Gene;

pub struct Fed {
    gene: Gene,
    db: &'static Database
}

impl Fed {
    pub fn new(_config: &Config, db: &'static Database, gene: Gene) -> Fed {
        Fed { gene, db }
    }
}
