#![feature(addr_parse_ascii)]
#![feature(io_error_more)]
pub mod api;
pub mod auth;
pub mod body;
pub mod config;
pub mod cost;
pub mod database;
pub mod error;
pub mod fed;
pub mod gene;
pub mod ir;
pub mod meme;

pub mod macros {
    #[macro_export]
    macro_rules! to_static {
        ($e:expr) => {
            Box::leak(Box::new($e)) as &'static _
        };
    }
}

type Result<T> = std::result::Result<T, error::Error>;
