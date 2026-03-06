use super::Database;
use crate::ir::Id;
use crate::{Error, Result};
use sqlx::Row;

impl Database {
    /// Get user's credit balance from CockroachDB.
    pub async fn get_credit(&self, uid: &Id) -> Result<i64> {
        let result = sqlx::query("SELECT credit FROM user_accounts WHERE uid = $1")
            .bind(&uid.0[..])
            .fetch_optional(&self.crdb)
            .await?;

        Ok(result.map_or(0, |row| row.get("credit")))
    }

    /// Create a new user account in CockroachDB.
    pub async fn create_user_account(&self, uid: &Id) -> Result<()> {
        sqlx::query(
            "INSERT INTO user_accounts (uid, credit) VALUES ($1, 0) 
             ON CONFLICT (uid) DO NOTHING",
        )
        .bind(&uid.0[..])
        .execute(&self.crdb)
        .await?;
        Ok(())
    }

    /// Increment user's credit.
    pub async fn incr_credit(
        &self,
        uid: &Id,
        other: Option<&Id>,
        n: i64,
        note: &str,
    ) -> Result<()> {
        if n < 0 {
            return Err(Error::NumCheck);
        }

        // Update credit balance
        sqlx::query(
            "INSERT INTO user_accounts (uid, credit) VALUES ($1, $2)
             ON CONFLICT (uid) DO UPDATE SET credit = user_accounts.credit + $2, updated_at = now()",
        )
        .bind(&uid.0[..])
        .bind(n)
        .execute(&self.crdb)
        .await?;

        // Log the transaction
        self.log_credit_transaction(uid, other, n, note).await;

        Ok(())
    }

    /// Decrement user's credit.
    pub async fn decr_credit(
        &self,
        uid: &Id,
        other: Option<&Id>,
        n: i64,
        note: &str,
    ) -> Result<()> {
        if n < 0 {
            return Err(Error::NumCheck);
        }

        // Check balance first
        let current_balance = self.get_credit(uid).await?;
        if n > current_balance - self.credit_limit {
            return Err(Error::CostInsufficientCredit);
        }

        // Update credit balance
        sqlx::query(
            "UPDATE user_accounts SET credit = credit - $1, updated_at = now() WHERE uid = $2",
        )
        .bind(n)
        .bind(&uid.0[..])
        .execute(&self.crdb)
        .await?;

        // Log the transaction
        self.log_credit_transaction(uid, other, -n, note).await;

        Ok(())
    }

    /// Log credit transaction (non-blocking).
    async fn log_credit_transaction(&self, uid: &Id, other: Option<&Id>, amount: i64, note: &str) {
        let crdb = self.crdb.clone();
        let uid = uid.0.to_vec();
        let other = other.map(|id| id.0.to_vec());
        let note = note.to_string();

        tokio::spawn(async move {
            let _ = sqlx::query(
                "INSERT INTO credit_log (uid, other_uid, amount, note) VALUES ($1, $2, $3, $4)",
            )
            .bind(&uid)
            .bind(&other)
            .bind(amount)
            .bind(&note)
            .execute(&crdb)
            .await;
        });
    }
}
