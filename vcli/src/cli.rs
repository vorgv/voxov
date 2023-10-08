use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Ping the server.
    Ping,
    /// Interactively authenticate with SMS.
    Auth,
    /// Balance related subcommands.
    Cost {
        #[command(subcommand)]
        command: CostCommand,
    },
    /// Function related subcommands.
    Gene {
        /// Defaults to the local instance.
        #[arg(short, long)]
        fed: Option<String>,
        #[command(subcommand)]
        command: GeneCommand,
    },
    /// Data related subcommands.
    Meme {
        #[command(subcommand)]
        command: MemeCommand,
    },
    /// Gene map.
    Map { file: Option<String> },
}

#[derive(Subcommand)]
pub enum CostCommand {
    /// Get the link to pay.
    Pay,
    /// Get the account balance.
    Get,
}

#[derive(Subcommand)]
pub enum GeneCommand {
    /// Get the metadata with GID.
    Meta { gid: String },
    /// Call the gene with ARG.
    Call { gid: String, arg: Option<String> },
}

#[derive(Subcommand)]
pub enum MemeCommand {
    /// Get the metadata of the meme by HASH.
    Meta { hash: String },
    /// Put the FILE as a meme, then keep DAYS days.
    Put { days: u32, file: Option<String> },
    /// Get meme by HASH. -p means public meme. Optionally saves to FILE.
    Get {
        #[arg(short, long)]
        public: bool,
        hash: String,
        file: Option<String>,
    },
}
