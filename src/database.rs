extern crate redis;

use crate::config::Config;
use redis::{Connection, RedisError};

pub struct Database {
    connection: redis::Connection,
}

fn connect_redis(addr: &str) -> Result<Connection, RedisError> {
    let client = redis::Client::open(addr).unwrap();
    client.get_connection()
}

impl Database {
    pub fn new(config: &Config) -> Database {
        Database {
            connection: match connect_redis(&config.redis_addr()) {
                Ok(con) => con,
                Err(_) => panic!("connection failed, todo trace"),
            },
        }
    }
}
