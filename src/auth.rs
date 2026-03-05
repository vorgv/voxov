//! Authentication and session management.

use crate::config::Config;
use crate::cost::Cost;
use crate::database::Database;
use crate::ir::{Id, Query, Reply};
use crate::{Error, Result};
use bytes::{BufMut, Bytes, BytesMut};

pub struct Auth {
    cost: &'static Cost,
    db: &'static Database,
    skip_auth: bool,
    phones: &'static Vec<String>,
}

impl Auth {
    pub fn new(config: &Config, db: &'static Database, cost: &'static Cost) -> Auth {
        Auth {
            cost,
            db,
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

        // Store tokens in ScyllaDB
        self.db.set_access(&access.0, &uid).await?;
        self.db.set_refresh(&refresh.0, &uid).await?;

        Ok(Reply::AuthSessionStart { access, refresh })
    }

    /// If refresh exists, reset its TTL, then generate a new access.
    async fn handle_session_refresh(&self, refresh: &Id) -> Result<Reply> {
        let uid = self
            .db
            .get_refresh_and_extend(&refresh.0)
            .await?
            .ok_or(Error::AuthInvalidRefreshToken)?;

        let access = {
            let mut rng = rand::thread_rng();
            Id::rand(&mut rng)?
        };

        self.db.set_access(&access.0, &uid).await?;

        Ok(Reply::AuthSessionRefresh { access })
    }

    /// If access is valid, delete access and optionally refresh.
    async fn handle_session_end(&self, access: &Id, option_refresh: &Option<Id>) -> Result<Reply> {
        let access_uid = self.authenticate(access).await?;

        self.db.del_session(&access.0).await?;

        if let Some(refresh) = option_refresh {
            // Check if uid matches
            if let Some(refresh_uid) = self.db.get_refresh_and_extend(&refresh.0).await? {
                if refresh_uid != access_uid {
                    return Err(Error::AuthTokensMismatch);
                }
            } else {
                return Err(Error::AuthInvalidRefreshToken);
            }
            self.db.del_session(&refresh.0).await?;
        }

        Ok(Reply::AuthSessionEnd)
    }

    /// Query UID from access token, zero is anonymous.
    async fn authenticate(&self, access: &Id) -> Result<Id> {
        self.db
            .get_access(&access.0)
            .await?
            .ok_or(Error::AuthInvalidAccessToken)
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

        self.db.set_sms_sendto(phone, &message.0).await?;

        Ok(Reply::AuthSmsSendTo { phone, message })
    }

    /// If sent, set tokens' value to uid.
    async fn handle_sms_sent(
        &self,
        access: &Id,
        refresh: &Id,
        phone: &str,
        message: &Id,
    ) -> Result<Reply> {
        self.authenticate(access).await?;
        let db = self.db;

        // Find user's phone from SMS sent records
        let user_phone = match self.skip_auth {
            true => phone.to_owned(),
            false => db
                .get_sms_sent(phone, &message.0)
                .await?
                .ok_or(Error::AuthInvalidPhone)?,
        };

        // Find user's uid by phone
        let mut is_new_user = false;
        let uid = match db.get_phone_to_uid(&user_phone).await? {
            Some(uid) => uid,
            None => {
                is_new_user = true;
                let mut rng = rand::thread_rng();
                Id::rand(&mut rng)?
            }
        };

        // Create or refresh UID <-> Phone mappings
        db.set_uid_to_phone(&uid, &user_phone).await?;
        db.set_phone_to_uid(&user_phone, &uid).await?;

        // Create user account in TigerBeetle if new
        if is_new_user {
            db.create_user_account(&uid).await?;
        }

        // Set uid of auth tokens
        db.set_access(&access.0, &uid).await?;
        db.set_refresh(&refresh.0, &uid).await?;

        Ok(Reply::AuthSmsSent { uid })
    }
}

/// Build namespaced key from phone and message (kept for compatibility).
pub fn nspm(n: u8, phone: &str, message: &Id) -> Bytes {
    use crate::config::PHONE_MAX_BYTES;
    use crate::ir::IDL;

    let mut buf = BytesMut::with_capacity(1 + PHONE_MAX_BYTES + IDL);
    buf.put(&[n][..]);
    buf.put(phone.as_bytes());
    buf.put(&message.0[..]);
    buf.into()
}

/// Namespacing phone (kept for compatibility).
pub fn nsp(n: u8, phone: &String) -> Bytes {
    use crate::config::PHONE_MAX_BYTES;

    let mut buf = BytesMut::with_capacity(1 + PHONE_MAX_BYTES);
    buf.put(&[n][..]);
    buf.put(phone.as_bytes());
    buf.into()
}
