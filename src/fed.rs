//! Big TODO here.
//! Memes can't be redirected.

use crate::config::Config;
use crate::gene::Gene;
use crate::ir::{Costs, Id, Query, Reply};
use crate::Result;

pub struct Fed {
    gene: &'static Gene,
}

impl Fed {
    pub fn new(_config: &Config, gene: &'static Gene) -> Fed {
        Fed { gene }
    }
    pub async fn handle(
        &self,
        query: Query,
        uid: &Id,
        changes: Costs,
        deadline: tokio::time::Instant,
    ) -> Result<Reply> {
        match query.get_fed() {
            Some(_) => Ok(Reply::Unimplemented),
            None => self.gene.handle(query, uid, changes, deadline).await,
        }
    }
}
