# Digital Network 7 Website

Official public website for **Digital Network 7 (dn7.cn)** and home of **DN7 Panel**.

Ships as a **single self-contained Rust binary**: the React frontend is built
and embedded into the binary (via `rust-embed`), and an axum server serves the
site, the JSON API, and the one-line installer.

## Layout

```
.
├── Cargo.toml            # Rust backend (single deploy binary)
├── src/
│   ├── main.rs           # axum server: SPA + /api + /admin + /start.sh
│   ├── assets.rs         # serves the embedded frontend (SPA fallback)
│   ├── api.rs            # release manifest + download + CI push + installer
│   ├── store.rs          # release store: pushed binaries + stable selection
│   ├── signing.rs        # Ed25519 verify (the auth for the CI push endpoint)
│   └── admin.rs          # minimal operator backend (pick the stable version)
└── frontend/             # Vite + React + TS site (built into frontend/dist)
    ├── src/i18n.ts        # zh-CN / zh-TW / en / ja dictionaries
    └── src/App.tsx
```

## Release flow

dn7.cn is the **domestic origin** for DN7 Panel and gates a curated **stable**
channel:

1. Panel CI builds + Ed25519-signs each binary and **pushes** it to
   `POST /api/panel/ingest`. The push is authenticated purely by the appended
   signature — the site verifies it against the **same embedded public key the
   panel trusts**, so only the release-key holder can publish here (no shared
   token).
2. The binary is stored verbatim (signature included) and appears in `/admin`
   as a new version.
3. An operator marks one version **stable** in `/admin`. The public
   download/version/installer endpoints then serve only that version. With no
   explicit selection the **newest** pushed version is served; with nothing
   pushed at all the download endpoints return an error.

This only governs the panel's default `dn7` update source. The panel's separate
`github` ("preview") source still tracks the absolute latest GitHub release, so
preview users are unaffected by the stable gate. Release notes (`releases.json`)
are still mirrored from the GitHub release.

## Routes

| Route | Purpose |
|-------|---------|
| `/`, `/assets/*` | Embedded React site |
| `/start.sh` | One-line DN7 Panel installer (`curl -fsSL https://dn7.cn/start.sh \| sh`) |
| `/api/health` | Health check |
| `/api/panel/version?arch=x86_64\|arm64` | Manifest the panel's `dn7` updater reads (stable version) |
| `/api/panel/latest` | Richer stable manifest for the website UI (both arches) |
| `/api/panel/releases` | Changelog index (mirrored from GitHub) |
| `/api/panel/download?arch=x86_64\|arm64` | Streams the stored **stable** binary |
| `POST /api/panel/ingest?version=&arch=` | CI push (auth = appended release signature) |
| `/admin`, `/admin/login`, `/admin/logout`, `/admin/stable` | Operator backend |

## Internationalization

Four languages today (zh-CN, zh-TW, en, ja). The language is resolved
**before first paint** by an inline script in `frontend/index.html`
(saved choice → browser language → English) so there's no flash of the wrong
language. A saved choice always wins on refresh and isn't auto-overridden.
Adding a language = one entry in `frontend/src/i18n.ts` plus its code in
`SUPPORTED`.

## Develop

```sh
# Terminal 1 — backend (serves /api + /start.sh on :8090)
cargo run

# Terminal 2 — frontend dev server (proxies /api to :8090)
cd frontend && npm install && npm run dev
```

## Build (single binary)

```sh
cd frontend && npm ci && npm run build   # produces frontend/dist
cd .. && cargo build --release           # embeds dist → one binary
```

CI (`.github/workflows/release.yml`) does both on every push to `main` and
publishes static musl binaries (x86_64 + arm64) as `1.0.<run_number>`.

## Configuration

- `DN7_PORT` — listen port (default `8090`). Runs plain HTTP behind a TLS
  reverse proxy.
- `DN7_DATA_DIR` — base data dir (default `/var/dn7/website`). Pushed binaries
  live under `<dir>/data/binaries/`; the index is `<dir>/data/store.json`
  (0600). **Persist this directory** — it holds the uploaded releases.
- `DN7_ADMIN_PASSWORD` — password for `/admin`. If unset, a random one is
  generated at startup and printed to the log (read it from the service log).
  Set it to keep the password stable across restarts.
- `DN7_PUBLIC_URL` — public base used to build absolute download URLs in
  manifests (default `https://dn7.cn`).
