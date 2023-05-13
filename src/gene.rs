use crate::config::Config;
use crate::database::Database;
use crate::meme::Meme;

pub struct Gene {
    meme: &'static Meme,
    db: &'static Database,
}

impl Gene {
    pub fn new(_config: &Config, db: &'static Database, meme: &'static Meme) -> Gene {
        Gene { meme, db }
    }
}
