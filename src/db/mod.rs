use std::str::FromStr;
use std::time::Duration;

use anyhow::Context;
use sqlx::SqlitePool;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous};

pub async fn connect(database_url: &str) -> anyhow::Result<SqlitePool> {
    let options = SqliteConnectOptions::from_str(database_url)
        .with_context(|| format!("invalid DATABASE_URL {database_url:?}"))?
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(SqliteSynchronous::Normal)
        .foreign_keys(true)
        .busy_timeout(Duration::from_secs(5))
        // The scratch image has no /tmp, so keep temporary tables in memory.
        .pragma("temp_store", "memory");

    let pool = SqlitePoolOptions::new()
        .connect_with(options)
        .await
        .context("failed to open database")?;

    sqlx::migrate!()
        .run(&pool)
        .await
        .context("failed to run migrations")?;

    Ok(pool)
}
