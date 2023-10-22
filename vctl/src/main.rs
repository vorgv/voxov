use clap::{Parser, Subcommand};
use std::process::exit;
use std::str::FromStr;
use voxov::config::Config;
use voxov::database::namespace::UID2CREDIT;
use voxov::database::{ns, Database};
use voxov::ir::Id;
use voxov::to_static;
use voxov::Result;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let result = execute(cli).await;
    if result.is_err() {
        exit(1);
    }
}

async fn execute(cli: Cli) -> Result<()> {
    let c = to_static!(Config::new());
    let db: &Database = to_static!(Database::new(c, false).await);

    match cli.command {
        Command::Sent { from, to, message } => db.sms_sent(&from, &to, &message).await,

        Command::AddCredit { uid, credit } => {
            let u2c = ns(UID2CREDIT, &Id::from_str(&uid)?);
            db.incrby(&u2c[..], credit).await
        }

        Command::DropIndexes => Ok(db.map1.drop_indexes(None).await?),
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

    /// Add credit to UID.
    AddCredit { uid: String, credit: i64 },

    /// Clear indexes of MongoDB,
    DropIndexes,
}
