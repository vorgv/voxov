use clap::{Parser, Subcommand};
use std::process::exit;
use std::str::FromStr;
use voxov::auth::nspm;
use voxov::config::Config;
use voxov::database::namespace::{SMSSENT, UID2CREDIT};
use voxov::database::{ns, Database};
use voxov::message::Id;
use voxov::to_static;

#[tokio::main]
async fn main() {
    let c = to_static!(Config::new());
    let db: &Database = to_static!(Database::new(c).await);

    let cli = Cli::parse();
    let result = match cli.command {
        Command::Sent { from, to, message } => {
            let message = Id::from_str(format!("{:0>32}", message).as_str()).unwrap();
            let s = nspm(SMSSENT, &to, &message);
            db.set(&s[..], from, c.access_ttl).await
        }

        Command::AddCredit { uid, credit } => {
            let u2c = ns(UID2CREDIT, &Id::from_str(&uid).unwrap());
            db.incrby(&u2c[..], credit).await
        }
    };

    if result.is_err() {
        exit(1);
    }
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// MESSAGE received from FROM to TO.
    Sent {
        from: String,
        to: String,
        message: String,
    },

    /// Add credit to UID
    AddCredit { uid: String, credit: i64 },
}
