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

pub type Result<T> = std::result::Result<T, error::Error>;

#[tokio::main]
pub async fn main() -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
    console_subscriber::init();

    // Config: collect ENV to static variables.
    let c = to_static!(config::Config::new());

    // Database: stateless database struct.
    let db = to_static!(database::Database::new(c, true).await);

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
