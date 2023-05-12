use crate::config::Config;
use crate::config::PHONE_MAX_BYTES;
use crate::cost::Cost;
use crate::database::namespace::ACCESS;
use crate::database::namespace::REFRESH;
use crate::database::namespace::SMSSENDTO;
use crate::database::{ns, Database};
use crate::message::{Error, Id, Query, Reply, IDL};
use bytes::{BufMut, Bytes, BytesMut};
use std::sync::Arc;

pub struct Auth {
    cost: Cost,
    db: &'static Database,
    access_ttl: usize,
    refresh_ttl: usize,
    phones: Arc<Vec<String>>,
}

impl Auth {
    pub fn new(config: &Config, db: &'static Database, cost: Cost) -> Auth {
        Auth {
            cost,
            db,
            access_ttl: config.access_ttl,
            refresh_ttl: config.refresh_ttl,
            phones: Arc::clone(&config.auth_phones),
        }
    }
    pub async fn handle(&self, query: &Query) -> Reply {
        let result = match query {
            // Session management
            Query::AuthSessionStart => self.handle_session_start().await,
            Query::AuthSessionRefresh { refresh } => self.handle_session_refresh(refresh).await,
            Query::AuthSessionEnd {
                access,
                option_refresh,
            } => self.handle_session_end(access, option_refresh).await,
            Query::AuthSmsSendTo { access } => self.handle_sms_send_to(access).await,
            Query::AuthSmsSent { access, refresh } => self.handle_sms_sent(access, refresh).await,
            // Authenticate and pass to next layer
            q => {
                let access = q.get_access();
                let uid = match self.authenticate(access).await {
                    Ok(u) => u,
                    Err(error) => return Reply::AuthError { error },
                };
                if uid.is_zero() {
                    return Reply::AuthError {
                        error: Error::NotFound,
                    };
                }
                Ok(self.cost.handle(&uid, q))
            }
        };
        match result {
            Ok(r) => r,
            Err(error) => Reply::AuthError { error },
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
    /// Query UID from access token, zero is anonymous.
    async fn authenticate(&self, access: &Id) -> Result<Id, Error> {
        let a = ns(ACCESS, access);
        match self.db.get::<_, Option<Vec<u8>>>(&a[..]).await? {
            Some(uid) => Ok(Id::try_from(uid)?),
            None => Err(Error::NotFound),
        }
    }
    /// Send what to who to authenticate
    async fn handle_sms_send_to(&self, access: &Id) -> Result<Reply, Error> {
        self.authenticate(access).await?;
        let (phone, message) = {
            let mut rng = rand::thread_rng();
            use rand::seq::SliceRandom;
            (
                (*self.phones)[..].choose(&mut rng).unwrap(),
                Id::rand(&mut rng)?,
            )
        };
        let key = nspm(SMSSENDTO, phone, &message);
        self.db
            .set(&key[..], &access.0[..], self.access_ttl)
            .await?;
        Ok(Reply::AuthSmsSendTo {
            phone: phone.clone(), //TODO: use index instead
            message,
        })
    }
    /// If sent, set tokens' value to uid
    async fn handle_sms_sent(&self, access: &Id, refresh: &Id) -> Result<Reply, Error> {
        // find user's phone in SMSSENT,phone,message
        // if not found return error
        // find user's uid by phone in UIDPHONE
        // if no uid found, use a free one in UIDPHONE & PHONEUID
        // return that uid
        Ok(Reply::Unimplemented)
    }
}

/// Build namespaced key from phone and message
pub fn nspm(n: u8, phone: &String, message: &Id) -> Bytes {
    let mut buf = BytesMut::with_capacity(1 + PHONE_MAX_BYTES + IDL);
    buf.put(&[n][..]);
    buf.put(phone.as_bytes());
    buf.put(&message.0[..]);
    buf.into()
}
