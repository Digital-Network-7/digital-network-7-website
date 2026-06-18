//! Digital Network 7 (dn7.cn) website + API.
//!
//! Ships as a SINGLE self-contained binary: the built frontend (`frontend/dist`)
//! is embedded via `rust-embed`, and this axum server serves four things:
//!
//!   * `/`, `/assets/*`, SPA routes  -> the embedded React site
//!   * `/api/*`                       -> JSON API (DN7 Panel release metadata)
//!   * `/admin*`                      -> minimal operator backend (set stable)
//!   * `/start.sh`                    -> the one-line installer for DN7 Panel
//!
//! dn7.cn is the domestic origin: panel CI pushes each signed build to
//! `/api/panel/ingest`, an operator marks one stable in `/admin`, and the public
//! download/version/installer endpoints serve that stable build. Release notes
//! are still mirrored from the DN7 Panel GitHub releases (`GITHUB_REPO`).

mod admin;
mod api;
mod assets;
mod signing;
mod store;

use std::collections::HashSet;
use std::ffi::{OsStr, OsString};
use std::fs::OpenOptions;
use std::net::{SocketAddr, TcpStream};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;

use axum::extract::DefaultBodyLimit;
use axum::routing::{get, post};
use axum::Router;

use store::Store;

/// GitHub `owner/repo` that publishes the DN7 Panel release notes
/// (`releases.json`). The binaries now live here; only the changelog is still
/// mirrored from GitHub. Kept in one place so it's trivial to change later.
pub const GITHUB_REPO: &str = "Digital-Network-7/DN7-Panel";

const DAEMON_CHILD_ENV: &str = "DN7_WEBSITE_DAEMON_CHILD";

#[derive(Clone)]
pub struct AppState {
    /// HTTP client for the GitHub changelog mirror.
    pub http: reqwest::Client,
    /// Release index (versions + stable selection), persisted to disk.
    pub store: Arc<RwLock<Store>>,
    /// Live admin session tokens (in memory; cleared on restart).
    pub sessions: Arc<Mutex<HashSet<String>>>,
    /// The admin console password (from env or generated at startup).
    pub admin_password: Arc<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if maybe_spawn_daemon().await? {
        return Ok(());
    }

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    let port = resolve_port();

    let http = reqwest::Client::builder()
        .user_agent("dn7-website")
        .build()?;

    let store = Store::load();
    tracing::info!(
        versions = store.versions.len(),
        stable = ?store.effective_stable().map(|e| e.version.clone()),
        data_dir = %store::data_root().display(),
        "release store loaded"
    );

    let state = AppState {
        http,
        store: Arc::new(RwLock::new(store)),
        sessions: Arc::new(Mutex::new(HashSet::new())),
        admin_password: Arc::new(admin::resolve_password()),
    };

    let app = Router::new()
        // Installer (kept at the root so the command is short: dn7.cn/start.sh).
        .route("/start.sh", get(api::start_script))
        // JSON API.
        .route("/api/health", get(api::health))
        .route("/api/panel/version", get(api::panel_version))
        .route("/api/panel/latest", get(api::panel_latest))
        .route("/api/panel/releases", get(api::panel_releases))
        .route("/api/panel/download", get(api::panel_download))
        // CI push endpoint (auth = appended release signature). Raise the body
        // limit well past the default 2 MiB — panel binaries are tens of MiB.
        .route(
            "/api/panel/ingest",
            post(api::panel_ingest).layer(DefaultBodyLimit::max(128 * 1024 * 1024)),
        )
        // Minimal operator backend.
        .route("/admin", get(admin::index))
        .route("/admin/login", post(admin::login))
        .route("/admin/logout", post(admin::logout))
        .route("/admin/stable", post(admin::set_stable))
        // Everything else: the embedded SPA (with client-side routing fallback).
        .fallback(assets::static_handler)
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!(%addr, "dn7 website listening");
    axum::serve(listener, app.into_make_service()).await?;
    Ok(())
}

fn resolve_port() -> u16 {
    std::env::var("DN7_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(8090)
}

async fn maybe_spawn_daemon() -> anyhow::Result<bool> {
    let args: Vec<OsString> = std::env::args_os().collect();
    let foreground = args
        .iter()
        .skip(1)
        .any(|a| is_foreground_arg(a.as_os_str()));
    if foreground || std::env::var_os(DAEMON_CHILD_ENV).is_some() {
        return Ok(false);
    }

    let exe = std::env::current_exe()?;
    let child_args = args
        .into_iter()
        .skip(1)
        .filter(|a| !is_daemon_arg(a.as_os_str()) && !is_foreground_arg(a.as_os_str()))
        .collect::<Vec<_>>();

    let log_path = store::data_root().join("logs").join("dn7-website.log");
    if let Some(parent) = log_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let stdout = open_log(&log_path)?;
    let stderr = stdout.try_clone()?;

    let mut child = Command::new(exe)
        .args(child_args)
        .env(DAEMON_CHILD_ENV, "1")
        .stdin(Stdio::null())
        .stdout(Stdio::from(stdout))
        .stderr(Stdio::from(stderr))
        .spawn()?;

    wait_for_daemon(&mut child, resolve_port(), &log_path).await?;

    println!(
        "DN7 Website started in the background.\n  pid: {}\n  log: {}",
        child.id(),
        log_path.display()
    );
    Ok(true)
}

fn is_daemon_arg(arg: &OsStr) -> bool {
    arg == "--daemon" || arg == "-d"
}

fn is_foreground_arg(arg: &OsStr) -> bool {
    arg == "--foreground" || arg == "-f"
}

async fn wait_for_daemon(child: &mut Child, port: u16, log_path: &Path) -> anyhow::Result<()> {
    for _ in 0..30 {
        if let Some(status) = child.try_wait()? {
            anyhow::bail!(
                "dn7 website failed to start in the background (status: {status}). Check {}",
                log_path.display()
            );
        }
        if TcpStream::connect(("127.0.0.1", port)).is_ok() {
            return Ok(());
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    anyhow::bail!(
        "dn7 website is still starting and port {port} is not reachable yet. Check {}",
        log_path.display()
    );
}

fn open_log(path: &Path) -> anyhow::Result<std::fs::File> {
    Ok(OpenOptions::new().create(true).append(true).open(path)?)
}
