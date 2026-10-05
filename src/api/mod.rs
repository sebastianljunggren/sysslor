mod assets;

use axum::Router;
use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::get;
use jiff::tz::TimeZone;
use sqlx::SqlitePool;

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    #[allow(dead_code)] // used by the task endpoints in milestone 2
    pub family_tz: TimeZone,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/healthz", get(healthz))
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

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    async fn app() -> Router {
        let pool = crate::db::connect("sqlite::memory:").await.unwrap();
        router(AppState {
            pool,
            family_tz: TimeZone::UTC,
        })
    }

    async fn get(app: Router, uri: &str) -> (StatusCode, String) {
        let response = app
            .oneshot(Request::get(uri).body(Body::empty()).unwrap())
            .await
            .unwrap();
        let status = response.status();
        let body = response.into_body().collect().await.unwrap().to_bytes();
        (status, String::from_utf8_lossy(&body).into_owned())
    }

    #[tokio::test]
    async fn healthz_is_ok() {
        let (status, _) = get(app().await, "/healthz").await;
        assert_eq!(status, StatusCode::OK);
    }

    #[tokio::test]
    async fn unknown_paths_fall_back_to_index() {
        let (status, body) = get(app().await, "/some/client/route").await;
        assert_eq!(status, StatusCode::OK);
        assert!(body.contains("<html"));
    }
}
