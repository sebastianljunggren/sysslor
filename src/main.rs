mod api;
mod db;
mod domain;

use std::net::SocketAddr;

use anyhow::{Context, bail};
use axum_extra::extract::cookie::Key;
use jiff::tz::TimeZone;
use tracing_subscriber::EnvFilter;

struct Config {
    database_url: String,
    bind_addr: SocketAddr,
    family_tz: TimeZone,
    admin_password: String,
    cookie_key: Key,
}

impl Config {
    fn from_env() -> anyhow::Result<Self> {
        let database_url = env_or("DATABASE_URL", "sqlite://sysslor.db");
        let bind_addr = env_or("BIND_ADDR", "0.0.0.0:8080")
            .parse()
            .context("BIND_ADDR is not a valid socket address")?;
        let tz_name = env_or("FAMILY_TZ", "Europe/Stockholm");
        let family_tz = TimeZone::get(&tz_name)
            .with_context(|| format!("FAMILY_TZ {tz_name:?} is not a known time zone"))?;
        // No defaults for secrets: a forgotten Secret must not leave the app open.
        let admin_password = required_env("ADMIN_PASSWORD")?;
        let cookie_key = required_env("COOKIE_KEY")?;
        if cookie_key.len() < 32 {
            bail!(
                "COOKIE_KEY must be at least 32 bytes, got {} (try `openssl rand -base64 48`)",
                cookie_key.len()
            );
        }
        Ok(Self {
            database_url,
            bind_addr,
            family_tz,
            admin_password,
            cookie_key: Key::derive_from(cookie_key.as_bytes()),
        })
    }
}

fn env_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_owned())
}

fn required_env(key: &str) -> anyhow::Result<String> {
    match std::env::var(key) {
        Ok(value) if !value.is_empty() => Ok(value),
        _ => bail!("{key} must be set"),
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let config = Config::from_env()?;
    let pool = db::connect(&config.database_url).await?;

    let events = api::Events::new();
    let state = api::AppState {
        pool,
        family_tz: config.family_tz,
        events: events.clone(),
        admin_password: config.admin_password.into(),
        cookie_key: config.cookie_key,
    };
    let app = api::router(state);

    let listener = tokio::net::TcpListener::bind(config.bind_addr)
        .await
        .with_context(|| format!("failed to bind {}", config.bind_addr))?;
    tracing::info!("listening on {}", config.bind_addr);
    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            shutdown_signal().await;
            events.shutdown();
        })
        .await?;
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to listen for ctrl-c");
    };
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to listen for SIGTERM")
            .recv()
            .await;
    };
    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }
    tracing::info!("shutting down");
}
