use tokio_util::sync::CancellationToken;

use crate::config::Config;
use crate::database::Database;
use crate::gene::Gene;
use crate::message::{Costs, Id, Query, Reply};

pub struct Fed {
    gene: &'static Gene,
    db: &'static Database,
}

impl Fed {
    pub fn new(_config: &Config, db: &'static Database, gene: &'static Gene) -> Fed {
        Fed { gene, db }
    }
    pub async fn handle(
        &self,
        query: &Query,
        uid: &Id,
        costs: &Costs,
        token: CancellationToken,
    ) -> Reply {
        Reply::Unimplemented
    }
}
