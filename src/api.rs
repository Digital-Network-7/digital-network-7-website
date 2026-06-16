//! JSON API + installer for the DN7 website.
//!
//! dn7.cn is the domestic origin for DN7 Panel. Panel CI **pushes** every
//! freshly-built, signed binary to `/api/panel/ingest`; an operator marks one
//! version stable in the admin backend. The public endpoints then serve only
//! that stable version:
//!   * `/start.sh`            -> installer that downloads from dn7.cn
//!   * `/api/panel/version`   -> manifest the panel's (dn7-source) updater reads
//!   * `/api/panel/latest`    -> richer manifest for the website UI
//!   * `/api/panel/download`  -> streams the stored stable binary
//!   * `/api/panel/ingest`    -> CI push (auth = appended release signature)
//!   * `/api/panel/releases`  -> changelog index (still mirrored from GitHub)
//!
//! "Stable" is what the panel's default `dn7` update source consumes; the
//! panel's separate `github` ("preview") source still tracks the absolute
//! latest, so this gate only governs the curated channel.

use axum::body::{Body, Bytes};
use axum::extract::{Query, State};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;
use serde_json::json;

use crate::{store, AppState, GITHUB_REPO};

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

fn no_stable() -> Response {
    (
        StatusCode::NOT_FOUND,
        Json(json!({ "ok": false, "error": "no stable release available yet" })),
    )
        .into_response()
}

/// GET /api/panel/version?arch= — the manifest the panel's self-update `dn7`
/// source consumes: version + absolute download URL + sha256 of the stable
/// build for `arch`.
pub async fn panel_version(State(state): State<AppState>, Query(q): Query<ArchQuery>) -> Response {
    let arch = norm_arch(q.arch.as_deref());
    let store = state.store.read().unwrap();
    let Some(entry) = store.effective_stable() else {
        return no_stable();
    };
    let Some(asset) = entry.arches.get(arch) else {
        return no_stable();
    };
    let base = public_base();
    Json(json!({
        "ok": true,
        "data": {
            "product": "DN7 Panel",
            "version": entry.version,
            "arch": arch,
            "url": format!("{base}/api/panel/download?arch={arch}"),
            "asset": store::asset_name(&entry.version, arch),
            "sha256": asset.sha256,
        }
    }))
    .into_response()
}

/// GET /api/panel/latest — richer manifest for the website UI (both arches).
pub async fn panel_latest(State(state): State<AppState>) -> Response {
    let store = state.store.read().unwrap();
    let Some(entry) = store.effective_stable() else {
        return no_stable();
    };
    let base = public_base();
    let mut downloads = serde_json::Map::new();
    let mut sha256 = serde_json::Map::new();
    for arch in store::ARCHES {
        if let Some(a) = entry.arches.get(arch) {
            downloads.insert(
                arch.to_string(),
                json!(format!("{base}/api/panel/download?arch={arch}")),
            );
            sha256.insert(arch.to_string(), json!(a.sha256));
        }
    }
    Json(json!({
        "ok": true,
        "data": {
            "product": "DN7 Panel",
            "version": entry.version,
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
/// consumes. Still mirrored from the GitHub release asset (reachable via the
/// deterministic `releases/latest/download/` redirect, no api.github.com) — the
/// changelog isn't gated by the stable selection, and GitHub remains the
/// source of release notes.
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

/// GET /api/panel/download?arch= — stream the stored **stable** binary (the
/// file is the binary with its 64-byte Ed25519 signature appended, so the
/// downloading panel re-verifies it against its embedded key).
pub async fn panel_download(State(state): State<AppState>, Query(q): Query<ArchQuery>) -> Response {
    let arch = norm_arch(q.arch.as_deref());
    let (asset, _version) = {
        let store = state.store.read().unwrap();
        let Some(entry) = store.effective_stable() else {
            return (StatusCode::NOT_FOUND, "no stable release available yet").into_response();
        };
        match entry.arches.get(arch) {
            Some(a) => (a.file.clone(), entry.version.clone()),
            None => {
                return (
                    StatusCode::NOT_FOUND,
                    format!("stable release has no {arch} build"),
                )
                    .into_response()
            }
        }
    };
    let bytes = match store::read_binary(&asset) {
        Ok(b) => b,
        Err(e) => {
            tracing::error!("download: stored asset {asset} unreadable: {e}");
            return (StatusCode::INTERNAL_SERVER_ERROR, "binary unavailable").into_response();
        }
    };
    let len = bytes.len();
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/octet-stream")
        .header(header::CONTENT_LENGTH, len)
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{asset}\""),
        )
        .body(Body::from(bytes))
        .unwrap_or_else(|_| (StatusCode::INTERNAL_SERVER_ERROR, "stream error").into_response())
}

#[derive(Debug, Deserialize)]
pub struct IngestQuery {
    pub version: String,
    pub arch: String,
}

/// POST /api/panel/ingest?version=&arch= — receive a freshly-built panel binary
/// from CI. The body is the binary with its 64-byte Ed25519 signature appended;
/// authentication IS that signature: it must verify against the embedded
/// release key (only the key holder can produce an acceptable binary, so no
/// shared token is needed). On success the file is stored verbatim and the
/// version index is updated.
pub async fn panel_ingest(
    State(state): State<AppState>,
    Query(q): Query<IngestQuery>,
    body: Bytes,
) -> Response {
    let version = q.version.trim().trim_start_matches('v').to_string();
    let arch = norm_arch(Some(&q.arch));
    if version.is_empty() || !version.chars().all(|c| c.is_ascii_digit() || c == '.') {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "ok": false, "error": "invalid version" })),
        )
            .into_response();
    }

    // Auth = valid appended release signature over the binary bytes.
    if !crate::signing::verify_appended(&body) {
        tracing::warn!(%version, arch, "ingest: rejected — signature verification failed");
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({ "ok": false, "error": "signature verification failed" })),
        )
            .into_response();
    }

    let file = store::asset_name(&version, arch);
    if let Err(e) = store::write_binary(&file, &body) {
        tracing::error!("ingest: failed to store {file}: {e}");
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "ok": false, "error": "failed to store binary" })),
        )
            .into_response();
    }
    let asset = store::ArchAsset {
        sha256: store::sha256_hex(&body),
        size: body.len() as u64,
        file: file.clone(),
    };
    let sha = asset.sha256.clone();
    {
        let mut store = state.store.write().unwrap();
        if let Err(e) = store.record(&version, arch, asset) {
            tracing::error!("ingest: failed to update index: {e}");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "ok": false, "error": "failed to update index" })),
            )
                .into_response();
        }
    }
    tracing::info!(%version, arch, size = body.len(), "ingest: stored signed binary");
    Json(json!({
        "ok": true,
        "data": { "version": version, "arch": arch, "asset": file, "sha256": sha }
    }))
    .into_response()
}

/// GET /start.sh — the one-line installer for DN7 Panel.
///
///   curl -fsSL https://dn7.cn/start.sh | sh
///
/// Detects the CPU arch and downloads the latest **stable** DN7 Panel from
/// dn7.cn, makes it executable, and runs it.
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
  curl -fL --progress-bar "$URL" -o "$OUT" || {
    echo "[DN7] download failed (no stable release published yet?)" >&2; exit 1; }
elif command -v wget >/dev/null 2>&1; then
  wget -O "$OUT" "$URL" || {
    echo "[DN7] download failed (no stable release published yet?)" >&2; exit 1; }
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
