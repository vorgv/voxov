use crate::config::Config;
use crate::cost::Cost;

pub struct Auth {
    cost: Cost
}

impl Auth {
    pub fn new(_config: &Config, cost: Cost) -> Auth {
        Auth {
            cost,
        }
    }
}
