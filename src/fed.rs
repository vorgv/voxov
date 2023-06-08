//! Big TODO here.
//! Memes can't be redirected.

use crate::config::Config;
use crate::gene::Gene;
use crate::message::{Costs, Id, Query, Reply};

pub struct Fed {
    gene: &'static Gene,
}

impl Fed {
    pub fn new(_config: &Config, gene: &'static Gene) -> Fed {
        Fed { gene }
    }
    pub async fn handle(
        &self,
        query: &Query,
        uid: &Id,
        change: Costs,
        deadline: tokio::time::Instant,
    ) -> Reply {
        match query.get_fed() {
            Some(_) => Reply::Unimplemented,
            None => self.gene.handle(query, uid, change, deadline).await,
        }
    }
}
