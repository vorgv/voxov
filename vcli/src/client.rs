use crate::config::{Config, Plan, Session};
use crate::Result;
use reqwest::blocking::{get, Client as ReqwestClient, RequestBuilder, Response};
use std::{
    fs::File,
    io::{stdin, Read, Write},
    process::exit,
    time::Duration,
};

/// Client state struct.
pub struct Client {
    config: Config,
}

/// If response is error type, print error message and exit.
macro_rules! handle_error {
    ($response:expr) => {
        let t = get_header(&$response, "type");
        if t == "Error" {
            let e = get_header(&$response, "error");
            eprintln!("{}", e);
            exit(1);
        }
    };
}

impl Client {
    /// Check connectivity.
    pub fn ping(&self) -> Result<String> {
        Ok(get(&self.config.url)?.text()?)
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
            .header("access", &self.config.session.as_ref().unwrap().access)
            .header("time", self.config.plan.time.to_string())
            .header("space", self.config.plan.space.to_string())
            .header("traffic", self.config.plan.traffic.to_string())
            .header("tips", self.config.plan.tips.to_string());
        if let Some(f) = fed {
            builder = builder.header("fed", f);
        }
        builder
    }

    /// Refresh or remake session.
    fn update_session(&mut self) {
        match &self.config.session {
            Some(s) if !s.refresh_expired() => {
                if s.needs_refresh() {
                    let access = self.auth_session_refresh().unwrap();
                    self.config.session.as_mut().unwrap().set_access(&access);
                    self.config.save();
                }
            }
            x => {
                if x.is_some() {
                    eprintln!("Refresh token expired. Session is reset for re-authentication.");
                }
                let (access, refresh) = self.auth_session_start().unwrap();
                self.config.session = Some(Session::new(&access, &refresh));
                self.config.save();
            }
        };
    }

    /// Authenticate interactively.
    pub fn auth(&self) -> Result<String> {
        let (phone, message) = self.auth_sms_send_to()?;
        println!("Send SMS message {} to {}.", message, phone);
        println!("Press enter after sent.");
        let mut s = "".to_string();
        let _ = stdin().read_line(&mut s);
        let uid = self.auth_sms_sent(&phone, &message)?;
        Ok(format!("Your user ID is {}", uid))
    }

    /// Get access and refresh tokens.
    pub fn auth_session_start(&self) -> Result<(String, String)> {
        let response = self.post().header("type", "AuthSessionStart").send()?;
        handle_error!(response);
        let access = get_header(&response, "access");
        let refresh = get_header(&response, "refresh");
        Ok((access, refresh))
    }

    /// Get a new access with refresh token.
    pub fn auth_session_refresh(&self) -> Result<String> {
        let response = self
            .post()
            .header("type", "AuthSessionRefresh")
            .header("refresh", &self.config.session.as_ref().unwrap().refresh)
            .send()?;
        handle_error!(response);
        let access = get_header(&response, "access");
        Ok(access)
    }

    /// Deactivate tokens.
    pub fn auth_session_end(&self, drop_refresh: bool) -> Result<()> {
        let mut builder = self
            .post()
            .header("type", "AuthSessionEnd")
            .header("access", &self.config.session.as_ref().unwrap().access);
        if drop_refresh {
            builder = builder.header("refresh", &self.config.session.as_ref().unwrap().refresh);
        }
        let response = builder.send()?;
        handle_error!(response);
        Ok(())
    }

    /// Get where to send SMS.
    pub fn auth_sms_send_to(&self) -> Result<(String, String)> {
        let response = self
            .post()
            .header("type", "AuthSmsSendTo")
            .header("access", &self.config.session.as_ref().unwrap().access)
            .header("refresh", &self.config.session.as_ref().unwrap().refresh)
            .send()?;
        handle_error!(response);
        let phone = get_header(&response, "phone");
        let message = get_header(&response, "message");
        Ok((phone, message))
    }

    /// Notify the server that SMS is sent.
    pub fn auth_sms_sent(&self, phone: &str, message: &str) -> Result<String> {
        let response = self
            .post()
            .header("type", "AuthSmsSent")
            .header("access", &self.config.session.as_ref().unwrap().access)
            .header("refresh", &self.config.session.as_ref().unwrap().refresh)
            .header("phone", phone)
            .header("message", message)
            .send()?;
        handle_error!(response);
        let uid = get_header(&response, "uid");
        Ok(uid)
    }

    /// Get the link to pay.
    pub fn cost_pay(&self) -> Result<String> {
        let response = self
            .post()
            .header("type", "CostPay")
            .header("access", &self.config.session.as_ref().unwrap().access)
            .header("vendor", "00000000000000000000000000000000")
            .send()?;
        handle_error!(response);
        let uri = get_header(&response, "uri");
        Ok(uri)
    }

    /// Get user balance.
    pub fn cost_get(&self) -> Result<String> {
        let response = self
            .post()
            .header("type", "CostGet")
            .header("access", &self.config.session.as_ref().unwrap().access)
            .send()?;
        handle_error!(response);
        let credit = get_header(&response, "credit");
        Ok(credit)
    }

    /// Print cost based on plan and returned changes.
    pub fn print_cost(&self, response: &Response) -> Result<()> {
        macro_rules! get {
            ($s:expr) => {
                get_header(response, $s).parse()?
            };
        }
        let changes = Plan {
            time: get!("time"),
            space: get!("space"),
            traffic: get!("traffic"),
            tips: get!("tips"),
        };
        let plan = &self.config.plan;
        println!(
            "time {} space {} traffic {} tips {}",
            plan.time - changes.time,
            plan.space - changes.space,
            plan.traffic - changes.traffic,
            plan.tips - changes.tips
        );
        Ok(())
    }

    /// Get functions metadata.
    pub fn gene_meta(&self, fed: Option<String>, gid: String) -> Result<String> {
        let response = self
            .post_head(fed)
            .header("type", "GeneMeta")
            .header("gid", gid)
            .send()?;
        handle_error!(response);
        self.print_cost(&response)?;
        Ok(response.text()?)
    }

    /// Call function.
    pub fn gene_call(
        &self,
        fed: Option<String>,
        gid: String,
        arg: Option<String>,
    ) -> Result<String> {
        let response = self
            .post_head(fed)
            .header("type", "GeneCall")
            .header("gid", gid)
            .header("arg", arg.unwrap_or_default())
            .send()?;
        handle_error!(response);
        self.print_cost(&response)?;
        Ok(response.text()?)
    }

    /// Get metadata of a meme.
    pub fn meme_meta(&self, hash: String) -> Result<String> {
        let response = self
            .post_head(None)
            .header("type", "MemeMeta")
            .header("hash", hash)
            .send()?;
        handle_error!(response);
        self.print_cost(&response)?;
        Ok(response.text()?)
    }

    /// Upload a file.
    pub fn meme_put(&self, days: u32, file: Option<String>) -> Result<String> {
        let mut builder = self
            .post_head(None)
            .header("type", "MemePut")
            .header("days", days);
        builder = match file {
            Some(file) => {
                let file = File::open(file)?;
                builder.body(file)
            }
            None => builder.body({
                let mut buf = Vec::<u8>::new();
                std::io::stdin().read_to_end(&mut buf)?;
                buf
            }),
        };
        let response = builder.send()?;
        handle_error!(response);
        self.print_cost(&response)?;
        let hash = get_header(&response, "hash");
        Ok(hash)
    }

    /// Download a file.
    pub fn meme_get(
        &self,
        public: bool,
        hash: String,
        file: Option<String>,
    ) -> Result<String> {
        let mut builder = self
            .post_head(None)
            .header("type", "MemeGet")
            .header("hash", hash);
        builder = match public {
            true => builder.header("public", "true"),
            false => builder.header("public", "false"),
        };
        let response = builder.send()?;
        handle_error!(response);
        if file.is_some() {
            self.print_cost(&response)?;
        }
        match file {
            Some(file) => {
                let mut file = File::create(file)?;
                file.write_all(&response.bytes()?)?;
                Ok("".into())
            }
            None => {
                std::io::stdout().write_all(&response.bytes()?)?;
                exit(0);
            }
        }
    }

    /// Map operations.
    pub fn gene_map(&self, file: Option<String>) -> Result<String> {
        match file {
            Some(file) => {
                let mut file = File::open(file)?;
                let mut buf = String::new();
                file.read_to_string(&mut buf)?;
                self.gene_call(None, "1".into(), Some(buf))
            }
            None => {
                let mut buf = Vec::<u8>::new();
                std::io::stdin().read_to_end(&mut buf)?;
                let buf = String::from_utf8(buf)?;
                self.gene_call(None, "1".into(), Some(buf))
            }
        }
    }
}

impl Default for Client {
    fn default() -> Self {
        let config = Config::load();
        let mut client = Client { config };
        client.update_session();
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
