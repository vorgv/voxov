use std::env;

pub struct Config {
    redis_addr: String,
}

impl Config {
    pub fn new() -> Config {
        Config {
            redis_addr: match env::var("REDIS_ADDR") {
                Ok(var) => var,
                Err(_) => String::from("redis://localhost/"),
            }
        }
    }
    pub fn redis_addr(&self) -> &String {
        &self.redis_addr
    }
}
