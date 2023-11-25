mod auth;
mod cost;
mod gene;
mod meme;

use crate::config::{Config, Plan, Session};
use crate::Result;
use reqwest::{get, Client as ReqwestClient, RequestBuilder, Response};
use std::{error, fmt};
use std::{io::stdin, time::Duration};

#[macro_use]
mod macros {
    /// If response is error type, print error message and exit.
    #[macro_export]
    macro_rules! handle_error {
        ($response:expr) => {
            let t = get_header(&$response, "type");
            if t == "Error" {
                let e = get_header(&$response, "error");
                eprintln!("{}", e);
                std::process::exit(1);
            }
        };
    }
}

/// Client state struct.
pub struct Client {
    pub config: Config,
}

impl Client {
    /// Check connectivity.
    pub async fn ping(&self) -> Result<String> {
        Ok(get(&self.config.url).await?.text().await?)
    }

    /// The http POST method.
    fn post(&self) -> RequestBuilder {
        ReqwestClient::new().post(&self.config.url)
    }

    /// Post, but with head included.
    fn post_head(&self, fed: Option<String>) -> RequestBuilder {
        let mut builder = self
            .post()
            .timeout(Duration::from_secs(60 * 60 * 24 * 30))
            .header("access", &self.get_access().unwrap())
            .header("time", self.config.plan.time.to_string())
            .header("space", self.config.plan.space.to_string())
            .header("traffic", self.config.plan.traffic.to_string())
            .header("tip", self.config.plan.tip.to_string());
        if let Some(f) = fed {
            builder = builder.header("fed", f);
        }
        builder
    }

    /// Get the access token.
    fn get_access(&self) -> Result<String> {
        Ok(self
            .config
            .session
            .as_ref()
            .ok_or(VcliError)?
            .access
            .clone())
    }

    /// Get the refresh token.
    fn get_refresh(&self) -> Result<String> {
        Ok(self
            .config
            .session
            .as_ref()
            .ok_or(VcliError)?
            .refresh
            .clone())
    }

    /// Refresh or remake session.
    pub async fn update_session(&mut self) {
        match &self.config.session {
            Some(s) if !s.refresh_expired() => {
                if s.needs_refresh() {
                    let access = self.auth_session_refresh().await.unwrap();
                    self.config.session.as_mut().unwrap().set_access(&access);
                    self.config.save();
                }
            }
            x => {
                if x.is_some() {
                    eprintln!("Refresh token expired. Please re-authentication.");
                }
                let (access, refresh) = self.auth_session_start().await.unwrap();
                self.config.session = Some(Session::new(&access, &refresh));
                self.config.save();
            }
        };
    }

    /// Authenticate interactively.
    pub async fn auth_sms(&self) -> Result<String> {
        let (phone, message) = self.auth_sms_send_to().await?;
        println!("Send SMS message {} to {}.", message, phone);
        println!("Press enter after sent.");
        let mut s = "".to_string();
        let _ = stdin().read_line(&mut s);
        let uid = self.auth_sms_sent(&phone, &message).await?;
        Ok(format!("Your user ID is {}", uid))
    }

    /// Skip authentication.
    pub async fn auth_skip(&self, phone: &str) -> Result<String> {
        let uid = self.auth_sms_sent(phone, "").await?;
        Ok(format!("Your user ID is {}", uid))
    }

    /// Print cost based on plan and returned changes.
    pub fn eprint_cost(&self, response: &Response) -> Result<()> {
        macro_rules! get {
            ($s:expr) => {
                get_header(response, $s).parse()?
            };
        }
        let changes = Plan {
            time: get!("time"),
            space: get!("space"),
            traffic: get!("traffic"),
            tip: get!("tip"),
        };
        let plan = &self.config.plan;
        eprintln!(
            "time {} space {} traffic {} tip {}",
            plan.time - changes.time,
            plan.space - changes.space,
            plan.traffic - changes.traffic,
            plan.tip - changes.tip
        );
        Ok(())
    }
}

impl Client {
    pub async fn zero() -> Self {
        Client {
            config: Config::default(),
        }
    }
    pub async fn default() -> Self {
        let config = Config::load();
        let mut client = Client { config };
        client.update_session().await;
        client
    }
}

/// Get header's value by key.
fn get_header(response: &Response, key: &str) -> String {
    response.headers()[key]
        .to_str()
        .unwrap_or_default()
        .to_string()
}

#[derive(Debug, Clone)]
struct VcliError;

impl fmt::Display for VcliError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "VcliError")
    }
}

impl error::Error for VcliError {}
