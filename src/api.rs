//! JSON API + installer for the DN7 website.
//!
//! For v1 the binaries still live at the existing distribution origin
//! (`crate::UPSTREAM`); these handlers present them under the dn7.cn brand:
//!   * `/start.sh`            -> installer that downloads from dn7.cn/api/...
//!   * `/api/panel/latest`    -> release manifest (proxied + rebranded)
//!   * `/api/panel/download`  -> streams the binary (proxied)

use axum::body::Body;
use axum::extract::{Query, State};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;
use serde_json::json;

use crate::AppState;

pub async fn health() -> Response {
    Json(json!({ "ok": true })).into_response()
}

#[derive(Debug, Deserialize)]
pub struct ArchQuery {
    #[serde(default)]
    pub arch: Option<String>,
}

fn norm_arch(a: Option<&str>) -> &'static str {
    match a.map(|s| s.trim().to_lowercase()).as_deref() {
        Some("arm64") | Some("aarch64") => "arm64",
        _ => "x86_64",
    }
}

/// GET /api/panel/latest — the DN7 Panel release manifest. Proxies the upstream
/// distribution manifest and rewrites the download URLs to go through dn7.cn so
/// the public site is the single entry point.
pub async fn panel_latest(State(state): State<AppState>) -> Response {
    let url = format!("{}/agent/dist/latest", crate::UPSTREAM);
    let upstream = match state.http.get(&url).send().await {
        Ok(r) => r,
        Err(e) => {
            return (
                StatusCode::BAD_GATEWAY,
                Json(json!({ "ok": false, "error": format!("upstream unreachable: {e}") })),
            )
                .into_response()
        }
    };
    let body: serde_json::Value = match upstream.json().await {
        Ok(v) => v,
        Err(_) => {
            return (
                StatusCode::BAD_GATEWAY,
                Json(json!({ "ok": false, "error": "bad upstream response" })),
            )
                .into_response()
        }
    };
    let data = body.get("data").cloned().unwrap_or(json!({}));
    let version = data
        .get("version")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let sizes = data.get("sizes").cloned().unwrap_or(json!({}));
    let sha256 = data.get("sha256").cloned().unwrap_or(json!({}));

    // Rewrite per-arch download URLs to dn7.cn.
    let mut downloads = serde_json::Map::new();
    for arch in ["x86_64", "arm64"] {
        downloads.insert(
            arch.to_string(),
            json!(format!("/api/panel/download?arch={arch}")),
        );
    }

    Json(json!({
        "ok": true,
        "data": {
            "product": "DN7 Panel",
            "version": version,
            "sizes": sizes,
            "sha256": sha256,
            "downloads": downloads,
            "install": "curl -fsSL https://dn7.cn/start.sh | sh",
        }
    }))
    .into_response()
}

/// GET /api/panel/download?arch= — stream the binary from the upstream origin.
pub async fn panel_download(State(state): State<AppState>, Query(q): Query<ArchQuery>) -> Response {
    let arch = norm_arch(q.arch.as_deref());
    let url = format!("{}/agent/dist/download?arch={arch}", crate::UPSTREAM);
    let upstream = match state.http.get(&url).send().await {
        Ok(r) => r,
        Err(e) => {
            return (
                StatusCode::BAD_GATEWAY,
                format!("upstream unreachable: {e}"),
            )
                .into_response()
        }
    };
    if !upstream.status().is_success() {
        return (StatusCode::BAD_GATEWAY, "binary not available yet").into_response();
    }
    // Carry through the filename + content type, stream the body.
    let disposition = upstream
        .headers()
        .get(header::CONTENT_DISPOSITION)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("attachment")
        .to_string();
    let length = upstream
        .headers()
        .get(header::CONTENT_LENGTH)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let mut resp = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/octet-stream")
        .header(header::CONTENT_DISPOSITION, disposition);
    if let Some(len) = length {
        resp = resp.header(header::CONTENT_LENGTH, len);
    }
    resp.body(Body::from_stream(upstream.bytes_stream()))
        .unwrap_or_else(|_| (StatusCode::INTERNAL_SERVER_ERROR, "stream error").into_response())
}

/// GET /start.sh — the one-line installer for DN7 Panel.
///
///   curl -fsSL https://dn7.cn/start.sh | sh
///
/// Detects the CPU arch and downloads the latest DN7 Panel from dn7.cn (which
/// proxies the current distribution origin), makes it executable, and runs it.
pub async fn start_script() -> Response {
    let script = r#"#!/bin/sh
# Digital Network 7 — DN7 Panel installer.
# Usage: curl -fsSL https://dn7.cn/start.sh | sh
set -e
SITE="https://dn7.cn"

case "$(uname -m)" in
  x86_64|amd64) ARCH=x86_64 ;;
  aarch64|arm64) ARCH=arm64 ;;
  *) echo "[dn7] unsupported arch: $(uname -m)" >&2; exit 1 ;;
esac

URL="$SITE/api/panel/download?arch=$ARCH"
OUT=dn7-panel

echo "[dn7] downloading DN7 Panel ($ARCH) ..."
if command -v curl >/dev/null 2>&1; then
  curl -fL --progress-bar "$URL" -o "$OUT"
elif command -v wget >/dev/null 2>&1; then
  wget -O "$OUT" "$URL"
else
  echo "[dn7] neither curl nor wget found" >&2; exit 1
fi

chmod +x "$OUT"
echo "[dn7] starting DN7 Panel ..."
./"$OUT"
"#;
    (
        StatusCode::OK,
        [(
            header::CONTENT_TYPE,
            "text/x-shellscript; charset=utf-8".to_string(),
        )],
        script,
    )
        .into_response()
}
