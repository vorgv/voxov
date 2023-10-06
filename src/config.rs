//! The config of VOxOV is fully static, therefore lock-free.
//! A restart is required to update any config.
//! To avoid interruption, prepend a load balancer.

use crate::{gene::GeneMeta, to_static};
use serde::Serialize;
use std::collections::HashMap;
use std::net::IpAddr;
use std::{
    env,
    net::{Ipv4Addr, SocketAddr},
};

/// Static config struct. Modification requires relaunch.
/// All constants are from environment variables.
#[derive(Serialize)]
pub struct Config {
    /// Source code. AGPL.
    pub source_code: String,

    /// Redis URI.
    #[serde(skip_serializing)]
    pub redis_addr: String,

    /// MongoDB URI.
    #[serde(skip_serializing)]
    pub mongo_addr: String,

    /// Ripperd handles meme expiration.
    /// 0 or unset for false, others for true.
    pub ripperd_disabled: bool,

    /// Ripperd's interval as seconds between meme rips.
    pub ripperd_interval: u64,

    /// S3 or compatible object storage URI.
    #[serde(skip_serializing)]
    pub s3_addr: String,

    /// S3 or compativle objest storage region.
    pub s3_region: String,

    /// S3 access key.
    #[serde(skip_serializing)]
    pub s3_access_key: String,

    /// S3 secret key.
    #[serde(skip_serializing)]
    pub s3_secret_key: String,

    /// Reset instance on resource draining.
    pub samsara: bool,

    /// Endpoint API in http.
    pub http_addr: SocketAddr,

    // graphql_addr
    /// Seconds before access token expire.
    pub access_ttl: i64,

    /// Seconds before refresh token expire.
    pub refresh_ttl: i64,

    /// Seconds before user account expire.
    pub user_ttl: i64,

    /// Minimum credit.
    pub credit_limit: i64,

    /// Cost per millisecond.
    pub time_cost: i64,

    /// Cost per KB per day in MongoDB.
    pub space_cost_doc: i64,

    /// Cost per KB per day in S3.
    pub space_cost_obj: i64,

    /// Cost per byte.
    pub traffic_cost: i64,

    /// Check-in award.
    pub check_in_award: i64,

    /// Seconds before check-in refresh.
    pub check_in_refresh: i64,

    /// SMS receivers for authentication.
    pub auth_phones: &'static Vec<String>,

    /// Skip auth.
    pub skip_auth: bool,

    /// Registered genes.
    pub gene_metas: &'static HashMap<String, GeneMeta>,
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

/// '0' or unset is false, otherwise is true.
macro_rules! env_bool {
    ($e:literal) => {
        match env::var($e) {
            Ok(x) if x == "0" => false,
            Ok(_) => true,
            _ => false,
        }
    };
}

impl Config {
    pub fn new() -> Config {
        Config {
            source_code: env_or!("SOURCE_CODE", "https://github.com/vorgv/voxov"),

            redis_addr: env_or!("REDIS_ADDR", "redis://localhost/"),

            mongo_addr: env_or!("MONGO_ADDR", "mongodb://127.0.0.1:27017/"),

            ripperd_disabled: env_bool!("RIPPERD_DISABLED"),

            ripperd_interval: env_or!("RIPPERD_INTERVAL", 60_u64), // seconds

            s3_addr: env_or!("S3_ADDR", "http://127.0.0.1:9000"),

            s3_region: env_or!("S3_REGION", "develop"),

            s3_access_key: env_or!("S3_ACCESS_KEY", "example-user"),

            s3_secret_key: env_or!("S3_SECRET_KEY", "example-password"),

            samsara: env_bool!("SAMSARA"),

            http_addr: match env::var("HTTP_ADDR") {
                Ok(var) => SocketAddr::parse_ascii(var.as_bytes()).unwrap(),
                Err(_) => {
                    SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), DEFAULT_HTTP_PORT)
                }
            },

            access_ttl: env_or!("ACCESS_TTL", 60 * 60_i64), // one hour

            refresh_ttl: env_or!("REFRESH_TTL", 60 * 60 * 24 * 30_i64), // one month

            user_ttl: env_or!("USER_TTL", 60 * 60 * 24 * 365 * 5_i64), // five years

            credit_limit: env_or!("CREDIT_LIMIT", -10_000_000_000_i64), // one USD

            time_cost: env_or!("TIME_COST", 1_000_i64), // per millisecond

            space_cost_doc: env_or!("SPACE_COST_DOC", 100_i64), // per KB per day

            space_cost_obj: env_or!("SPACE_COST_OBJ", 10_i64), // per KB per day

            traffic_cost: env_or!("TRAFFIC_COST", 1_i64), // per byte outbound

            check_in_award: env_or!("CHECK_IN_AWARD", 10_000_000_i64), // 1 GB/day storage

            check_in_refresh: env_or!("CHECK_IN_REFRESH", 60 * 60 * 24_i64), // 1 check-in/day

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

            skip_auth: env_bool!("SKIP_AUTH"),

            gene_metas: to_static!(GeneMeta::new_map()),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Config::new()
    }
}
