pub mod completions;
pub mod executors;
pub mod groups;
pub mod projects;
pub mod tasks;

use std::str::FromStr;
use std::time::Duration;

use anyhow::Context;
use jiff::Timestamp;
use sqlx::SqlitePool;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous};

pub async fn connect(database_url: &str) -> anyhow::Result<SqlitePool> {
    let options = SqliteConnectOptions::from_str(database_url)
        .with_context(|| format!("invalid SYSSLOR_DATABASE_URL {database_url:?}"))?
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

fn to_millis(timestamp: Timestamp) -> i64 {
    timestamp.as_millisecond()
}

fn from_millis(millis: i64) -> anyhow::Result<Timestamp> {
    Timestamp::from_millisecond(millis)
        .with_context(|| format!("timestamp {millis} ms is out of range"))
}
