use voxov::{api, auth, config, cost, database, fed, gene, meme, to_static};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Config: collect ENV to global variable only once
    let c = to_static!(config::Config::new());
    // Database: stateless global database struct
    let db = to_static!(database::Database::new(c).await);
    // Meme: data types
    let meme: &'static meme::Meme = to_static!(meme::Meme::new(c, db));
    tokio::spawn(meme.ripperd());
    // Gene: functions
    let gene = to_static!(gene::Gene::new(c, db, meme));
    // Fed: call other instances
    let fed = to_static!(fed::Fed::new(c, gene));
    // Cost: set spacetime limit
    let cost = to_static!(cost::Cost::new(c, db, fed));
    // Auth: authentication
    let auth = to_static!(auth::Auth::new(c, db, cost));
    // API: GraphQL & Static
    let api: &'static api::Api = to_static!(api::Api::new(c, auth));
    // Serve
    api.serve().await
}
