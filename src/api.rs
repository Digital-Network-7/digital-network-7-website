//! JSON API + installer for the DN7 website.
//!
//! dn7.cn is a domestic mirror/front-end for the DN7 Panel GitHub releases. The
//! binaries are published on GitHub (`Digital-Network-7/DN7-Panel`); these
//! handlers resolve the latest release deterministically (via the
//! `releases/latest` redirect — no api.github.com, so no rate limit) and:
//!   * `/start.sh`            -> installer that downloads from dn7.cn/api/...
//!   * `/api/panel/version`   -> release manifest the panel's updater consumes
//!   * `/api/panel/latest`    -> richer manifest for the website UI
//!   * `/api/panel/download`  -> streams the binary (proxied from GitHub)

use axum::body::Body;
use axum::extract::{Query, State};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;
use serde_json::json;

use crate::{AppState, GITHUB_REPO};

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

/// Public base URL used to build absolute download links in manifests.
fn public_base() -> String {
    std::env::var("DN7_PUBLIC_URL")
        .unwrap_or_else(|_| "https://dn7.cn".to_string())
        .trim_end_matches('/')
        .to_string()
}

fn asset_name(version: &str, arch: &str) -> String {
    format!("dn7-panel-linux-{arch}-v{version}")
}

/// Resolve the latest release tag (e.g. `v1.0.9`) without hitting
/// api.github.com: follow the `releases/latest` redirect and read the final
/// URL's tag segment.
async fn latest_tag(http: &reqwest::Client) -> anyhow::Result<String> {
    let url = format!("https://github.com/{GITHUB_REPO}/releases/latest");
    let resp = http.get(&url).send().await?;
    let final_url = resp.url().as_str().to_string();
    let tag = final_url
        .rsplit('/')
        .next()
        .unwrap_or("")
        .trim()
        .to_string();
    if tag.is_empty() || !tag.starts_with('v') {
        anyhow::bail!("could not resolve latest tag (final url: {final_url})");
    }
    Ok(tag)
}

/// Best-effort: fetch the release's SHA256SUMS and find `asset`'s hash.
async fn sha_for(http: &reqwest::Client, tag: &str, asset: &str) -> Option<String> {
    let url = format!("https://github.com/{GITHUB_REPO}/releases/download/{tag}/SHA256SUMS");
    let body = http.get(&url).send().await.ok()?.text().await.ok()?;
    for line in body.lines() {
        let mut it = line.split_whitespace();
        let hash = it.next()?;
        let name = it.next().unwrap_or("").trim_start_matches('*');
        if name == asset && hash.len() == 64 {
            return Some(hash.to_lowercase());
        }
    }
    None
}

/// GET /api/panel/version?arch= — the manifest the panel's self-update DN7
/// source consumes: version + absolute download URL + sha256.
pub async fn panel_version(State(state): State<AppState>, Query(q): Query<ArchQuery>) -> Response {
    let arch = norm_arch(q.arch.as_deref());
    let tag = match latest_tag(&state.http).await {
        Ok(t) => t,
        Err(e) => {
            return (
                StatusCode::BAD_GATEWAY,
                Json(json!({ "ok": false, "error": format!("upstream unreachable: {e}") })),
            )
                .into_response()
        }
    };
    let version = tag.trim_start_matches('v').to_string();
    let asset = asset_name(&version, arch);
    let base = public_base();
    Json(json!({
        "ok": true,
        "data": {
            "product": "DN7 Panel",
            "version": version,
            "arch": arch,
            "url": format!("{base}/api/panel/download?arch={arch}"),
            "asset": asset,
        }
    }))
    .into_response()
}

/// GET /api/panel/latest — richer manifest for the website UI (both arches).
pub async fn panel_latest(State(state): State<AppState>) -> Response {
    let tag = match latest_tag(&state.http).await {
        Ok(t) => t,
        Err(e) => {
            return (
                StatusCode::BAD_GATEWAY,
                Json(json!({ "ok": false, "error": format!("upstream unreachable: {e}") })),
            )
                .into_response()
        }
    };
    let version = tag.trim_start_matches('v').to_string();
    let base = public_base();
    let mut downloads = serde_json::Map::new();
    let mut sha256 = serde_json::Map::new();
    for arch in ["x86_64", "arm64"] {
        downloads.insert(
            arch.to_string(),
            json!(format!("{base}/api/panel/download?arch={arch}")),
        );
        if let Some(h) = sha_for(&state.http, &tag, &asset_name(&version, arch)).await {
            sha256.insert(arch.to_string(), json!(h));
        }
    }
    Json(json!({
        "ok": true,
        "data": {
            "product": "DN7 Panel",
            "version": version,
            "downloads": downloads,
            "sha256": sha256,
            "install": "curl -fsSL https://dn7.cn/start.sh | sh",
        }
    }))
    .into_response()
}

/// Fetch + parse the changelog index asset from the latest GitHub release.
async fn fetch_releases_index(http: &reqwest::Client) -> anyhow::Result<serde_json::Value> {
    let url = format!("https://github.com/{GITHUB_REPO}/releases/latest/download/releases.json");
    Ok(http
        .get(&url)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?)
}

/// GET /api/panel/releases — the changelog index the panel's "what's new" view
/// consumes. Mirrored from the GitHub release asset reachable via the
/// deterministic `releases/latest/download/` redirect (no api.github.com), so
/// dn7.cn can serve the same changelog when GitHub is slow/blocked.
pub async fn panel_releases(State(state): State<AppState>) -> Response {
    match fetch_releases_index(&state.http).await {
        Ok(v) => {
            // Accept either a bare index or one wrapped in {data:...}.
            let inner = v.get("data").cloned().unwrap_or(v);
            Json(json!({ "ok": true, "data": inner })).into_response()
        }
        Err(e) => (
            StatusCode::BAD_GATEWAY,
            Json(json!({ "ok": false, "error": format!("upstream unreachable: {e}") })),
        )
            .into_response(),
    }
}

/// GET /api/panel/download?arch= — stream the binary from the GitHub release.
pub async fn panel_download(State(state): State<AppState>, Query(q): Query<ArchQuery>) -> Response {
    let arch = norm_arch(q.arch.as_deref());
    let tag = match latest_tag(&state.http).await {
        Ok(t) => t,
        Err(e) => {
            return (
                StatusCode::BAD_GATEWAY,
                format!("upstream unreachable: {e}"),
            )
                .into_response()
        }
    };
    let version = tag.trim_start_matches('v');
    let asset = asset_name(version, arch);
    let url = format!("https://github.com/{GITHUB_REPO}/releases/download/{tag}/{asset}");
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
    let length = upstream
        .headers()
        .get(header::CONTENT_LENGTH)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    let mut resp = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/octet-stream")
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{asset}\""),
        );
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
/// mirrors the GitHub release), makes it executable, and runs it.
pub async fn start_script() -> Response {
    let script = r#"#!/bin/sh
# Digital Network 7 — DN7 Panel installer.
# Usage: curl -fsSL https://dn7.cn/start.sh | sh
set -e
SITE="https://dn7.cn"

case "$(uname -m)" in
  x86_64|amd64) ARCH=x86_64 ;;
  aarch64|arm64) ARCH=arm64 ;;
  *) echo "[DN7] unsupported arch: $(uname -m)" >&2; exit 1 ;;
esac

URL="$SITE/api/panel/download?arch=$ARCH"
OUT=dn7-panel

echo "[DN7] downloading DN7 Panel ($ARCH) ..."
if command -v curl >/dev/null 2>&1; then
  curl -fL --progress-bar "$URL" -o "$OUT"
elif command -v wget >/dev/null 2>&1; then
  wget -O "$OUT" "$URL"
else
  echo "[DN7] neither curl nor wget found" >&2; exit 1
fi

chmod +x "$OUT"
echo "[DN7] starting DN7 Panel ..."

# Launch. The panel installs itself, prints the console address + credentials
# to this terminal, then daemonizes — so the disclosure lives in the binary,
# not in this script.
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
