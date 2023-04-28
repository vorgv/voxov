#![feature(addr_parse_ascii)]
mod message;
mod api;
mod auth;
mod config;
mod cost;
mod database;
mod fed;
mod gene;
mod meme;

extern crate tokio;
use std::error::Error;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    // Config: collect ENV
    let config = config::Config::new();
    // Database: database operations
    let database = database::Database::new(&config);
    // Meme: data types
    let meme = meme::Meme::new(&config, database);
    // Gene: functions
    let gene = gene::Gene::new(&config, meme);
    // Fed: call other instances
    let fed = fed::Fed::new(&config, gene);
    // Cost: set spacetime limit
    let cost = cost::Cost::new(&config, fed);
    // Auth: authentication
    let auth = auth::Auth::new(&config, cost);
    // API: GraphQL & Static
    let api = api::Api::new(&config, auth);
    // Serve
    api.serve().await
}
