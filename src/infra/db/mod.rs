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
            .synchronous(SqliteSynchronous::Normal);

        let pool = SqlitePoolOptions::new()
            .connect_with(connect_options)
            .await?;

        sqlx::migrate!().run(&pool).await?;

        Ok(Self { pool })
    }

    pub async fn close(&self) {
        self.pool.close().await;
    }
}
