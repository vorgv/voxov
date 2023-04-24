use crate::config::Config;
use crate::gene::Gene;

pub struct Fed {
    gene: Gene
}

impl Fed {
    pub fn new(_config: &Config, gene: Gene) -> Fed {
        Fed {
            gene,
        }
    }
}
