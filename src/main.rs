//! Digital Network 7 (dn7.cn) website + API.
//!
//! Ships as a SINGLE self-contained binary: the built frontend (`frontend/dist`)
//! is embedded via `rust-embed`, and this axum server serves three things:
//!
//!   * `/`, `/assets/*`, SPA routes  -> the embedded React site
//!   * `/api/*`                       -> JSON API (DN7 Panel release metadata)
//!   * `/start.sh`                    -> the one-line installer for DN7 Panel
//!
//! Downloads are presented under the dn7.cn brand and mirror the DN7 Panel
//! GitHub releases (`GITHUB_REPO`), so the public URL is dn7.cn today and the
//! origin can move without changing the user-facing command.

mod api;
mod assets;

use std::net::SocketAddr;

use axum::routing::get;
use axum::Router;

/// GitHub `owner/repo` that publishes the DN7 Panel release binaries. The site
/// mirrors/proxies these; kept in one place so it's trivial to change later.
pub const GITHUB_REPO: &str = "Digital-Network-7/DN7-Panel";

#[derive(Clone)]
pub struct AppState {
    pub http: reqwest::Client,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    let port: u16 = std::env::var("DN7_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(8090);

    let http = reqwest::Client::builder()
        .user_agent("dn7-website")
        .build()?;
    let state = AppState { http };

    let app = Router::new()
        // Installer (kept at the root so the command is short: dn7.cn/start.sh).
        .route("/start.sh", get(api::start_script))
        // JSON API.
        .route("/api/health", get(api::health))
        .route("/api/panel/version", get(api::panel_version))
        .route("/api/panel/latest", get(api::panel_latest))
        .route("/api/panel/download", get(api::panel_download))
        .route("/api/panel/download.sig", get(api::panel_download_sig))
        // Everything else: the embedded SPA (with client-side routing fallback).
        .fallback(assets::static_handler)
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!(%addr, "dn7 website listening");
    axum::serve(listener, app.into_make_service()).await?;
    Ok(())
}
