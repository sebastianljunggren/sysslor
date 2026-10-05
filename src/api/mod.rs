mod assets;
mod auth;
mod completions;
mod events;
mod executors;
mod projects;
mod tasks;
#[cfg(test)]
mod test_util;
pub mod types;

use std::sync::Arc;

use axum::extract::{FromRef, State};
use axum::http::StatusCode;
use axum::middleware;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post, put};
use axum::{Json, Router};
use axum_extra::extract::cookie::Key;
use jiff::tz::TimeZone;
use sqlx::SqlitePool;

pub use events::Events;
use types::ErrorBody;

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub family_tz: TimeZone,
    pub events: Events,
    pub admin_password: Arc<str>,
    pub cookie_key: Key,
}

impl FromRef<AppState> for Key {
    fn from_ref(state: &AppState) -> Self {
        state.cookie_key.clone()
    }
}

pub fn router(state: AppState) -> Router {
    let api = Router::new()
        .route("/events", get(events::stream))
        .route("/projects/{id}", get(projects::get))
        .route("/projects/{id}/tasks", post(tasks::create))
        .route("/tasks/{id}", put(tasks::update).delete(tasks::archive))
        .route("/executors", get(executors::list).post(executors::create))
        .route("/executors/{id}", put(executors::rename))
        .route(
            "/completions/{id}",
            put(completions::put).delete(completions::delete),
        )
        // A route layer, so it doesn't apply to the routes merged below.
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth::require_session,
        ))
        .merge(auth::login_routes())
        .route("/logout", post(auth::logout))
        // Without this, unknown API paths would get the SPA's index.html.
        .fallback(|| async { ApiError::NotFound("no such endpoint") });

    Router::new()
        .route("/healthz", get(healthz))
        .nest("/api", api)
        .fallback(assets::serve)
        .with_state(state)
}

async fn healthz(State(state): State<AppState>) -> StatusCode {
    match sqlx::query("SELECT 1").execute(&state.pool).await {
        Ok(_) => StatusCode::OK,
        Err(err) => {
            tracing::error!("health check failed: {err}");
            StatusCode::SERVICE_UNAVAILABLE
        }
    }
}

#[derive(Debug)]
pub enum ApiError {
    NotFound(&'static str),
    Unauthorized(&'static str),
    TooManyRequests,
    /// The request is well-formed JSON but its values are not acceptable.
    Invalid(String),
    Internal(anyhow::Error),
}

impl From<anyhow::Error> for ApiError {
    fn from(err: anyhow::Error) -> Self {
        Self::Internal(err)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error) = match self {
            Self::NotFound(what) => (StatusCode::NOT_FOUND, what.to_owned()),
            Self::Unauthorized(why) => (StatusCode::UNAUTHORIZED, why.to_owned()),
            Self::TooManyRequests => (
                StatusCode::TOO_MANY_REQUESTS,
                "too many attempts, try again later".to_owned(),
            ),
            Self::Invalid(message) => (StatusCode::UNPROCESSABLE_ENTITY, message),
            Self::Internal(err) => {
                tracing::error!("request failed: {err:#}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal error".to_owned(),
                )
            }
        };
        (status, Json(ErrorBody { error })).into_response()
    }
}

pub type ApiResult<T> = Result<T, ApiError>;

/// Trims a user-supplied name and rejects empty or absurdly long ones.
fn validate_name(name: &str) -> ApiResult<String> {
    let name = name.trim();
    if name.is_empty() {
        return Err(ApiError::Invalid("name must not be empty".to_owned()));
    }
    if name.chars().count() > 200 {
        return Err(ApiError::Invalid(
            "name must be at most 200 characters".to_owned(),
        ));
    }
    Ok(name.to_owned())
}

#[cfg(test)]
mod tests {
    use super::test_util::*;
    use axum::http::StatusCode;
    use sqlx::SqlitePool;

    #[sqlx::test]
    async fn healthz_is_ok(pool: SqlitePool) {
        let (status, _) = request(&pool, "GET", "/healthz", None).await;
        assert_eq!(status, StatusCode::OK);
    }

    #[sqlx::test]
    async fn unknown_paths_fall_back_to_index(pool: SqlitePool) {
        let (status, body) = request(&pool, "GET", "/some/client/route", None).await;
        assert_eq!(status, StatusCode::OK);
        assert!(body.contains("<html"));
    }

    #[sqlx::test]
    async fn unknown_api_paths_are_not_found(pool: SqlitePool) {
        let (status, body) = request(&pool, "GET", "/api/nope", None).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert!(body.contains("\"error\""));
    }
}
