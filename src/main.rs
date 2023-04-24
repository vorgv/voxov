mod config;
mod database;
mod meme;
mod gene;
mod fed;
mod cost;
mod auth;
mod api;

extern crate tokio;

#[tokio::main]
pub async fn main() {
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
    api.serve();
}
