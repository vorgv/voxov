use std::{
    env,
    net::{IpAddr, Ipv4Addr, SocketAddr},
};

pub struct Config {
    pub redis_addr: String,
    pub static_addr: SocketAddr,
}

const STATIC_PORT: u16 = 8080;

impl Config {
    pub fn new() -> Config {
        Config {
            redis_addr: match env::var("REDIS_ADDR") {
                Ok(var) => var,
                Err(_) => String::from("redis://localhost/"),
            },
            static_addr: match env::var("STATIC_ADDR") {
                Ok(var) => SocketAddr::parse_ascii(var.as_bytes()).unwrap(),
                Err(_) => SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), STATIC_PORT),
            },
        }
    }
}
