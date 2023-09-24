//! Authentication and session management.

use crate::config::Config;
use crate::config::PHONE_MAX_BYTES;
use crate::cost::Cost;
use crate::database::namespace::ACCESS;
use crate::database::namespace::PHONE2UID;
use crate::database::namespace::REFRESH;
use crate::database::namespace::SMSSENDTO;
use crate::database::namespace::SMSSENT;
use crate::database::namespace::UID2CREDIT;
use crate::database::namespace::UID2PHONE;
use crate::database::{ns, Database};
use crate::ir::{Id, Query, Reply, IDL};
use crate::{Error, Result};
use bytes::{BufMut, Bytes, BytesMut};

pub struct Auth {
    cost: &'static Cost,
    db: &'static Database,
    access_ttl: i64,
    refresh_ttl: i64,
    user_ttl: i64,
    skip_auth: bool,
    phones: &'static Vec<String>,
}

impl Auth {
    pub fn new(config: &Config, db: &'static Database, cost: &'static Cost) -> Auth {
        Auth {
            cost,
            db,
            access_ttl: config.access_ttl,
            refresh_ttl: config.refresh_ttl,
            user_ttl: config.user_ttl,
            skip_auth: config.skip_auth,
            phones: config.auth_phones,
        }
    }

    pub async fn handle(&self, query: Query) -> Result<Reply> {
        match query {
            // Session management
            Query::AuthSessionStart => self.handle_session_start().await,
            Query::AuthSessionRefresh { refresh } => self.handle_session_refresh(&refresh).await,
            Query::AuthSessionEnd {
                access,
                option_refresh,
            } => self.handle_session_end(&access, &option_refresh).await,
            Query::AuthSmsSendTo { access } => self.handle_sms_send_to(&access).await,
            Query::AuthSmsSent {
                access,
                refresh,
                phone,
                message,
            } => {
                self.handle_sms_sent(&access, &refresh, &phone, &message)
                    .await
            }

            // Authenticate and pass to next layer
            q => {
                let access = q.get_access();
                let uid = self.authenticate(access).await?;
                if uid.is_zero() {
                    return Err(Error::AuthNotAuthenticated);
                }
                Ok(self.cost.handle(q, &uid).await?)
            }
        }
    }

    /// Generate two random tokens.
    async fn handle_session_start(&self) -> Result<Reply> {
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

    /// If refresh exists, reset its TTL, then gengerate a new access.
    async fn handle_session_refresh(&self, refresh: &Id) -> Result<Reply> {
        let r = ns(REFRESH, refresh);
        let uid: Vec<u8> = match self.db.getex(&r[..], self.refresh_ttl).await? {
            Some(v) => v,
            None => return Err(Error::AuthInvalidRefreshToken),
        };
        let access = {
            let mut rng = rand::thread_rng();
            Id::rand(&mut rng)?
        };
        let a = ns(ACCESS, &access);
        self.db.set(&a[..], &uid, self.access_ttl).await?;
        Ok(Reply::AuthSessionRefresh { access })
    }

    /// If access is valid, delete access and optionally refresh.
    async fn handle_session_end(&self, access: &Id, option_refresh: &Option<Id>) -> Result<Reply> {
        let access_uid = self.authenticate(access).await?;
        let a = ns(ACCESS, access);
        self.db.del(&a[..]).await?;
        if let Some(refresh) = option_refresh {
            // Check if uid matches
            let r = ns(REFRESH, refresh);
            if let Some(refresh_uid) = self.db.get::<_, Option<Vec<u8>>>(&r[..]).await? {
                if Id::try_from(refresh_uid)? != access_uid {
                    return Err(Error::AuthTokensMismatch);
                }
            } else {
                return Err(Error::AuthInvalidRefreshToken);
            }
            self.db.del(&r[..]).await?;
        }
        Ok(Reply::AuthSessionEnd)
    }

    /// Query UID from access token, zero is anonymous.
    async fn authenticate(&self, access: &Id) -> Result<Id> {
        let a = ns(ACCESS, access);
        match self.db.get::<_, Option<Vec<u8>>>(&a[..]).await? {
            Some(uid) => Ok(Id::try_from(uid)?),
            None => Err(Error::AuthInvalidAccessToken),
        }
    }

    /// Send what to who to authenticate.
    async fn handle_sms_send_to(&self, access: &Id) -> Result<Reply> {
        self.authenticate(access).await?;
        let (phone, message): (&'static _, _) = {
            let mut rng = rand::thread_rng();
            use rand::seq::SliceRandom;
            (
                (*self.phones)[..].choose(&mut rng).unwrap(),
                Id::rand(&mut rng)?,
            )
        };
        let key = nspm(SMSSENDTO, phone, &message);
        self.db.set(&key[..], &access.0, self.access_ttl).await?;
        Ok(Reply::AuthSmsSendTo { phone, message })
    }

    /// If sent, set tokens' value to uid.
    async fn handle_sms_sent(
        &self,
        access: &Id,
        refresh: &Id,
        phone: &String,
        message: &Id,
    ) -> Result<Reply> {
        self.authenticate(access).await?;
        let db = self.db;

        // Find user's phone in SMSSENT, phone, message.
        let user_phone = match self.skip_auth {
            true => phone.clone(),
            false => {
                let key = nspm(SMSSENT, phone, message);
                db.get::<&[u8], Option<String>>(&key[..])
                    .await?
                    .ok_or(Error::AuthInvalidPhone)?
            }
        };

        // Find user's uid by phone in PHONE2UID.
        let p2u = nsp(PHONE2UID, &user_phone);
        let mut is_new_user = false;
        let uid = match db.get::<&[u8], Option<Vec<u8>>>(&p2u[..]).await? {
            Some(uid) => Id::try_from(uid)?,
            None => {
                is_new_user = true;
                let mut rng = rand::thread_rng();
                Id::rand(&mut rng)?
            }
        };

        // Create one in or refresh UID2PHONE & PHONE2UID.
        let u2p = ns(UID2PHONE, &uid);
        db.set(&u2p[..], user_phone, self.user_ttl).await?;
        db.set(&p2u[..], &uid.0, self.user_ttl).await?;

        // Create user account.
        let u2c = ns(UID2CREDIT, &uid);
        if is_new_user {
            db.set(&u2c[..], 0, self.user_ttl).await?;
        } else {
            db.expire(&u2c[..], self.user_ttl).await?;
        }

        // Set uid of auth tokens.
        let a = ns(ACCESS, access);
        let r = ns(REFRESH, refresh);
        db.set(&a[..], &uid.0, self.access_ttl).await?;
        db.set(&r[..], &uid.0, self.refresh_ttl).await?;

        Ok(Reply::AuthSmsSent { uid })
    }
}

/// Build namespaced key from phone and message.
pub fn nspm(n: u8, phone: &String, message: &Id) -> Bytes {
    let mut buf = BytesMut::with_capacity(1 + PHONE_MAX_BYTES + IDL);
    buf.put(&[n][..]);
    buf.put(phone.as_bytes());
    buf.put(&message.0[..]);
    buf.into()
}

/// Namespacing phone.
pub fn nsp(n: u8, phone: &String) -> Bytes {
    let mut buf = BytesMut::with_capacity(1 + PHONE_MAX_BYTES);
    buf.put(&[n][..]);
    buf.put(phone.as_bytes());
    buf.into()
}
