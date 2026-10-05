use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use axum::response::Response;
use http_body_util::BodyExt;
use jiff::tz::TimeZone;
use serde::de::DeserializeOwned;
use serde_json::Value;
use sqlx::SqlitePool;
use tower::ServiceExt;

use super::{AppState, Events, router};

pub fn state(pool: &SqlitePool) -> AppState {
    AppState {
        pool: pool.clone(),
        family_tz: TimeZone::get("Europe/Stockholm").unwrap(),
        events: Events::new(),
    }
}

/// Sends one request through the full router and returns the status and raw body.
pub async fn request(
    pool: &SqlitePool,
    method: &str,
    uri: &str,
    body: Option<Value>,
) -> (StatusCode, String) {
    request_with(&state(pool), method, uri, body).await
}

/// Like [`request`], but with a given state, e.g. to observe its events.
pub async fn request_with(
    state: &AppState,
    method: &str,
    uri: &str,
    body: Option<Value>,
) -> (StatusCode, String) {
    let response = response_with(state, method, uri, body).await;
    let status = response.status();
    let body = response.into_body().collect().await.unwrap().to_bytes();
    (status, String::from_utf8_lossy(&body).into_owned())
}

/// Like [`request_with`], but returns the response without reading the body.
pub async fn response_with(
    state: &AppState,
    method: &str,
    uri: &str,
    body: Option<Value>,
) -> Response {
    let builder = Request::builder().method(method).uri(uri);
    let request = match body {
        Some(json) => builder
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(json.to_string())),
        None => builder.body(Body::empty()),
    }
    .unwrap();
    router(state.clone()).oneshot(request).await.unwrap()
}

/// Like [`request`], but parses the body as JSON.
pub async fn json<T: DeserializeOwned>(
    pool: &SqlitePool,
    method: &str,
    uri: &str,
    body: Option<Value>,
) -> (StatusCode, T) {
    let (status, text) = request(pool, method, uri, body).await;
    let parsed = serde_json::from_str(&text)
        .unwrap_or_else(|err| panic!("{method} {uri} returned {status} {text:?}: {err}"));
    (status, parsed)
}

pub const DEFAULT_PROJECT: i64 = 1;

pub fn now_ms() -> i64 {
    jiff::Timestamp::now().as_millisecond()
}

/// Now minus `days` calendar days in the family time zone. Subtracting `days * 24h`
/// instead would make tests flaky around DST changes.
pub fn days_ago(days: i64) -> i64 {
    jiff::Zoned::now()
        .with_time_zone(TimeZone::get("Europe/Stockholm").unwrap())
        .checked_sub(jiff::Span::new().days(days))
        .unwrap()
        .timestamp()
        .as_millisecond()
}

/// Creates a task in the default project and returns its id.
pub async fn create_task(pool: &SqlitePool, name: &str, days: u32, priority: u8) -> i64 {
    let body = serde_json::json!({
        "name": name,
        "cadence": { "amount": days, "unit": "days" },
        "priority": priority,
    });
    let (status, task): (_, Value) = json(pool, "POST", "/api/projects/1/tasks", Some(body)).await;
    assert_eq!(status, StatusCode::CREATED);
    task["id"].as_i64().unwrap()
}

/// Creates an executor and returns its id.
pub async fn create_executor(pool: &SqlitePool, name: &str) -> i64 {
    let body = serde_json::json!({ "name": name });
    let (status, executor): (_, Value) = json(pool, "POST", "/api/executors", Some(body)).await;
    assert_eq!(status, StatusCode::CREATED);
    executor["id"].as_i64().unwrap()
}

pub async fn complete(
    pool: &SqlitePool,
    id: &str,
    task_id: i64,
    executor_id: i64,
    completed_at: i64,
) -> (StatusCode, String) {
    let body = serde_json::json!({
        "task_id": task_id,
        "executor_id": executor_id,
        "completed_at": completed_at,
    });
    request(pool, "PUT", &format!("/api/completions/{id}"), Some(body)).await
}

pub async fn default_project(pool: &SqlitePool) -> crate::api::types::ProjectView {
    let (status, view) = json(pool, "GET", "/api/projects/1", None).await;
    assert_eq!(status, StatusCode::OK);
    view
}
