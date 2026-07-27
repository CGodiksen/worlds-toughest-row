# World's Toughest Row

A live progress tracker for a row across the Atlantic, from La Gomera in the Canary Islands to Antigua in the Caribbean.
Log the distance you've rowed and watch your boat crawl along the ~4,800 km trade-wind route on a full-screen
map, with running stats, a history log, and a finish-line celebration.

The whole thing ships as a **single Rust binary** that serves the JSON API and embeds the built React UI. Run it, and it
opens in your browser.

## Technology stack

- **Backend** — Rust, [axum](https://github.com/tokio-rs/axum), SQLite (via `rusqlite`)
- **Frontend** — React + TypeScript + Vite, [MapLibre GL](https://maplibre.org/)
- **Shared types** — Rust structs are the single source of truth. [`ts-rs`](https://github.com/Aleph-Alpha/ts-rs)
  generates the matching TypeScript definitions.

## Project layout

```
crates/
  api-types/       Shared DTOs (source of the generated TS bindings)
  route/           Route geometry + progress math + the Storage trait
  server/          axum HTTP server that embeds the built UI
  storage-sqlite/  SQLite implementation of the Storage trait
web/               React + Vite frontend
```

## Running it

### Development (hot reload)

Two terminals:

```sh
# 1) API server on :8080
cargo run -p server

# 2) Vite dev server on :5173 (proxies /api to :8080)
cd web && npm install && npm run dev
```

Then browse **http://localhost:5173**.

### Production (single binary)

Build the UI, then run the release server. It serves everything on `:8080` and opens your browser:

```sh
cd web && npm run build
cargo run --release -p server
```

## API

| Method   | Path            | Description                                         |
|----------|-----------------|-----------------------------------------------------|
| `GET`    | `/api/progress` | Current position, trail, and stats.                 |
| `POST`   | `/api/entries`  | Log a row (defaults to 500 m) and returns progress. |
| `GET`    | `/api/entries`  | List all logged entries (newest first).             |
| `DELETE` | `/api/entries`  | Reset and clear all entries.                        |

## Regenerating TypeScript types

After changing any `#[derive(TS)]` type in `api-types`, regenerate the bindings in `web/src/bindings/`:

```sh
cargo test -p api-types
```
