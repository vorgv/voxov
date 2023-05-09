use std::{
    env,
    net::{IpAddr, Ipv4Addr, SocketAddr},
};

pub struct Config {
    pub redis_addr: String,
    pub static_addr: SocketAddr,
    pub access_ttl: usize,
    pub refresh_ttl: usize,
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
            access_ttl: match env::var("ACCESS_TTL") {
                Ok(var) => var.parse().unwrap(),
                Err(_) => 60 * 60, // one hour
            },
            refresh_ttl: match env::var("REFRESH_TTL") {
                Ok(var) => var.parse().unwrap(),
                Err(_) => 60 * 60 * 24 * 30, // one month
            },
        }
    }
}
