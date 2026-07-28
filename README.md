# World's Toughest Row

A live progress tracker for a row across the Atlantic, from La Gomera in the Canary Islands to Antigua in the Caribbean.
Log the distance you've rowed and watch your boat crawl along the ~4,800 km trade-wind route on a full-screen
map, with running stats, a history log, and a finish-line celebration. The project is inspired by the annual
[World's Toughest Row](https://worldstoughestrow.com/) event.

Rows can be logged by hand or captured automatically from a Concept2 erg with a small background agent that connects to 
your rowing machine over Bluetooth and posts each session for you. Either way the map updates live, with no refresh.

The tracker ships as a **single Rust binary** that serves the JSON API and embeds the built React UI. Run it, 
then open it in your browser.

## Technology stack

- **Backend:** Rust, [axum](https://github.com/tokio-rs/axum), SQLite (via `rusqlite`), Server-Sent Events for live updates.
- **Frontend:** React + TypeScript + Vite, [MapLibre GL](https://maplibre.org/).
- **Erg agent:** Rust, [btleplug](https://github.com/deviceplug/btleplug) for Bluetooth, `reqwest` for posting rows.
- **Shared types:** Rust structs are the single source of truth. [`ts-rs`](https://github.com/Aleph-Alpha/ts-rs)
  generates the matching TypeScript definitions.

## Project layout

```
crates/
  api-types/       Shared DTOs (source of the generated TS bindings)
  storage/         The Storage trait that provides an abstraction layer over the database
  storage-sqlite/  SQLite implementation of the Storage trait
  route/           Route geometry + progress math
  server/          axum HTTP server that embeds the built UI
  erg-agent/       Background agent that logs rows from a Concept2 PM5 over Bluetooth
web/               React + Vite frontend
```

## Running it

### Development (hot reload)

Two terminals:

```sh
# 1) API server on :4800
cargo run -p server

# 2) Vite dev server on :5173 (proxies /api to :4800)
cd web && npm install && npm run dev
```

Then browse **http://localhost:5173**.

### Production (single binary)

Build the UI, then run the release server, which serves everything on `:4800`:

```sh
cd web && npm run build
cargo run --release -p server
```

Then browse http://localhost:4800.

## Automatic erg capture

The `erg-agent` binary logs rows from a Concept2 PM5 monitor with no manual entry. It waits for the erg to wake
(the monitor advertises over Bluetooth once you start pulling), follows the cumulative distance, and posts the total
to the server once you stop. Then it lets the monitor go back to sleep and waits for the next row.

Run it alongside the server, on a machine with Bluetooth that is in range of the erg:

```sh
cargo run -p erg-agent
```

It posts to the same `POST /api/entries` endpoint the web form uses, so captured rows show up in the history and on
the map immediately. If the server is not reachable, a row is skipped and logged, and the manual form stays available
as a fallback.

## API

| Method   | Path            | Description                                             |
|----------|-----------------|---------------------------------------------------------|
| `GET`    | `/api/progress` | Current position, trail, and stats.                     |
| `POST`   | `/api/entries`  | Log a row (defaults to 500 m) and returns progress.     |
| `GET`    | `/api/entries`  | List all logged entries (newest first).                 |
| `DELETE` | `/api/entries`  | Reset and clear all entries.                            |
| `GET`    | `/api/events`   | Server-Sent Events stream of state snapshots (live UI). |

## Regenerating TypeScript types

After changing any `#[derive(TS)]` type in `api-types`, regenerate the bindings in `web/src/bindings/`:

```sh
cargo test -p api-types
```
