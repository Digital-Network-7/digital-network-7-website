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
│   ├── main.rs           # axum server: SPA + /api + /start.sh
│   ├── assets.rs         # serves the embedded frontend (SPA fallback)
│   └── api.rs            # release manifest proxy + installer script
└── frontend/             # Vite + React + TS site (built into frontend/dist)
    ├── src/i18n.ts        # zh-CN / zh-TW / en / ja dictionaries
    └── src/App.tsx
```

## Routes

| Route | Purpose |
|-------|---------|
| `/`, `/assets/*` | Embedded React site |
| `/start.sh` | One-line DN7 Panel installer (`curl -fsSL https://dn7.cn/start.sh \| sh`) |
| `/api/health` | Health check |
| `/api/panel/latest` | DN7 Panel release manifest (version, sizes, sha256, download URLs) |
| `/api/panel/download?arch=x86_64\|arm64` | Streams the binary |

For this first version `/api/panel/*` proxies the existing distribution origin
(`src/main.rs` → `UPSTREAM`); the site is structured to become the full origin
later without changing the public URLs.

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
