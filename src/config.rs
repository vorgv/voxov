use crate::message::Uint;
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
    pub access_ttl: Uint,

    /// Seconds before refresh token expire.
    pub refresh_ttl: Uint,

    /// Seconds before user account expire.
    pub user_ttl: Uint,

    /// Cost per millisecond
    pub time_cost: Uint,

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

/// Set an entry by environment variable, or use the default.
macro_rules! env_or {
    ($e:literal, $d:expr) => {
        match env::var($e) {
            Ok(var) => var.parse().unwrap(),
            Err(_) => $d.into(),
        }
    };
}

impl Config {
    pub fn new() -> Config {
        Config {
            redis_addr: env_or!("REDIS_ADDR", "redis://localhost/"),

            mongo_addr: env_or!("MONGO_ADDR", "mongodb://127.0.0.1:27017/"),

            s3_addr: env_or!("S3_ADDR", "http://127.0.0.1:9000/"),

            s3_region: env_or!("S3_REGION", "develop"),

            http_addr: match env::var("HTTP_ADDR") {
                Ok(var) => SocketAddr::parse_ascii(var.as_bytes()).unwrap(),
                Err(_) => {
                    SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), DEFAULT_HTTP_PORT)
                }
            },

            access_ttl: env_or!("ACCESS_TTL", 60 * 60 as Uint), // one hour

            refresh_ttl: env_or!("REFRESH_TTL", 60 * 60 * 24 * 30 as Uint), // one month

            user_ttl: env_or!("USER_TTL", 60 * 60 * 24 * 365 * 5 as Uint), // 5 years

            time_cost: env_or!("TIME_COST", 1000 as Uint),

            auth_phones: to_static!(match env::var("AUTH_PHONES") {
                Ok(var) => {
                    let ap: Vec<_> = var.split(':').map(String::from).collect();
                    let max_bytes = ap.iter().map(|s| s.as_bytes().len()).max().unwrap();
                    if max_bytes > PHONE_MAX_BYTES {
                        panic!("Phone number too long")
                    }
                    ap
                }
                Err(_) => vec!["12345".to_string(), "67890".to_string()],
            }),

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
