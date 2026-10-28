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
    if !is_client_route(path) {
        return StatusCode::NOT_FOUND.into_response();
    }
    file("index.html").unwrap_or_else(|| StatusCode::NOT_FOUND.into_response())
}

/// Unknown paths are client-side routes, except things that look like files.
fn is_client_route(path: &str) -> bool {
    path.rsplit('/')
        .next()
        .is_none_or(|name| !name.contains('.'))
}

/// SvelteKit puts content-hashed files in _app/immutable/, so they can be cached forever.
fn is_immutable(path: &str) -> bool {
    path.starts_with("_app/immutable/")
}

fn file(path: &str) -> Option<Response> {
    let path = if path.is_empty() { "index.html" } else { path };
    let asset = Assets::get(path)?;
    let mime = mime_guess::from_path(path).first_or_octet_stream();
    let cache_control = if is_immutable(path) {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paths_without_a_file_name_are_client_routes() {
        assert!(is_client_route(""));
        assert!(is_client_route("manage"));
        assert!(is_client_route("manage/people"));
        assert!(!is_client_route("favicon.ico"));
        assert!(!is_client_route("_app/immutable/entry/start.BEclZ97v.js"));
    }

    #[test]
    fn only_hashed_files_are_immutable() {
        assert!(is_immutable("_app/immutable/entry/start.BEclZ97v.js"));
        assert!(is_immutable("_app/immutable/assets/0.C9rK0-DK.css"));
        assert!(!is_immutable("_app/version.json"));
        assert!(!is_immutable("index.html"));
    }
}
