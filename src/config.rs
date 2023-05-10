use std::{
    env,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Arc,
};

pub struct Config {
    pub redis_addr: String,
    pub static_addr: SocketAddr,
    pub access_ttl: usize,
    pub refresh_ttl: usize,
    pub auth_phones: Arc<Vec<String>>,
}

const STATIC_PORT: u16 = 8080;
/// Changing this invalidates the old auth phones
pub const PHONE_MAX_BYTES: usize = 16;

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
            auth_phones: Arc::new(match env::var("AUTH_PHONES") {
                Ok(var) => {
                    let ap: Vec<_> = var.split(":").map(String::from).collect();
                    let max_bytes = ap.iter().map(|s| s.as_bytes().len()).max().unwrap();
                    if max_bytes > PHONE_MAX_BYTES {
                        panic!("Phone number too long")
                    }
                    ap
                }
                Err(_) => vec!["12345".to_string(), "67890".to_string()],
            }),
        }
    }
}
