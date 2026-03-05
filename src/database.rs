mod credit;
pub mod ripperd;

use crate::config::Config;
use crate::ir::id::Id;
use crate::Result;
use chrono::{DateTime, Utc};
use s3::creds::Credentials;
use s3::region::Region;
use s3::Bucket;
use scylla::prepared_statement::PreparedStatement;
use scylla::transport::session::Session;
use scylla::SessionBuilder;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration as StdDuration;
use sysinfo::{DiskExt, System, SystemExt};

/// Prepared statements for ScyllaDB operations.
pub struct ScyllaPreparedStatements {
    // Sessions
    pub insert_session: PreparedStatement,
    pub select_session: PreparedStatement,
    pub delete_session: PreparedStatement,
    // SMS codes
    pub insert_sms_sendto: PreparedStatement,
    pub insert_sms_sent: PreparedStatement,
    pub select_sms_sent: PreparedStatement,
    // User identity
    pub insert_phone_to_uid: PreparedStatement,
    pub select_phone_to_uid: PreparedStatement,
    pub insert_uid_to_phone: PreparedStatement,
    pub select_uid_to_phone: PreparedStatement,
    // Check-ins
    pub insert_checkin: PreparedStatement,
    pub select_checkin: PreparedStatement,
}

pub struct Database {
    /// ScyllaDB session
    pub scylla: Arc<Session>,

    /// CockroachDB pool
    pub crdb: PgPool,

    /// S3 bucket for meme data
    pub mr: Bucket,

    /// Prepared statements
    pub stmts: ScyllaPreparedStatements,

    pub credit_limit: i64,
    pub access_ttl: i64,
    pub refresh_ttl: i64,
    pub user_ttl: i64,
}

impl Database {
    /// Connect to all databases, panic on failure.
    pub async fn new(config: &Config, create_schema: bool) -> Database {
        // Connect to CockroachDB
        let crdb = PgPoolOptions::new()
            .max_connections(10)
            .connect(&config.crdb_addr)
            .await
            .expect("CockroachDB offline?");

        // Connect to ScyllaDB
        let scylla = SessionBuilder::new()
            .known_node(&config.scylla_addr)
            .build()
            .await
            .expect("ScyllaDB offline?");
        let scylla = Arc::new(scylla);

        // Create S3 bucket handle
        let mr = Bucket::new(
            "voxov",
            Region::Custom {
                region: config.s3_region.clone(),
                endpoint: config.s3_addr.clone(),
            },
            Credentials::new(
                Some(&config.s3_access_key),
                Some(&config.s3_secret_key),
                None,
                None,
                None,
            )
            .unwrap(),
        )
        .expect("S3 offline?")
        .with_path_style();

        // Create schemas if requested
        if create_schema {
            Self::create_scylla_schema(&scylla).await;
            Self::create_crdb_schema(&crdb).await;
        }

        // Prepare ScyllaDB statements
        let stmts = Self::prepare_statements(&scylla).await;

        let db = Database {
            scylla,
            crdb,
            mr,
            stmts,
            credit_limit: config.credit_limit,
            access_ttl: config.access_ttl,
            refresh_ttl: config.refresh_ttl,
            user_ttl: config.user_ttl,
        };

        if config.samsara {
            db.samsara().await;
        }

        db
    }

    /// Database with default config and no samsara.
    pub async fn default() -> Database {
        let config = Config::new();
        Database::new(&config, false).await
    }

    /// Create ScyllaDB keyspace and tables.
    async fn create_scylla_schema(scylla: &Session) {
        // Create keyspace
        scylla
            .query(
                "CREATE KEYSPACE IF NOT EXISTS voxov WITH replication = {'class': 'SimpleStrategy', 'replication_factor': 1}",
                &[],
            )
            .await
            .expect("Failed to create keyspace");

        // Sessions table
        scylla
            .query(
                "CREATE TABLE IF NOT EXISTS voxov.sessions (
                    sid BLOB PRIMARY KEY,
                    uid BLOB,
                    kind TINYINT
                )",
                &[],
            )
            .await
            .expect("Failed to create sessions table");

        // SMS codes table (tracks SMS verification)
        scylla
            .query(
                "CREATE TABLE IF NOT EXISTS voxov.sms_codes (
                    phone TEXT,
                    message BLOB,
                    user_phone TEXT,
                    PRIMARY KEY (phone, message)
                )",
                &[],
            )
            .await
            .expect("Failed to create sms_codes table");

        // Phone to UID mapping
        scylla
            .query(
                "CREATE TABLE IF NOT EXISTS voxov.phone_to_uid (
                    phone TEXT PRIMARY KEY,
                    uid BLOB
                )",
                &[],
            )
            .await
            .expect("Failed to create phone_to_uid table");

        // UID to phone mapping
        scylla
            .query(
                "CREATE TABLE IF NOT EXISTS voxov.uid_to_phone (
                    uid BLOB PRIMARY KEY,
                    phone TEXT
                )",
                &[],
            )
            .await
            .expect("Failed to create uid_to_phone table");

        // Check-ins table
        scylla
            .query(
                "CREATE TABLE IF NOT EXISTS voxov.checkins (
                    uid BLOB PRIMARY KEY,
                    last_checkin TIMESTAMP
                )",
                &[],
            )
            .await
            .expect("Failed to create checkins table");
    }

    /// Create CockroachDB tables.
    async fn create_crdb_schema(crdb: &PgPool) {
        // User accounts table (for credits)
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS user_accounts (
                uid BYTEA PRIMARY KEY,
                credit BIGINT NOT NULL DEFAULT 0,
                created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
            )",
        )
        .execute(crdb)
        .await
        .expect("Failed to create user_accounts table");

        // Credit transactions log
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS credit_log (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                uid BYTEA NOT NULL,
                other_uid BYTEA,
                amount BIGINT NOT NULL,
                note TEXT NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT now()
            )",
        )
        .execute(crdb)
        .await
        .expect("Failed to create credit_log table");

        sqlx::query("CREATE INDEX IF NOT EXISTS credit_log_uid_idx ON credit_log (uid)")
            .execute(crdb)
            .await
            .ok();

        sqlx::query("CREATE INDEX IF NOT EXISTS credit_log_created_idx ON credit_log (created_at)")
            .execute(crdb)
            .await
            .ok();

        // Meme metadata table
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS meme_meta (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                uid BYTEA NOT NULL,
                oid BYTEA NOT NULL UNIQUE,
                hash BYTEA NOT NULL,
                size BIGINT NOT NULL,
                pub BOOLEAN NOT NULL DEFAULT FALSE,
                tip BIGINT NOT NULL DEFAULT 0,
                eol TIMESTAMPTZ NOT NULL
            )",
        )
        .execute(crdb)
        .await
        .expect("Failed to create meme_meta table");

        sqlx::query("CREATE INDEX IF NOT EXISTS meme_meta_eol_idx ON meme_meta (eol)")
            .execute(crdb)
            .await
            .ok();

        sqlx::query("CREATE INDEX IF NOT EXISTS meme_meta_hash_idx ON meme_meta (hash)")
            .execute(crdb)
            .await
            .ok();

        // Map documents table
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS map_docs (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                uid BYTEA NOT NULL,
                pub BOOLEAN NOT NULL DEFAULT FALSE,
                eol TIMESTAMPTZ NOT NULL,
                tip BIGINT NOT NULL DEFAULT 0,
                ns TEXT NOT NULL DEFAULT '',
                size BIGINT NOT NULL DEFAULT 0,
                i0 JSONB, i1 JSONB, i2 JSONB, i3 JSONB,
                i4 JSONB, i5 JSONB, i6 JSONB, i7 JSONB,
                geo_lon DOUBLE PRECISION,
                geo_lat DOUBLE PRECISION,
                body JSONB
            )",
        )
        .execute(crdb)
        .await
        .expect("Failed to create map_docs table");

        sqlx::query("CREATE INDEX IF NOT EXISTS map_docs_eol_idx ON map_docs (eol)")
            .execute(crdb)
            .await
            .ok();

        sqlx::query("CREATE INDEX IF NOT EXISTS map_docs_uid_idx ON map_docs (uid)")
            .execute(crdb)
            .await
            .ok();

        sqlx::query("CREATE INDEX IF NOT EXISTS map_docs_ns_idx ON map_docs (ns)")
            .execute(crdb)
            .await
            .ok();
    }

    /// Prepare ScyllaDB statements for better performance.
    async fn prepare_statements(scylla: &Session) -> ScyllaPreparedStatements {
        ScyllaPreparedStatements {
            insert_session: scylla
                .prepare("INSERT INTO voxov.sessions (sid, uid, kind) VALUES (?, ?, ?) USING TTL ?")
                .await
                .expect("Failed to prepare insert_session"),

            select_session: scylla
                .prepare("SELECT uid, kind FROM voxov.sessions WHERE sid = ?")
                .await
                .expect("Failed to prepare select_session"),

            delete_session: scylla
                .prepare("DELETE FROM voxov.sessions WHERE sid = ?")
                .await
                .expect("Failed to prepare delete_session"),

            insert_sms_sendto: scylla
                .prepare("INSERT INTO voxov.sms_codes (phone, message) VALUES (?, ?) USING TTL ?")
                .await
                .expect("Failed to prepare insert_sms_sendto"),

            insert_sms_sent: scylla
                .prepare("UPDATE voxov.sms_codes USING TTL ? SET user_phone = ? WHERE phone = ? AND message = ?")
                .await
                .expect("Failed to prepare insert_sms_sent"),

            select_sms_sent: scylla
                .prepare("SELECT user_phone FROM voxov.sms_codes WHERE phone = ? AND message = ?")
                .await
                .expect("Failed to prepare select_sms_sent"),

            insert_phone_to_uid: scylla
                .prepare("INSERT INTO voxov.phone_to_uid (phone, uid) VALUES (?, ?) USING TTL ?")
                .await
                .expect("Failed to prepare insert_phone_to_uid"),

            select_phone_to_uid: scylla
                .prepare("SELECT uid FROM voxov.phone_to_uid WHERE phone = ?")
                .await
                .expect("Failed to prepare select_phone_to_uid"),

            insert_uid_to_phone: scylla
                .prepare("INSERT INTO voxov.uid_to_phone (uid, phone) VALUES (?, ?) USING TTL ?")
                .await
                .expect("Failed to prepare insert_uid_to_phone"),

            select_uid_to_phone: scylla
                .prepare("SELECT phone FROM voxov.uid_to_phone WHERE uid = ?")
                .await
                .expect("Failed to prepare select_uid_to_phone"),

            insert_checkin: scylla
                .prepare("INSERT INTO voxov.checkins (uid, last_checkin) VALUES (?, ?)")
                .await
                .expect("Failed to prepare insert_checkin"),

            select_checkin: scylla
                .prepare("SELECT last_checkin FROM voxov.checkins WHERE uid = ?")
                .await
                .expect("Failed to prepare select_checkin"),
        }
    }

    // --- Session operations (ScyllaDB) ---

    /// Insert access token.
    pub async fn set_access(&self, token: &[u8], uid: &Id) -> Result<()> {
        self.scylla
            .execute(
                &self.stmts.insert_session,
                (token, &uid.0[..], 0_i8, self.access_ttl as i32),
            )
            .await?;
        Ok(())
    }

    /// Insert refresh token.
    pub async fn set_refresh(&self, token: &[u8], uid: &Id) -> Result<()> {
        self.scylla
            .execute(
                &self.stmts.insert_session,
                (token, &uid.0[..], 1_i8, self.refresh_ttl as i32),
            )
            .await?;
        Ok(())
    }

    /// Get UID from access token.
    pub async fn get_access(&self, token: &[u8]) -> Result<Option<Id>> {
        let result = self
            .scylla
            .execute(&self.stmts.select_session, (token,))
            .await?;

        if let Some(row) = result.rows_typed::<(Vec<u8>, i8)>()?.next() {
            let (uid_bytes, kind) = row?;
            if kind == 0 {
                let mut id = Id::zero();
                id.0.copy_from_slice(&uid_bytes);
                return Ok(Some(id));
            }
        }
        Ok(None)
    }

    /// Get UID from refresh token and refresh its TTL.
    pub async fn get_refresh_and_extend(&self, token: &[u8]) -> Result<Option<Id>> {
        let result = self
            .scylla
            .execute(&self.stmts.select_session, (token,))
            .await?;

        if let Some(row) = result.rows_typed::<(Vec<u8>, i8)>()?.next() {
            let (uid_bytes, kind) = row?;
            if kind == 1 {
                // Refresh TTL by re-inserting
                let mut id = Id::zero();
                id.0.copy_from_slice(&uid_bytes);
                self.set_refresh(token, &id).await?;
                return Ok(Some(id));
            }
        }
        Ok(None)
    }

    /// Delete a session token.
    pub async fn del_session(&self, token: &[u8]) -> Result<()> {
        self.scylla
            .execute(&self.stmts.delete_session, (token,))
            .await?;
        Ok(())
    }

    // --- SMS operations (ScyllaDB) ---

    /// Record that we asked the client to send a message to a phone.
    pub async fn set_sms_sendto(&self, phone: &str, message: &[u8]) -> Result<()> {
        self.scylla
            .execute(
                &self.stmts.insert_sms_sendto,
                (phone, message, self.access_ttl as i32),
            )
            .await?;
        Ok(())
    }

    /// Record that user_phone sent the SMS message to server phone.
    pub async fn sms_sent(&self, user_phone: &str, phone: &str, message: &[u8]) -> Result<()> {
        self.scylla
            .execute(
                &self.stmts.insert_sms_sent,
                (self.access_ttl as i32, user_phone, phone, message),
            )
            .await?;
        Ok(())
    }

    /// Get the user_phone that sent the SMS to server phone with message.
    pub async fn get_sms_sent(&self, phone: &str, message: &[u8]) -> Result<Option<String>> {
        let result = self
            .scylla
            .execute(&self.stmts.select_sms_sent, (phone, message))
            .await?;

        if let Some(row) = result.rows_typed::<(Option<String>,)>()?.next() {
            let (user_phone,) = row?;
            return Ok(user_phone);
        }
        Ok(None)
    }

    // --- User identity operations (ScyllaDB) ---

    /// Set phone to UID mapping.
    pub async fn set_phone_to_uid(&self, phone: &str, uid: &Id) -> Result<()> {
        self.scylla
            .execute(
                &self.stmts.insert_phone_to_uid,
                (phone, &uid.0[..], self.user_ttl as i32),
            )
            .await?;
        Ok(())
    }

    /// Get UID from phone.
    pub async fn get_phone_to_uid(&self, phone: &str) -> Result<Option<Id>> {
        let result = self
            .scylla
            .execute(&self.stmts.select_phone_to_uid, (phone,))
            .await?;

        if let Some(row) = result.rows_typed::<(Vec<u8>,)>()?.next() {
            let (uid_bytes,) = row?;
            let mut id = Id::zero();
            id.0.copy_from_slice(&uid_bytes);
            return Ok(Some(id));
        }
        Ok(None)
    }

    /// Set UID to phone mapping.
    pub async fn set_uid_to_phone(&self, uid: &Id, phone: &str) -> Result<()> {
        self.scylla
            .execute(
                &self.stmts.insert_uid_to_phone,
                (&uid.0[..], phone, self.user_ttl as i32),
            )
            .await?;
        Ok(())
    }

    /// Get phone from UID.
    pub async fn get_uid_to_phone(&self, uid: &Id) -> Result<Option<String>> {
        let result = self
            .scylla
            .execute(&self.stmts.select_uid_to_phone, (&uid.0[..],))
            .await?;

        if let Some(row) = result.rows_typed::<(String,)>()?.next() {
            let (phone,) = row?;
            return Ok(Some(phone));
        }
        Ok(None)
    }

    // --- Check-in operations (ScyllaDB) ---

    /// Get last check-in time.
    pub async fn get_last_checkin(&self, uid: &Id) -> Result<Option<DateTime<Utc>>> {
        let result = self
            .scylla
            .execute(&self.stmts.select_checkin, (&uid.0[..],))
            .await?;

        if let Some(row) = result
            .rows_typed::<(scylla::frame::value::CqlTimestamp,)>()?
            .next()
        {
            let (ts,) = row?;
            // CqlTimestamp is milliseconds since epoch
            let dt = DateTime::from_timestamp_millis(ts.0).unwrap_or_else(Utc::now);
            return Ok(Some(dt));
        }
        Ok(None)
    }

    /// Set check-in time.
    pub async fn set_checkin(&self, uid: &Id) -> Result<()> {
        let now = scylla::frame::value::CqlTimestamp(Utc::now().timestamp_millis());
        self.scylla
            .execute(&self.stmts.insert_checkin, (&uid.0[..], now))
            .await?;
        Ok(())
    }

    // --- User account existence check (CockroachDB) ---

    /// Check if user account exists.
    pub async fn user_exists(&self, uid: &Id) -> Result<bool> {
        let result = sqlx::query("SELECT 1 FROM user_accounts WHERE uid = $1")
            .bind(&uid.0[..])
            .fetch_optional(&self.crdb)
            .await?;
        Ok(result.is_some())
    }

    /// Reset databases on resource draining.
    async fn samsara(&self) {
        let scylla = self.scylla.clone();
        let crdb = self.crdb.clone();
        let mr = self.mr.clone();

        tokio::spawn(async move {
            let mut sys = System::new_all();
            loop {
                tokio::time::sleep(StdDuration::from_secs(60)).await;

                let mut draining = false;
                sys.refresh_all();

                let maybe_disk = sys
                    .disks()
                    .iter()
                    .map(|d| (d.total_space(), d.available_space()))
                    .max();

                if let Some(disk) = maybe_disk {
                    if (disk.1 as f32 / disk.0 as f32) < 0.1 {
                        draining = true;
                    }
                } else {
                    println!("Samsara error: disk not found");
                    break;
                }

                if (sys.used_memory() as f32 / sys.total_memory() as f32) > 0.9 {
                    draining = true;
                }

                if !draining {
                    continue;
                }

                println!("Samsara");

                // Truncate ScyllaDB tables
                for table in [
                    "sessions",
                    "sms_codes",
                    "phone_to_uid",
                    "uid_to_phone",
                    "checkins",
                ] {
                    if let Err(e) = scylla.query(format!("TRUNCATE voxov.{}", table), &[]).await {
                        println!("Samsara ScyllaDB error truncating {}: {}", table, e);
                    }
                }

                // Truncate CockroachDB tables
                for table in ["meme_meta", "map_docs", "user_accounts", "credit_log"] {
                    if let Err(e) = sqlx::query(&format!("TRUNCATE TABLE {}", table))
                        .execute(&crdb)
                        .await
                    {
                        println!("Samsara CockroachDB error truncating {}: {}", table, e);
                    }
                }

                if let Err(error) = mr.delete().await {
                    println!("Samsara S3 error: {}", error);
                }
            }
        });
    }
}

/// Convert Id to u128.
pub fn uid_to_u128(uid: &Id) -> u128 {
    u128::from_be_bytes(uid.0)
}

/// Convert u128 to Id.
pub fn u128_to_uid(v: u128) -> Id {
    Id(v.to_be_bytes())
}
