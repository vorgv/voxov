use super::Database;
use crate::database::namespace::UID2CREDIT;
use crate::database::ns;
use crate::ir::Id;
use crate::{Error, Result};
use bson::doc;

impl Database {
    /// Non-blocking credit logging to MongoDB.
    fn spawn_log(&self, uid: &Id, n: i64, note: &str) {
        let cl = self.cl.clone();
        let uid = uid.to_string();
        let note = note.to_string();
        tokio::spawn(async move {
            let _ = cl
                .insert_one(
                    doc! {
                        "uid": uid,
                        "n": n,
                        "note": note,
                    },
                    None,
                )
                .await;
        });
    }

    pub async fn incr_credit(&self, uid: &Id, n: i64, note: &str) -> Result<()> {
        if n < 0 {
            return Err(Error::NumCheck);
        }

        // Log after credit gain.
        let u2c = ns(UID2CREDIT, uid);
        self.incrby(&u2c[..], n).await?;
        self.spawn_log(uid, n, note);

        Ok(())
    }

    pub async fn decr_credit(&self, uid: &Id, n: i64, note: &str) -> Result<()> {
        if n < 0 {
            return Err(Error::NumCheck);
        }

        // Log before credit loss.
        self.spawn_log(uid, -n, note);
        let u2p = ns(UID2CREDIT, uid);
        let credit = self.get::<&[u8], i64>(&u2p).await?;

        if n > credit - self.credit_limit {
            return Err(Error::CostInsufficientCredit);
        } else {
            let u2c = ns(UID2CREDIT, uid);
            self.decrby(&u2c[..], n).await?;
        }

        let u2c = ns(UID2CREDIT, uid);
        self.decrby(&u2c[..], n).await?;

        Ok(())
    }
}
