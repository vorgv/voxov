use tokio_util::sync::CancellationToken;

use crate::config::Config;
use crate::database::Database;
use crate::meme::Meme;
use crate::message::{Costs, Id, Query, Reply};

pub struct Gene {
    meme: &'static Meme,
    db: &'static Database,
}

impl Gene {
    pub fn new(_config: &Config, db: &'static Database, meme: &'static Meme) -> Gene {
        Gene { meme, db }
    }
    pub fn handle(
        &self,
        query: &Query,
        uid: &Id,
        costs: &Costs,
        token: CancellationToken,
    ) -> Reply {
        match query {
            Query::GeneMeta { head, id } => Reply::Unimplemented,
            Query::GeneCall { head, id, arg } => Reply::Unimplemented,
            Query::MemeMeta { head, key } => Reply::Unimplemented,
            Query::MemeRawPut { head, key, raw } => Reply::Unimplemented,
            Query::MemeRawGet { head, key } => Reply::Unimplemented,
            _ => Reply::Error {
                error: crate::error::Error::Logical,
            },
        }
    }
}

pub struct GeneMeta {
    name: String,
    version: (), //TODO: semver
}
