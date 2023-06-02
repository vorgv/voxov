//! Big TODO here.
//! Memes can't be redirected.

use tokio_util::sync::CancellationToken;

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
        costs: &Costs,
        token: CancellationToken,
    ) -> Reply {
        match query.get_fed() {
            Some(_) => Reply::Unimplemented,
            None => self.gene.handle(query, uid, costs, token).await,
        }
    }
}
