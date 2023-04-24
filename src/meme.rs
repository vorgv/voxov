use crate::config::Config;
use crate::database::Database;

pub struct Meme {
    database: Database,
}

impl Meme {
    pub fn new(_config: &Config, database: Database) -> Meme {
        Meme { database }
    }
}
