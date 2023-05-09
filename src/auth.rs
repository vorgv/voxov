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
            Query::AuthSessionEnd {
                access,
                option_refresh,
            } => match self.handle_session_end(access, option_refresh).await {
                Ok(r) => r,
                Err(e) => Reply::AuthError { error: e },
            },
            Query::AuthSmsSendTo { access: _ } => Reply::Unimplemented,
            Query::AuthSmsSent { access: _ } => Reply::Unimplemented,
            // Authenticate and pass to next layer
            q => {
                let access = q.get_access();
                let uid = match self.authenticate(access).await {
                    Ok(u) => u,
                    Err(e) => return Reply::AuthError { error: e },
                };
                self.cost.handle(&uid, q)
            }
        }
    }
    /// Generate two random tokens
    async fn handle_session_start(&self) -> Result<Reply, Error> {
        let (access, refresh) = {
            let mut rng = rand::thread_rng();
            (Id::rand(&mut rng)?, Id::rand(&mut rng)?)
        }; // drop rng before await
        let uid = Id::zero();
        let a = ns(ACCESS, &access);
        self.db.set(&a[..], &uid.0, self.access_ttl).await?;
        let r = ns(REFRESH, &refresh);
        self.db.set(&r[..], &uid.0, self.refresh_ttl).await?;
        Ok(Reply::AuthSessionStart { access, refresh })
    }
    /// If refresh exists, reset its TTL, then gengerate a new access
    async fn handle_session_refresh(&self, refresh: &Id) -> Result<Reply, Error> {
        let r = ns(REFRESH, refresh);
        let uid: Option<Vec<u8>> = match self.db.getex(&r[..], self.refresh_ttl).await? {
            Some(v) => v,
            None => return Err(Error::NotFound),
        };
        let access = {
            let mut rng = rand::thread_rng();
            Id::rand(&mut rng)?
        };
        let a = ns(ACCESS, &access);
        self.db.set(&a[..], &uid, self.access_ttl).await?;
        Ok(Reply::AuthSessionRefresh { access })
    }
    /// If access is valid, delete access and optionally refresh
    async fn handle_session_end(
        &self,
        access: &Id,
        option_refresh: &Option<Id>,
    ) -> Result<Reply, Error> {
        let access_uid = self.authenticate(access).await?;
        let a = ns(ACCESS, access);
        self.db.del(&a[..]).await?;
        if let Some(refresh) = option_refresh {
            // Check if uid matches
            let r = ns(REFRESH, refresh);
            if let Some(r_uid) = self.db.get::<_, Option<Vec<u8>>>(&r[..]).await? {
                if Id::try_from(r_uid)? != access_uid {
                    return Err(Error::Auth);
                }
            } else {
                return Err(Error::Auth);
            }
            self.db.del(&r[..]).await?;
        }
        Ok(Reply::AuthSessionEnd)
    }
    /// Query UID from access token
    async fn authenticate(&self, access: &Id) -> Result<Id, Error> {
        let a = ns(ACCESS, access);
        match self.db.get::<_, Option<Vec<u8>>>(&a[..]).await? {
            Some(uid) => match Id::try_from(uid)? {
                x if x.is_zero() => Err(Error::Auth),
                x => Ok(x),
            },
            None => Err(Error::NotFound),
        }
    }
}

/// Prepend namespace tag before Id
fn ns(n: u8, id: &Id) -> Bytes {
    ([n][..]).chain(&id.0[..]).copy_to_bytes(1 + IDL)
}
