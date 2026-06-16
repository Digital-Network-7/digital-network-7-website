//! Minimal operator backend — list pushed versions, pick the stable one.
//!
//! Deliberately tiny: no React, no client JS. axum renders a couple of HTML
//! pages directly. Auth is a single shared admin password (env
//! `DN7_ADMIN_PASSWORD`; if unset, a random one is generated at startup and
//! printed to the log, mirroring how the panel discloses its first password)
//! plus an in-memory session cookie.
//!
//! Served over plain HTTP behind the same TLS reverse proxy as the rest of the
//! site; the session cookie is HttpOnly + SameSite=Strict.

use axum::extract::State;
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{Html, IntoResponse, Response};
use axum::Form;
use rand::Rng;
use serde::Deserialize;

use crate::store::{self, Store};
use crate::AppState;

const COOKIE: &str = "dn7_admin";

/// Resolve the admin password at startup: use `DN7_ADMIN_PASSWORD` if set,
/// otherwise generate a random one and log it once (the operator reads it from
/// the service log, like the panel's first-run banner).
pub fn resolve_password() -> String {
    if let Ok(p) = std::env::var("DN7_ADMIN_PASSWORD") {
        let p = p.trim().to_string();
        if !p.is_empty() {
            tracing::info!("admin: using DN7_ADMIN_PASSWORD from the environment");
            return p;
        }
    }
    let pw = random_password();
    tracing::warn!(
        "admin: DN7_ADMIN_PASSWORD not set — generated a random one for this run:\n\
         \n    admin console password: {pw}\n\
         \n  Set DN7_ADMIN_PASSWORD to keep it stable across restarts."
    );
    pw
}

/// A readable random password, e.g. `k7P2-xQ9m-Lf3a`.
fn random_password() -> String {
    const ALPHABET: &[u8] = b"abcdefghijkmnpqrstuvwxyzABCDEFGHJKLMNPQRSTUVWXYZ23456789";
    let mut rng = rand::thread_rng();
    let mut groups = Vec::new();
    for _ in 0..3 {
        let g: String = (0..4)
            .map(|_| ALPHABET[rng.gen_range(0..ALPHABET.len())] as char)
            .collect();
        groups.push(g);
    }
    groups.join("-")
}

/// A random opaque session token (hex).
fn new_token() -> String {
    let bytes: [u8; 24] = rand::random();
    let mut s = String::with_capacity(48);
    for b in bytes {
        use std::fmt::Write;
        let _ = write!(s, "{b:02x}");
    }
    s
}

/// Read the session token from the request's Cookie header.
fn cookie_token(headers: &HeaderMap) -> Option<String> {
    let raw = headers.get(header::COOKIE)?.to_str().ok()?;
    for part in raw.split(';') {
        let part = part.trim();
        if let Some(v) = part.strip_prefix(&format!("{COOKIE}=")) {
            return Some(v.to_string());
        }
    }
    None
}

/// Is this request authenticated?
fn is_authed(state: &AppState, headers: &HeaderMap) -> bool {
    match cookie_token(headers) {
        Some(tok) => state
            .sessions
            .lock()
            .map(|s| s.contains(&tok))
            .unwrap_or(false),
        None => false,
    }
}

#[derive(Debug, Deserialize)]
pub struct LoginForm {
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct StableForm {
    pub version: String,
}

/// GET /admin — dashboard when authed, login form otherwise.
pub async fn index(State(state): State<AppState>, headers: HeaderMap) -> Response {
    if is_authed(&state, &headers) {
        let store = state.store.read().unwrap().clone();
        Html(dashboard_html(&store)).into_response()
    } else {
        Html(login_html(false)).into_response()
    }
}

/// POST /admin/login — check the password, set a session cookie.
pub async fn login(State(state): State<AppState>, Form(form): Form<LoginForm>) -> Response {
    if !constant_time_eq(form.password.as_bytes(), state.admin_password.as_bytes()) {
        return (StatusCode::UNAUTHORIZED, Html(login_html(true))).into_response();
    }
    let token = new_token();
    if let Ok(mut s) = state.sessions.lock() {
        s.insert(token.clone());
    }
    let cookie = format!("{COOKIE}={token}; Path=/admin; HttpOnly; SameSite=Strict; Max-Age=86400");
    (
        StatusCode::SEE_OTHER,
        [
            (header::SET_COOKIE, cookie),
            (header::LOCATION, "/admin".to_string()),
        ],
    )
        .into_response()
}

/// POST /admin/logout — drop the session.
pub async fn logout(State(state): State<AppState>, headers: HeaderMap) -> Response {
    if let Some(tok) = cookie_token(&headers) {
        if let Ok(mut s) = state.sessions.lock() {
            s.remove(&tok);
        }
    }
    let cookie = format!("{COOKIE}=; Path=/admin; HttpOnly; SameSite=Strict; Max-Age=0");
    (
        StatusCode::SEE_OTHER,
        [
            (header::SET_COOKIE, cookie),
            (header::LOCATION, "/admin".to_string()),
        ],
    )
        .into_response()
}

/// POST /admin/stable — set the stable version (must exist in the store).
pub async fn set_stable(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(form): Form<StableForm>,
) -> Response {
    if !is_authed(&state, &headers) {
        return (StatusCode::UNAUTHORIZED, Html(login_html(false))).into_response();
    }
    let version = form.version.trim().trim_start_matches('v').to_string();
    let mut store = state.store.write().unwrap();
    if store.find(&version).is_none() {
        return (
            StatusCode::BAD_REQUEST,
            Html(format!(
                "<p>unknown version: {}</p><p><a href=\"/admin\">back</a></p>",
                html_escape(&version)
            )),
        )
            .into_response();
    }
    store.stable = Some(version.clone());
    if let Err(e) = store.save() {
        tracing::error!("admin: failed to persist stable selection: {e}");
        return (StatusCode::INTERNAL_SERVER_ERROR, "failed to save").into_response();
    }
    tracing::info!(version = %version, "admin: stable version updated");
    (
        StatusCode::SEE_OTHER,
        [(header::LOCATION, "/admin".to_string())],
    )
        .into_response()
}

/// Constant-time byte comparison (avoid leaking the password via timing).
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

const STYLE: &str = "body{font-family:system-ui,-apple-system,Segoe UI,Roboto,sans-serif;background:#0b0f17;color:#e6edf3;margin:0;padding:40px;}\
.card{max-width:880px;margin:0 auto;background:#121826;border:1px solid #1f2937;border-radius:14px;padding:28px 30px;}\
h1{font-size:20px;margin:0 0 4px;}p.sub{color:#9aa4b2;margin:0 0 24px;font-size:14px;}\
table{width:100%;border-collapse:collapse;font-size:13px;}th,td{text-align:left;padding:10px 12px;border-bottom:1px solid #1f2937;vertical-align:top;}\
th{color:#9aa4b2;font-weight:600;}tr:last-child td{border-bottom:none;}\
.tag{display:inline-block;padding:2px 8px;border-radius:999px;font-size:11px;font-weight:600;}\
.tag.stable{background:#0f2a1a;color:#4ade80;border:1px solid #14532d;}\
.tag.eff{background:#0f1f2a;color:#60a5fa;border:1px solid #163a52;}\
.sha{color:#6b7686;font-family:ui-monospace,SFMono-Regular,Menlo,monospace;font-size:11px;}\
button{background:#2563eb;color:#fff;border:0;border-radius:8px;padding:7px 14px;font-size:13px;cursor:pointer;}\
button:hover{background:#1d4ed8;}button.cur{background:#1f2937;color:#6b7686;cursor:default;}\
input{background:#0b0f17;border:1px solid #1f2937;border-radius:8px;color:#e6edf3;padding:10px 12px;font-size:14px;width:100%;box-sizing:border-box;}\
form.login{max-width:320px;margin:60px auto;}form.login button{width:100%;margin-top:14px;}\
.err{color:#f87171;font-size:13px;margin-top:10px;}.muted{color:#6b7686;}.right{text-align:right;}\
a{color:#60a5fa;}";

fn login_html(error: bool) -> String {
    let err = if error {
        "<p class=\"err\">Incorrect password.</p>"
    } else {
        ""
    };
    format!(
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\">\
         <meta name=\"viewport\" content=\"width=device-width,initial-scale=1\">\
         <title>DN7 — Release Admin</title><style>{STYLE}</style></head><body>\
         <form class=\"login\" method=\"post\" action=\"/admin/login\">\
         <h1>DN7 Release Admin</h1>\
         <p class=\"sub\">Sign in to manage panel releases.</p>\
         <input type=\"password\" name=\"password\" placeholder=\"Admin password\" autofocus>\
         {err}<button type=\"submit\">Sign in</button></form></body></html>"
    )
}

fn dashboard_html(store: &Store) -> String {
    let effective = store.effective_stable().map(|e| e.version.clone());
    let selected = store.stable.clone();

    let mut rows = String::new();
    let sorted = store.sorted();
    if sorted.is_empty() {
        rows.push_str(
            "<tr><td colspan=\"5\" class=\"muted\">No versions pushed yet. \
             Panel CI publishes here on each release.</td></tr>",
        );
    }
    for e in &sorted {
        let is_selected = selected.as_deref() == Some(e.version.as_str());
        let is_effective = effective.as_deref() == Some(e.version.as_str());

        let mut tags = String::new();
        if is_selected {
            tags.push_str(" <span class=\"tag stable\">stable</span>");
        }
        if is_effective && !is_selected {
            tags.push_str(" <span class=\"tag eff\">effective (newest)</span>");
        }

        // Arches + sizes + truncated sha.
        let mut arch_cells = String::new();
        for arch in store::ARCHES {
            match e.arches.get(arch) {
                Some(a) => arch_cells.push_str(&format!(
                    "<div><b>{arch}</b> · {} <span class=\"sha\">{}…</span></div>",
                    human_size(a.size),
                    html_escape(&a.sha256[..a.sha256.len().min(16)]),
                )),
                None => arch_cells.push_str(&format!(
                    "<div class=\"muted\">{arch} · <i>missing</i></div>"
                )),
            }
        }

        let action = if is_selected {
            "<button class=\"cur\" disabled>current</button>".to_string()
        } else {
            format!(
                "<form method=\"post\" action=\"/admin/stable\" style=\"margin:0\">\
                 <input type=\"hidden\" name=\"version\" value=\"{v}\">\
                 <button type=\"submit\">Set stable</button></form>",
                v = html_escape(&e.version)
            )
        };

        rows.push_str(&format!(
            "<tr><td><b>v{v}</b>{tags}</td><td>{arches}</td><td class=\"muted\">{date}</td>\
             <td class=\"right\">{action}</td></tr>",
            v = html_escape(&e.version),
            arches = arch_cells,
            date = fmt_date(e.uploaded_at),
        ));
    }

    let eff_note = match &effective {
        Some(v) if selected.as_deref() == Some(v.as_str()) => {
            format!("Serving <b>v{}</b> (operator-selected).", html_escape(v))
        }
        Some(v) => format!(
            "Serving <b>v{}</b> (newest; no version pinned).",
            html_escape(v)
        ),
        None => {
            "Nothing to serve yet — downloads will error until a version is pushed.".to_string()
        }
    };

    format!(
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\">\
         <meta name=\"viewport\" content=\"width=device-width,initial-scale=1\">\
         <title>DN7 — Release Admin</title><style>{STYLE}</style></head><body>\
         <div class=\"card\"><div style=\"display:flex;justify-content:space-between;align-items:baseline\">\
         <h1>DN7 Panel releases</h1>\
         <form method=\"post\" action=\"/admin/logout\" style=\"margin:0\">\
         <button class=\"cur\">Sign out</button></form></div>\
         <p class=\"sub\">{eff_note}</p>\
         <table><thead><tr><th>Version</th><th>Builds</th><th>Pushed</th><th></th></tr></thead>\
         <tbody>{rows}</tbody></table></div></body></html>"
    )
}

fn human_size(bytes: u64) -> String {
    const MIB: f64 = 1024.0 * 1024.0;
    if bytes >= MIB as u64 {
        format!("{:.1} MiB", bytes as f64 / MIB)
    } else {
        format!("{:.0} KiB", bytes as f64 / 1024.0)
    }
}

/// Very small UTC date formatter (YYYY-MM-DD) from unix seconds — avoids a
/// chrono dependency for a single cosmetic field.
fn fmt_date(secs: u64) -> String {
    if secs == 0 {
        return "—".to_string();
    }
    let days = secs / 86_400;
    let (mut y, mut d) = (1970i64, days as i64);
    loop {
        let leap = (y % 4 == 0 && y % 100 != 0) || y % 400 == 0;
        let dy = if leap { 366 } else { 365 };
        if d < dy {
            break;
        }
        d -= dy;
        y += 1;
    }
    let leap = (y % 4 == 0 && y % 100 != 0) || y % 400 == 0;
    let months = [
        31,
        if leap { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    let mut m = 0usize;
    while m < 12 && d >= months[m] {
        d -= months[m];
        m += 1;
    }
    format!("{y:04}-{:02}-{:02}", m + 1, d + 1)
}
