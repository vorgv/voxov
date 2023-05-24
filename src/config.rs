use std::net::IpAddr;
use std::{
    env,
    net::{Ipv4Addr, SocketAddr},
};
use crate::{gene::GeneMeta, to_static};

pub struct Config {
    pub redis_addr: String,
    pub mongo_addr: String,
    pub http_addr: SocketAddr,
    pub access_ttl: usize,
    pub refresh_ttl: usize,
    pub user_ttl: usize,
    pub auth_phones: &'static Vec<String>,
    pub gene_metas: &'static Vec<GeneMeta>,
    //pub fed_members: &'static HashMap<Id, String>,
}

const DEFAULT_HTTP_PORT: u16 = 8080;
/// Changing this constant invalidates the old auth phones.
/// Never touch this in production.
pub const PHONE_MAX_BYTES: usize = 16;

impl Config {
    pub fn new() -> Config {
        Config {
            redis_addr: match env::var("REDIS_ADDR") {
                Ok(var) => var,
                Err(_) => String::from("redis://localhost/"),
            },
            mongo_addr: match env::var("MONGO_ADDR") {
                Ok(var) => var,
                Err(_) => String::from("mongodb://127.0.0.1:27017/"),
            },
            http_addr: match env::var("HTTP_ADDR") {
                Ok(var) => SocketAddr::parse_ascii(var.as_bytes()).unwrap(),
                Err(_) => {
                    SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), DEFAULT_HTTP_PORT)
                }
            },
            access_ttl: match env::var("ACCESS_TTL") {
                Ok(var) => var.parse().unwrap(),
                Err(_) => 60 * 60, // one hour
            },
            refresh_ttl: match env::var("REFRESH_TTL") {
                Ok(var) => var.parse().unwrap(),
                Err(_) => 60 * 60 * 24 * 30, // one month
            },
            user_ttl: match env::var("USER_TTL") {
                Ok(var) => var.parse().unwrap(),
                Err(_) => 60 * 60 * 24 * 365 * 5, // 5 years
            },
            auth_phones: Box::leak(Box::new(match env::var("AUTH_PHONES") {
                Ok(var) => {
                    let ap: Vec<_> = var.split(':').map(String::from).collect();
                    let max_bytes = ap.iter().map(|s| s.as_bytes().len()).max().unwrap();
                    if max_bytes > PHONE_MAX_BYTES {
                        panic!("Phone number too long")
                    }
                    ap
                }
                Err(_) => vec!["12345".to_string(), "67890".to_string()],
            })) as &'static _,
            //TODO: load external genes.
            gene_metas: to_static!(vec![]),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Config::new()
    }
}
