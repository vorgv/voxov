use crate::{gene::GeneMeta, to_static};
use std::net::IpAddr;
use std::{
    env,
    net::{Ipv4Addr, SocketAddr},
};

/// Static config struct. Modification requires relaunch.
/// All constants are from environment variables.
pub struct Config {
    /// Redis URI.
    pub redis_addr: String,

    /// MongoDB URI.
    pub mongo_addr: String,

    /// S3 or compatible object storage URI.
    pub s3_addr: String,

    /// S3 or compativle objest storage region.
    pub s3_region: String,

    /// Endpoint API in http.
    pub http_addr: SocketAddr,

    // graphql_addr
    /// Seconds before access token expire.
    pub access_ttl: usize,

    /// Seconds before refresh token expire.
    pub refresh_ttl: usize,

    /// Seconds before user account expire.
    pub user_ttl: usize,

    /// SMS receivers for authentication.
    pub auth_phones: &'static Vec<String>,

    /// Registered genes.
    pub gene_metas: &'static Vec<GeneMeta>,
    //pub fed_members: &'static HashMap<Id, String>,
}

/// Default port for http endpoint.
const DEFAULT_HTTP_PORT: u16 = 8080;

/// Changing this constant invalidates all phone numbers!!
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

            s3_addr: match env::var("S3_ADDR") {
                Ok(var) => var,
                Err(_) => String::from("http://127.0.0.1:9000/"),
            },

            s3_region: match env::var("S3_REGION") {
                Ok(var) => var,
                Err(_) => String::from("develop"),
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
