use axum::http::{StatusCode, Uri, header};
use axum::response::{IntoResponse, Response};
use rust_embed::Embed;

/// The built Svelte app. Read from disk in debug builds, embedded in release builds.
#[derive(Embed)]
#[folder = "web/dist"]
struct Assets;

pub async fn serve(uri: Uri) -> Response {
    let path = uri.path().trim_start_matches('/');
    if let Some(response) = file(path) {
        return response;
    }
    // Unknown paths are client-side routes, except things that look like files.
    if path
        .rsplit('/')
        .next()
        .is_some_and(|name| name.contains('.'))
    {
        return StatusCode::NOT_FOUND.into_response();
    }
    file("index.html").unwrap_or_else(|| StatusCode::NOT_FOUND.into_response())
}

fn file(path: &str) -> Option<Response> {
    let path = if path.is_empty() { "index.html" } else { path };
    let asset = Assets::get(path)?;
    let mime = mime_guess::from_path(path).first_or_octet_stream();
    // Vite puts content-hashed files in assets/, so they can be cached forever.
    let cache_control = if path.starts_with("assets/") {
        "public, max-age=31536000, immutable"
    } else {
        "no-cache"
    };
    Some(
        (
            [
                (header::CONTENT_TYPE, mime.as_ref()),
                (header::CACHE_CONTROL, cache_control),
            ],
            asset.data,
        )
            .into_response(),
    )
}
