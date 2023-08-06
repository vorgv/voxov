use voxov::{api, auth, config, cost, database, fed, gene, meme, to_static};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    console_subscriber::init();

    // Config: collect ENV to static variables.
    let c = to_static!(config::Config::new());

    // Database: stateless database struct.
    let db = to_static!(database::Database::new(c).await);

    // Meme: data primitives.
    let meme: &'static meme::Meme = to_static!(meme::Meme::new(c, db));

    // Ripperd: delete expired data.
    tokio::spawn(meme.ripperd());

    // Gene: function primitives.
    let gene = to_static!(gene::Gene::new(c, db, meme));

    // Fed: call other instances.
    let fed = to_static!(fed::Fed::new(c, gene));

    // Cost: set limit on time, space, traffic and tip.
    let cost = to_static!(cost::Cost::new(c, db, fed));

    // Auth: OAuth 2.0 style authentication.
    let auth = to_static!(auth::Auth::new(c, db, cost));

    // API: GraphQL & plain http.
    let api: &'static api::Api = to_static!(api::Api::new(c, auth));

    // Open endpoints.
    api.serve().await
}
