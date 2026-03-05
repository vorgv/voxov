use clap::{Parser, Subcommand};
use std::process::exit;
use std::str::FromStr;
use voxov::config::Config;
use voxov::database::Database;
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
        Command::Sent { from, to, message } => {
            let message_id = Id::from_str(&message)?;
            db.sms_sent(&from, &to, &message_id.0).await
        }

        Command::AddCredit { uid, credit } => {
            let uid_id = Id::from_str(&uid)?;
            db.incr_credit(&uid_id, None, credit, "vctl add-credit")
                .await
        }
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
}
