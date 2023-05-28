#![feature(addr_parse_ascii)]
mod api;
mod auth;
mod config;
mod cost;
mod database;
mod error;
mod fed;
mod gene;
mod meme;
mod message;

use voxov::to_static;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Config: collect ENV to global variable only once
    let c = to_static!(config::Config::new());
    // Database: stateless global database struct
    let db = to_static!(database::Database::new(c).await);
    // Meme: data types
    let meme = to_static!(meme::Meme::new(c, db));
    // Gene: functions
    let gene = to_static!(gene::Gene::new(c, db, meme));
    // Fed: call other instances
    let fed = to_static!(fed::Fed::new(c, db, gene));
    // Cost: set spacetime limit
    let cost = to_static!(cost::Cost::new(c, db, fed));
    // Auth: authentication
    let auth = to_static!(auth::Auth::new(c, db, cost));
    // API: GraphQL & Static
    let api: &'static api::Api = to_static!(api::Api::new(c, db, auth));
    // Serve
    api.serve().await
}
