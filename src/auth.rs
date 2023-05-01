use crate::config::Config;
use crate::cost::Cost;
use crate::message::{Query, Reply};

pub struct Auth {
    cost: Cost,
}

impl Auth {
    pub fn new(_config: &Config, cost: Cost) -> Auth {
        Auth { cost }
    }
    pub fn handle(self: &Self, q: &Query) -> Reply {
        Reply::Unknown
    }
}
