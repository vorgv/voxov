#![feature(addr_parse_ascii)]
mod api;
mod auth;
mod config;
mod cost;
mod database;
mod fed;
mod gene;
mod meme;
mod message;

use config::Config;
use database::Database;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    // Config: collect ENV to global variable only once
    let c = Box::leak(Box::new(Config::new())) as &'static _;
    // Database: stateless global database struct
    let db = Box::leak(Box::new(Database::new(c).await)) as &'static _;
    // Meme: data types
    let meme = meme::Meme::new(c, db);
    // Gene: functions
    let gene = gene::Gene::new(c, db, meme);
    // Fed: call other instances
    let fed = fed::Fed::new(c, db, gene);
    // Cost: set spacetime limit
    let cost = cost::Cost::new(c, db, fed);
    // Auth: authentication
    let auth = auth::Auth::new(c, db, cost);
    // API: GraphQL & Static
    let api = api::Api::new(c, db, auth);
    // Serve
    api.serve().await
}
