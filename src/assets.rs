//! Embedded frontend serving.
//!
//! The Vite build output (`frontend/dist`) is compiled into the binary. Real
//! files are served with their content type; unknown paths fall back to
//! `index.html` so the SPA's client-side router handles them.

use axum::http::{header, StatusCode, Uri};
use axum::response::{IntoResponse, Response};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "frontend/dist"]
struct Frontend;

pub async fn static_handler(uri: Uri) -> Response {
    let path = uri.path().trim_start_matches('/');
    let path = if path.is_empty() { "index.html" } else { path };

    match Frontend::get(path) {
        Some(content) => serve(path, content),
        None => {
            // SPA fallback: serve index.html for client-routed paths.
            match Frontend::get("index.html") {
                Some(content) => serve("index.html", content),
                None => (StatusCode::NOT_FOUND, "not found").into_response(),
            }
        }
    }
}

fn serve(path: &str, content: rust_embed::EmbeddedFile) -> Response {
    let mime = mime_guess::from_path(path).first_or_octet_stream();
    // Long-cache hashed assets; keep index.html fresh so deploys take effect.
    let cache = if path == "index.html" {
        "no-cache"
    } else if path.starts_with("assets/") {
        "public, max-age=31536000, immutable"
    } else {
        "public, max-age=3600"
    };
    (
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, mime.as_ref().to_string()),
            (header::CACHE_CONTROL, cache.to_string()),
        ],
        content.data.into_owned(),
    )
        .into_response()
}
