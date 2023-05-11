use std::io;
use tokio::sync::OnceCell;
use voxov::config::Config;
use voxov::database::Database;

static CONFIG: OnceCell<Config> = OnceCell::const_new();

async fn get_config() -> &'static Config {
    CONFIG.get_or_init(|| async { Config::new() }).await
}

static DB: OnceCell<Database> = OnceCell::const_new();

async fn get_db() -> &'static Database {
    DB.get_or_init(|| async { Database::new(get_config().await).await })
        .await
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let config = get_config().await;
    let db = get_db().await;
    let mut line_buffer = String::new();
    loop {
        line_buffer.clear();
        io::stdin().read_line(&mut line_buffer)?;
        let command = Command::new(&line_buffer, config, db);
        let output = command.execute().await;
        println!("{}", output);
    }
}

struct Command {
    argv: Vec<String>,
    config: &'static Config,
    db: &'static Database,
}

impl Command {
    fn new(s: &String, config: &'static Config, db: &'static Database) -> Self {
        Command {
            argv: s.split_whitespace().map(String::from).collect(),
            config,
            db,
        }
    }
    async fn execute(&self) -> String {
        if self.argv.is_empty() {
            return "".to_string();
        }
        match &self.argv[0] {
            unknown => format!("Unknown command: {}", unknown),
        }
    }
}
