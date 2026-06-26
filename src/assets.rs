//! Embedded frontend serving.
//!
//! The Vite build output (`frontend/dist`) is compiled into the binary. To keep
//! per-request CPU low (the site sits behind nginx and gets hammered), every
//! asset is preprocessed ONCE at startup into [`Asset`]:
//!
//!   * the identity bytes, plus precompressed gzip + brotli variants (so we
//!     never compress per request — only pick the right precomputed blob),
//!   * a strong ETag (the embed's SHA-256) for `If-None-Match` -> 304,
//!   * static `Content-Type` / `Cache-Control` header values (no per-request
//!     `String` allocation).
//!
//! Bodies are served as `Bytes` (zero-copy clone = an atomic refcount bump),
//! so a request copies the body once into the socket and nothing more. Unknown
//! paths fall back to `index.html` for the SPA's client-side router.

use std::collections::HashMap;
use std::sync::OnceLock;

use axum::body::{Body, Bytes};
use axum::http::{header, HeaderMap, HeaderValue, StatusCode, Uri};
use axum::response::{IntoResponse, Response};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "frontend/dist"]
struct Frontend;

/// One embedded asset, fully prepared for serving.
struct Asset {
    content_type: HeaderValue,
    cache: HeaderValue,
    etag: HeaderValue,
    identity: Bytes,
    gzip: Option<Bytes>,
    brotli: Option<Bytes>,
}

static ASSETS: OnceLock<HashMap<String, Asset>> = OnceLock::new();

fn assets() -> &'static HashMap<String, Asset> {
    ASSETS.get_or_init(build_assets)
}

/// Build the precompressed asset cache. Call once at startup (only the serving
/// process) so the first real request doesn't pay the compression cost.
pub fn warm() {
    let map = assets();
    tracing::info!(assets = map.len(), "frontend asset cache built (gzip+brotli precompressed)");
}

fn build_assets() -> HashMap<String, Asset> {
    let mut map = HashMap::new();
    for path in Frontend::iter() {
        let path = path.into_owned();
        if let Some(file) = Frontend::get(&path) {
            let asset = make_asset(&path, file);
            map.insert(path, asset);
        }
    }
    map
}

fn make_asset(path: &str, file: rust_embed::EmbeddedFile) -> Asset {
    let identity = match file.data {
        std::borrow::Cow::Borrowed(b) => Bytes::from_static(b),
        std::borrow::Cow::Owned(v) => Bytes::from(v),
    };

    // Strong ETag from the embed's precomputed SHA-256 — cheap and stable.
    let etag = {
        use std::fmt::Write;
        let mut s = String::with_capacity(66);
        s.push('"');
        for b in file.metadata.sha256_hash() {
            let _ = write!(s, "{b:02x}");
        }
        s.push('"');
        HeaderValue::from_str(&s).expect("hex etag is valid")
    };

    // Precompress text-y assets that are big enough to be worth it. Keep a
    // variant only if it actually shrank the body.
    let (gzip, brotli) = if is_compressible(path) && identity.len() >= 256 {
        (gzip_bytes(&identity), brotli_bytes(&identity))
    } else {
        (None, None)
    };

    Asset {
        content_type: content_type(path),
        cache: cache_control(path),
        etag,
        identity,
        gzip,
        brotli,
    }
}

pub async fn static_handler(uri: Uri, headers: HeaderMap) -> Response {
    let raw = uri.path().trim_start_matches('/');
    let key = if raw.is_empty() { "index.html" } else { raw };
    let map = assets();

    let asset = match map.get(key) {
        Some(a) => a,
        // SPA fallback: serve index.html for client-routed paths.
        None => match map.get("index.html") {
            Some(a) => a,
            None => return (StatusCode::NOT_FOUND, "not found").into_response(),
        },
    };
    serve(asset, &headers)
}

fn serve(asset: &Asset, headers: &HeaderMap) -> Response {
    // Conditional request: a matching If-None-Match -> 304 with an empty body.
    if let Some(inm) = headers.get(header::IF_NONE_MATCH) {
        if etag_matches(inm, &asset.etag) {
            return Response::builder()
                .status(StatusCode::NOT_MODIFIED)
                .header(header::ETAG, asset.etag.clone())
                .header(header::CACHE_CONTROL, asset.cache.clone())
                .body(Body::empty())
                .expect("304 response is valid");
        }
    }

    // Content negotiation against the precompressed variants (br > gzip > id).
    let accept = headers
        .get(header::ACCEPT_ENCODING)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let (body, encoding) = if accepts(accept, "br") && asset.brotli.is_some() {
        (asset.brotli.clone().unwrap(), Some("br"))
    } else if accepts(accept, "gzip") && asset.gzip.is_some() {
        (asset.gzip.clone().unwrap(), Some("gzip"))
    } else {
        (asset.identity.clone(), None)
    };

    let mut builder = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, asset.content_type.clone())
        .header(header::CACHE_CONTROL, asset.cache.clone())
        .header(header::ETAG, asset.etag.clone())
        .header(header::VARY, HeaderValue::from_static("Accept-Encoding"));
    if let Some(enc) = encoding {
        builder = builder.header(header::CONTENT_ENCODING, HeaderValue::from_static(enc));
    }
    builder
        .body(Body::from(body))
        .expect("asset response is valid")
}

/// Does the Accept-Encoding header allow `enc` (and not via an explicit q=0)?
fn accepts(accept_encoding: &str, enc: &str) -> bool {
    accept_encoding.split(',').any(|part| {
        let mut it = part.split(';');
        let tok = it.next().unwrap_or("").trim();
        if !tok.eq_ignore_ascii_case(enc) && tok != "*" {
            return false;
        }
        // Reject only if an explicit q=0 is attached.
        !it.any(|p| {
            let p = p.trim();
            p == "q=0" || p == "q=0.0" || p == "q=0.00" || p == "q=0.000"
        })
    })
}

/// RFC-7232 weak comparison of the request's If-None-Match against our ETag.
fn etag_matches(inm: &HeaderValue, etag: &HeaderValue) -> bool {
    let Ok(inm) = inm.to_str() else {
        return false;
    };
    let want = etag.to_str().unwrap_or("");
    inm.split(',').any(|t| {
        let t = t.trim().trim_start_matches("W/");
        t == "*" || t == want
    })
}

/// Text-y assets worth compressing. Already-compressed binaries are skipped.
fn is_compressible(path: &str) -> bool {
    matches!(
        ext(path),
        "html" | "js" | "mjs" | "css" | "svg" | "json" | "map" | "txt" | "xml" | "ico" | "wasm"
    )
}

fn ext(path: &str) -> &str {
    path.rsplit('.').next().unwrap_or("")
}

/// Static content type by extension — avoids `mime_guess` + a `String` alloc on
/// every request. Covers everything the embedded site actually serves.
fn content_type(path: &str) -> HeaderValue {
    HeaderValue::from_static(match ext(path) {
        "html" => "text/html; charset=utf-8",
        "js" | "mjs" => "text/javascript; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "svg" => "image/svg+xml",
        "json" | "map" => "application/json",
        "ico" => "image/x-icon",
        "png" => "image/png",
        "webp" => "image/webp",
        "woff2" => "font/woff2",
        "woff" => "font/woff",
        "wasm" => "application/wasm",
        "txt" => "text/plain; charset=utf-8",
        _ => "application/octet-stream",
    })
}

fn cache_control(path: &str) -> HeaderValue {
    // Long-cache hashed assets; keep index.html fresh so deploys take effect.
    HeaderValue::from_static(if path == "index.html" {
        "no-cache"
    } else if path.starts_with("assets/") {
        "public, max-age=31536000, immutable"
    } else {
        "public, max-age=3600"
    })
}

fn gzip_bytes(data: &[u8]) -> Option<Bytes> {
    use flate2::{write::GzEncoder, Compression};
    use std::io::Write;
    let mut e = GzEncoder::new(Vec::new(), Compression::best());
    e.write_all(data).ok()?;
    let out = e.finish().ok()?;
    (out.len() < data.len()).then(|| Bytes::from(out))
}

fn brotli_bytes(data: &[u8]) -> Option<Bytes> {
    use std::io::Write;
    // quality 11, window 22 — max ratio; this runs once per asset at startup.
    let mut w = brotli::CompressorWriter::new(Vec::new(), 4096, 11, 22);
    w.write_all(data).ok()?;
    let out = w.into_inner();
    (out.len() < data.len()).then(|| Bytes::from(out))
}
