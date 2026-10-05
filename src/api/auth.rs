use std::sync::Arc;
use std::time::Duration;

use axum::extract::{Request, State};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::{Json, Router};
use axum_extra::extract::SignedCookieJar;
use axum_extra::extract::cookie::{Cookie, SameSite};
use jiff::{SignedDuration, Timestamp};
use tower_governor::GovernorLayer;
use tower_governor::governor::GovernorConfigBuilder;
use tower_governor::key_extractor::GlobalKeyExtractor;

use super::types::LoginInput;
use super::{ApiError, ApiResult, AppState};

const SESSION_COOKIE: &str = "session";
const SESSION_LIFETIME: SignedDuration = SignedDuration::from_hours(365 * 24);
// Devices in regular use are re-issued a cookie, so they never hit the lifetime.
const RENEW_AFTER: SignedDuration = SignedDuration::from_hours(7 * 24);

async fn login(
    State(state): State<AppState>,
    jar: SignedCookieJar,
    Json(input): Json<LoginInput>,
) -> ApiResult<(SignedCookieJar, StatusCode)> {
    if !constant_time_eq(input.password.as_bytes(), state.admin_password.as_bytes()) {
        return Err(ApiError::Unauthorized("wrong password"));
    }
    Ok((
        jar.add(session_cookie(Timestamp::now())),
        StatusCode::NO_CONTENT,
    ))
}

pub async fn logout(jar: SignedCookieJar) -> (SignedCookieJar, StatusCode) {
    (
        jar.remove(Cookie::build(SESSION_COOKIE).path("/")),
        StatusCode::NO_CONTENT,
    )
}

/// Rejects requests without a valid session cookie.
pub async fn require_session(jar: SignedCookieJar, request: Request, next: Next) -> Response {
    let now = Timestamp::now();
    let Some(issued_at) = jar
        .get(SESSION_COOKIE)
        .and_then(|c| parse_issued_at(c.value()))
    else {
        return ApiError::Unauthorized("not logged in").into_response();
    };
    // The cookie's Max-Age is only a hint to the browser; a copied cookie must expire too.
    let age = now.duration_since(issued_at);
    if age > SESSION_LIFETIME {
        return ApiError::Unauthorized("session expired").into_response();
    }
    let response = next.run(request).await;
    if age > RENEW_AFTER {
        return (jar.add(session_cookie(now)), response).into_response();
    }
    response
}

/// `POST /login`, rate limited across all clients. Behind the ingress every request
/// comes from the same peer and forwarded-for headers can be spoofed, so a per-IP
/// limit would either be shared anyway or be trivially bypassed. Sessions last a
/// year, so a briefly locked login form is a minor nuisance.
pub fn login_routes() -> Router<AppState> {
    let config = GovernorConfigBuilder::default()
        .key_extractor(GlobalKeyExtractor)
        .period(Duration::from_secs(10))
        .burst_size(5)
        .finish()
        .expect("rate limit period and burst size are non-zero");
    let rate_limit = GovernorLayer::new(Arc::new(config))
        .error_handler(|_| ApiError::TooManyRequests.into_response());
    Router::new()
        .route("/login", post(login))
        .route_layer(rate_limit)
}

pub(super) fn session_cookie(issued_at: Timestamp) -> Cookie<'static> {
    Cookie::build((SESSION_COOKIE, issued_at.as_millisecond().to_string()))
        .path("/")
        .http_only(true)
        // Browsers treat http://localhost as secure, so this works in development too.
        .secure(true)
        // Also protects against CSRF: the API is only ever called from our own pages.
        .same_site(SameSite::Strict)
        .max_age(
            SESSION_LIFETIME
                .unsigned_abs()
                .try_into()
                .expect("session lifetime fits"),
        )
        .build()
}

fn parse_issued_at(value: &str) -> Option<Timestamp> {
    Timestamp::from_millisecond(value.parse().ok()?).ok()
}

/// Compares without short-circuiting, so response times don't reveal how much of a
/// guess was right. Only the length can leak.
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let diff = a.iter().zip(b).fold(0, |acc, (x, y)| acc | (x ^ y));
    std::hint::black_box(diff) == 0
}

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::http::{HeaderMap, Request, StatusCode, header};
    use jiff::{SignedDuration, Timestamp};
    use serde_json::json;
    use sqlx::SqlitePool;
    use tower::ServiceExt;

    use super::constant_time_eq;
    use crate::api::test_util::*;
    use crate::api::{AppState, router};

    fn get(uri: &str, cookie: Option<&str>) -> Request<Body> {
        let mut builder = Request::get(uri);
        if let Some(cookie) = cookie {
            builder = builder.header(header::COOKIE, cookie);
        }
        builder.body(Body::empty()).unwrap()
    }

    fn login_request(password: &str) -> Request<Body> {
        Request::post("/api/login")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(json!({ "password": password }).to_string()))
            .unwrap()
    }

    fn set_cookie(headers: &HeaderMap) -> Option<&str> {
        headers
            .get(header::SET_COOKIE)
            .map(|value| value.to_str().unwrap())
    }

    async fn status(state: &AppState, request: Request<Body>) -> StatusCode {
        send(state, request).await.status()
    }

    #[sqlx::test]
    async fn api_requires_a_session(pool: SqlitePool) {
        let state = state(&pool);
        for uri in ["/api/projects/1", "/api/executors", "/api/events"] {
            assert_eq!(
                status(&state, get(uri, None)).await,
                StatusCode::UNAUTHORIZED
            );
        }
        let put = Request::put("/api/completions/0199b4a2-7c00-7000-8000-000000000001")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from("{}"))
            .unwrap();
        assert_eq!(status(&state, put).await, StatusCode::UNAUTHORIZED);
    }

    #[sqlx::test]
    async fn health_and_assets_are_public(pool: SqlitePool) {
        let state = state(&pool);
        assert_eq!(status(&state, get("/healthz", None)).await, StatusCode::OK);
        assert_eq!(status(&state, get("/", None)).await, StatusCode::OK);
    }

    #[sqlx::test]
    async fn login_issues_a_working_session(pool: SqlitePool) {
        let state = state(&pool);
        let response = send(&state, login_request(PASSWORD)).await;
        assert_eq!(response.status(), StatusCode::NO_CONTENT);
        let issued = set_cookie(response.headers()).unwrap();
        for attribute in [
            "HttpOnly",
            "Secure",
            "SameSite=Strict",
            "Path=/",
            "Max-Age=31536000",
        ] {
            assert!(issued.contains(attribute), "{issued:?}");
        }

        let cookie = issued.split(';').next().unwrap();
        let response = send(&state, get("/api/projects/1", Some(cookie))).await;
        assert_eq!(response.status(), StatusCode::OK);
        // A fresh session isn't renewed on every request.
        assert_eq!(set_cookie(response.headers()), None);
    }

    #[sqlx::test]
    async fn wrong_password_is_rejected(pool: SqlitePool) {
        let state = state(&pool);
        for password in ["", "wrong", &format!("{PASSWORD} ")] {
            let response = send(&state, login_request(password)).await;
            assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
            assert_eq!(set_cookie(response.headers()), None);
        }
    }

    #[sqlx::test]
    async fn forged_sessions_are_rejected(pool: SqlitePool) {
        let state = state(&pool);
        let now = Timestamp::now().as_millisecond();
        let other_key = AppState {
            cookie_key: axum_extra::extract::cookie::Key::derive_from(&[8; 32]),
            ..state.clone()
        };
        let cookie = session_cookie(&state, Timestamp::now());
        let (_, signature_and_value) = cookie.split_once('=').unwrap();
        let tampered = format!(
            "session={}{}",
            &signature_and_value[..signature_and_value.len() - 13],
            now + 1
        );
        for cookie in [
            format!("session={now}"),
            session_cookie(&other_key, Timestamp::now()),
            tampered,
        ] {
            let status = status(&state, get("/api/projects/1", Some(&cookie))).await;
            assert_eq!(status, StatusCode::UNAUTHORIZED, "{cookie:?}");
        }
    }

    #[sqlx::test]
    async fn sessions_expire_after_a_year(pool: SqlitePool) {
        let state = state(&pool);
        let issued_at = Timestamp::now() - SignedDuration::from_hours(366 * 24);
        let cookie = session_cookie(&state, issued_at);
        let status = status(&state, get("/api/projects/1", Some(&cookie))).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
    }

    #[sqlx::test]
    async fn old_sessions_are_renewed(pool: SqlitePool) {
        let state = state(&pool);
        let issued_at = Timestamp::now() - SignedDuration::from_hours(30 * 24);
        let cookie = session_cookie(&state, issued_at);
        let response = send(&state, get("/api/projects/1", Some(&cookie))).await;
        assert_eq!(response.status(), StatusCode::OK);
        let renewed = set_cookie(response.headers()).unwrap();
        assert!(renewed.contains("Max-Age=31536000"), "{renewed:?}");
        assert_ne!(renewed.split(';').next().unwrap(), cookie);
    }

    #[sqlx::test]
    async fn logout_removes_the_cookie(pool: SqlitePool) {
        let state = state(&pool);
        let request = Request::post("/api/logout")
            .header(header::COOKIE, session_cookie(&state, Timestamp::now()))
            .body(Body::empty())
            .unwrap();
        let response = send(&state, request).await;
        assert_eq!(response.status(), StatusCode::NO_CONTENT);
        let removal = set_cookie(response.headers()).unwrap();
        assert!(removal.starts_with("session=;"), "{removal:?}");
        assert!(removal.contains("Max-Age=0"), "{removal:?}");
    }

    #[sqlx::test]
    async fn login_attempts_are_rate_limited(pool: SqlitePool) {
        // One router, so all attempts share a rate limiter like in production.
        let app = router(state(&pool));
        for _ in 0..5 {
            let response = app.clone().oneshot(login_request("wrong")).await.unwrap();
            assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        }
        // Even the right password is turned away until the limit recovers.
        let response = app.clone().oneshot(login_request(PASSWORD)).await.unwrap();
        assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
    }

    #[test]
    fn constant_time_eq_compares_contents_and_length() {
        assert!(constant_time_eq(b"secret", b"secret"));
        assert!(constant_time_eq(b"", b""));
        assert!(!constant_time_eq(b"secret", b"secreT"));
        assert!(!constant_time_eq(b"secret", b"secret!"));
        assert!(!constant_time_eq(b"", b"x"));
    }
}
