use std::{env, fs, path::PathBuf};

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Session {
    pub access: String,
    pub refresh: String,
    pub access_utc: String,
    pub refresh_utc: String,
    pub access_minutes: i64,
    pub refresh_days: i64,
}

impl Session {
    pub fn new(access: &str, refresh: &str) -> Self {
        let now = Utc::now().to_rfc3339();
        Session {
            access: access.into(),
            refresh: refresh.into(),
            access_utc: now.clone(),
            refresh_utc: now,
            access_minutes: 60,
            refresh_days: 30,
        }
    }

    pub fn set_access(&mut self, access: &str) {
        self.access = access.into();
        self.access_utc = Utc::now().to_rfc3339();
    }

    pub fn set_refresh(&mut self, refresh: &str) {
        self.refresh = refresh.into();
        self.refresh_utc = Utc::now().to_rfc3339();
    }

    pub fn access_expired(&self) -> bool {
        let then = DateTime::parse_from_rfc3339(&self.access_utc).unwrap();
        Utc::now() > then + Duration::minutes(self.access_minutes)
    }

    pub fn refresh_expired(&self) -> bool {
        let then = DateTime::parse_from_rfc3339(&self.refresh_utc).unwrap();
        Utc::now() > then + Duration::days(self.refresh_days)
    }

    pub fn needs_refresh(&self) -> bool {
        let then = DateTime::parse_from_rfc3339(&self.access_utc).unwrap();
        Utc::now() > then + Duration::minutes(self.access_minutes / 2)
    }
}

#[derive(Deserialize, Serialize)]
pub struct Plan {
    pub time: u64,
    pub space: u64,
    pub traffic: u64,
    pub tips: u64,
}

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub url: String,
    pub session: Option<Session>,
    pub plan: Plan,
}

impl Config {
    pub fn path() -> PathBuf {
        let mut config_path = env::current_dir().unwrap();
        if cfg!(target_os = "linux") {
            config_path = match env::var("XDG_CONFIG_HOME") {
                Ok(s) => PathBuf::from(s),
                Err(_) => {
                    let mut p = PathBuf::from(env::var("HOME").unwrap());
                    p.push(".config");
                    p
                }
            }
        } else if cfg!(target_os = "macos") {
            config_path = {
                let mut p = PathBuf::from(env::var("HOME").unwrap());
                p.push("Library/Application Support");
                p
            }
        } else if cfg!(target_os = "windows") {
            config_path = PathBuf::from(env::var("APPDATA").unwrap());
        } else {
            eprintln!("Config path is not supported on this OS, using current directory.");
        }
        config_path.push("voxov-cli");
        fs::create_dir_all(&config_path).unwrap();
        config_path.push("config.toml");
        config_path
    }

    pub fn load() -> Self {
        let config_path = Config::path();
        if config_path.is_file() {
            let s = fs::read_to_string(config_path).unwrap();
            toml::from_str(&s).unwrap()
        } else {
            let config = Self::default();
            config.save();
            config
        }
    }

    pub fn save(&self) {
        let s = toml::to_string(self).unwrap();
        fs::write(Config::path(), s).unwrap();
    }
}

impl Default for Config {
    fn default() -> Self {
        let default_cost = 1_000_000_000_u64;
        Config {
            url: "http://localhost:8080".into(),
            session: None,
            plan: Plan {
                time: default_cost,
                space: default_cost,
                traffic: default_cost,
                tips: default_cost,
            },
        }
    }
}
