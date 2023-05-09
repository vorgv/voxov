use crate::config::Config;
use crate::cost::Cost;
use crate::database::{namespace::ACCESS, namespace::REFRESH, Database};
use crate::message::{Error, Id, Query, Reply, IDL};
use bytes::{Buf, Bytes};

pub struct Auth {
    cost: Cost,
    db: &'static Database,
    access_ttl: usize,
    refresh_ttl: usize,
}

impl Auth {
    pub fn new(config: &Config, db: &'static Database, cost: Cost) -> Auth {
        Auth {
            cost,
            db,
            access_ttl: config.access_ttl,
            refresh_ttl: config.refresh_ttl,
        }
    }
    pub async fn handle(&self, query: &Query) -> Reply {
        match query {
            // Session management
            Query::AuthSessionStart => match self.handle_session_start().await {
                Ok(r) => r,
                Err(e) => Reply::AuthError { error: e },
            },
            Query::AuthSessionRefresh { refresh } => {
                match self.handle_session_refresh(refresh).await {
                    Ok(r) => r,
                    Err(e) => Reply::AuthError { error: e },
                }
            }
            Query::AuthSessionEnd { access, refresh } => Reply::Unimplemented,
            Query::AuthSmsSendTo { access } => Reply::Unimplemented,
            Query::AuthSmsSent { access } => Reply::Unimplemented,
            // Authenticate and pass to next layer
            q => {
                //TODO authenticate
                self.cost.handle(q)
            }
        }
    }
    async fn handle_session_start(&self) -> Result<Reply, Error> {
        let (access, refresh) = {
            let mut rng = rand::thread_rng();
            (Id::rand(&mut rng)?, Id::rand(&mut rng)?)
        }; // drop rng before await
        let pid = Id::zero();
        let a = ns(ACCESS, &access);
        self.db.set(&a[..], &pid.0, self.access_ttl).await?;
        let r = ns(REFRESH, &refresh);
        self.db.set(&r[..], &pid.0, self.refresh_ttl).await?;
        Ok(Reply::AuthSessionStart { access, refresh })
    }
    async fn handle_session_refresh(&self, refresh: &Id) -> Result<Reply, Error> {
        let r = ns(REFRESH, refresh);
        self.db.getex(&r[..], self.refresh_ttl).await?;
        let access = {
            let mut rng = rand::thread_rng();
            Id::rand(&mut rng)?
        };
        Ok(Reply::AuthSessionRefresh { access })
    }
}

fn ns(n: u8, id: &Id) -> Bytes {
    ([n][..]).chain(&id.0[..]).copy_to_bytes(1 + IDL)
}
