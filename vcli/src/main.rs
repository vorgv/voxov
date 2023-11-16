use clap::Parser;
use std::process::exit;
use vcli::{
    cli::{AuthCommand, Cli, Command, CostCommand, GeneCommand, MemeCommand},
    client::Client,
};

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let client = Client::default().await;

    let result = match cli.command {
        Command::Ping => client.ping().await,
        Command::Auth { command } => match command {
            AuthCommand::Sms => client.auth_sms().await,
            AuthCommand::Skip { phone } => client.auth_skip(&phone).await,
        },
        Command::Cost { command } => match command {
            CostCommand::Pay => client.cost_pay().await,
            CostCommand::Get => client.cost_get().await,
        },
        Command::Gene { fed, command } => match command {
            GeneCommand::Meta { gid } => client.gene_meta(fed, &gid).await,
            GeneCommand::Call { gid, arg } => client.gene_call(fed, &gid, arg).await,
        },
        Command::Meme { command } => match command {
            MemeCommand::Meta { hash } => client.meme_meta(hash).await,
            MemeCommand::Put { days, file } => client.meme_put(days, file).await,
            MemeCommand::Get { public, hash, file } => client.meme_get(public, hash, file).await,
        },
        Command::Map { file } => client.gene_map_1(file).await,
    };

    match result {
        Ok(s) => println!("{}", s),
        Err(error) => {
            eprintln!("{}", error);
            exit(1)
        }
    }
}
