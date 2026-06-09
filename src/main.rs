//! Digital Network 7 (dn7.cn) website + API.
//!
//! Ships as a SINGLE self-contained binary: the built frontend (`frontend/dist`)
//! is embedded via `rust-embed`, and this axum server serves three things:
//!
//!   * `/`, `/assets/*`, SPA routes  -> the embedded React site
//!   * `/api/*`                       -> JSON API (DN7 Panel release metadata)
//!   * `/start.sh`                    -> the one-line installer for DN7 Panel
//!
//! Downloads are presented under the dn7.cn brand. For this first version the
//! actual binaries still come from the existing distribution origin
//! (`UPSTREAM`); the proxy here lets the public URL be `dn7.cn` today and become
//! the full origin later without changing the user-facing command.

mod api;
mod assets;

use std::net::SocketAddr;

use axum::routing::get;
use axum::Router;

/// Existing agent-distribution origin we proxy to for v1 (binaries, version).
/// Kept in one place so it's trivial to flip to a local origin later.
pub const UPSTREAM: &str = "https://api.teaops.dn7.cn";

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
        .route("/api/panel/latest", get(api::panel_latest))
        .route("/api/panel/download", get(api::panel_download))
        // Everything else: the embedded SPA (with client-side routing fallback).
        .fallback(assets::static_handler)
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!(%addr, "dn7 website listening");
    axum::serve(listener, app.into_make_service()).await?;
    Ok(())
}
