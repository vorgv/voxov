#![feature(addr_parse_ascii)]
pub mod api;
pub mod auth;
pub mod config;
pub mod cost;
pub mod database;
pub mod error;
pub mod fed;
pub mod gene;
pub mod meme;
pub mod message;

pub mod macros {
    #[macro_export]
    macro_rules! to_static {
        ($e:expr) => {
            Box::leak(Box::new($e)) as &'static _
        };
    }
}
