use crate::config::Config;
use crate::database::Database;

pub struct Meme {
    database: &'static Database,
}

impl Meme {
    pub fn new(_config: &Config, database: &'static Database) -> Meme {
        Meme { database }
    }
}
