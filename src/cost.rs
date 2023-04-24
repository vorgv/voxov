use crate::config::Config;
use crate::fed::Fed;

pub struct Cost {
    fed: Fed,
}

impl Cost {
    pub fn new(_config: &Config, fed: Fed) -> Cost {
        Cost { fed }
    }
}
