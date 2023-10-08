use clap::Parser;
use std::process::exit;
use vcli::{
    cli::{Cli, Command, CostCommand, GeneCommand, MemeCommand},
    client::Client,
};

fn main() {
    let cli = Cli::parse();
    let client = Client::default();

    let result = match cli.command {
        Command::Ping => client.ping(),
        Command::Auth => client.auth(),
        Command::Cost { command } => match command {
            CostCommand::Pay => client.cost_pay(),
            CostCommand::Get => client.cost_get(),
        },
        Command::Gene { fed, command } => match command {
            GeneCommand::Meta { gid } => client.gene_meta(fed, gid),
            GeneCommand::Call { gid, arg } => client.gene_call(fed, gid, arg),
        },
        Command::Meme { command } => match command {
            MemeCommand::Meta { hash } => client.meme_meta(hash),
            MemeCommand::Put { days, file } => client.meme_put(days, file),
            MemeCommand::Get { public, hash, file } => client.meme_get(public, hash, file),
        },
        Command::Map { file } => client.gene_map(file),
    };

    match result {
        Ok(s) => println!("{}", s),
        Err(error) => {
            eprintln!("{}", error);
            exit(1)
        }
    }
}
