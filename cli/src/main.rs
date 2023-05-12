use std::io;
use std::str::FromStr;
use tokio::sync::OnceCell;
use voxov::auth::nspm;
use voxov::config::Config;
use voxov::database::namespace::SMSSENT;
use voxov::database::Database;
use voxov::message::id::Id;

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
            s if s == "sent" => {
                let phone = &self.argv[1];
                println!("'{}'", &self.argv[2]);
                let message = Id::from_str(format!("{:0>32}", self.argv[2]).as_str()).unwrap();
                let user_phone = &self.argv[3];
                let s = nspm(SMSSENT, &phone, &message);
                match self
                    .db
                    .set(&s[..], user_phone, self.config.access_ttl)
                    .await
                {
                    Ok(_) => "Ok".to_string(),
                    Err(e) => e.to_string(),
                }
            }
            unknown => format!("Unknown command: {}", unknown),
        }
    }
}
