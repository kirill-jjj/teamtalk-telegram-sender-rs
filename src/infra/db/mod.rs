pub mod admins;
pub mod bans;
pub mod deeplinks;
pub mod mutes;
pub mod pending_channel_replies;
pub mod pending_replies;
pub mod subscriptions;
pub mod types;
pub mod user_settings;

use anyhow::Result;
use sqlx::{
    Pool, Sqlite,
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous},
};
use std::time::Duration;

#[derive(Clone)]
pub struct Database {
    pub pool: Pool<Sqlite>,
}

impl Database {
    pub async fn new(db_file: &str) -> Result<Self> {
        let connect_options = SqliteConnectOptions::new()
            .filename(db_file)
            .create_if_missing(true)
            .journal_mode(SqliteJournalMode::Wal)
            .synchronous(SqliteSynchronous::Normal)
            .busy_timeout(Duration::from_secs(30));

        let pool = SqlitePoolOptions::new()
            .max_connections(20)
            .min_connections(2)
            .acquire_timeout(Duration::from_secs(30))
            .connect_with(connect_options)
            .await?;

        sqlx::migrate!().run(&pool).await?;

        Ok(Self { pool })
    }

    pub async fn close(&self) {
        self.pool.close().await;
    }
}
