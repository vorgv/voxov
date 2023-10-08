pub mod cli;
pub mod client;
pub mod config;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
