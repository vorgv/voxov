use crate::config::Config;
use crate::meme::Meme;

pub struct Gene {
    meme: Meme
}

impl Gene {
    pub fn new(_config: &Config, meme: Meme) -> Gene {
        Gene {
            meme,
        }
    }
}
