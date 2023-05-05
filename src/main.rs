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
use tokio::sync::OnceCell;

static CONFIG: OnceCell<Config> = OnceCell::const_new();

async fn get_config() -> &'static Config {
    CONFIG.get_or_init(|| async { Config::new() }).await
}

static DB: OnceCell<Database> = OnceCell::const_new();

async fn get_db() -> &'static Database {
    DB.get_or_init(|| async { Database::new(get_config().await) })
        .await
}

extern crate tokio;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    // Config: collect ENV to global variable only once
    let c = get_config().await;
    // Database: stateless global database struct
    let db = get_db().await;
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
