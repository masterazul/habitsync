# habitsync

[![CI](https://github.com/masterazul/habitsync/actions/workflows/ci.yml/badge.svg)](https://github.com/masterazul/habitsync/actions/workflows/ci.yml)
[![Security](https://github.com/masterazul/habitsync/actions/workflows/security.yml/badge.svg)](https://github.com/masterazul/habitsync/actions/workflows/security.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
![Rust](https://img.shields.io/badge/rust-2021-orange.svg)

Offline-first backend for a habit tracker. Clients work fully offline and reconcile with
the server through a single delta-sync endpoint; the server also computes streak
analytics. It is a small HTTP service with no database to run — state is an in-memory map
optionally persisted to a JSON file, written atomically so an interrupted save can't
corrupt it.

## Sync model

Every record (habit or check-in) carries `id`, `updated_at` (the client's clock) and a
`deleted` tombstone. The server keeps a monotonic `seq` per change.

`POST /sync` is push + pull in one round trip:

```json
{ "since": 0, "habits": [ ... ], "checkins": [ ... ] }
```

The server applies incoming records with last-write-wins on `updated_at`, then returns
everything changed past `since`:

```json
{ "cursor": 42, "habits": [ ... ], "checkins": [ ... ] }
```

The client stores `cursor` and sends it as the next `since`. Any client (mobile, web, CLI)
implements the same three fields — that is the whole protocol.

## Endpoints

| method | path         | purpose                                  |
|--------|--------------|------------------------------------------|
| GET    | /health      | liveness                                 |
| POST   | /sync        | delta push/pull                          |
| GET    | /habits      | current habits                           |
| GET    | /analytics   | per-habit current/longest streak + rate  |

## Run

```
cargo run --release -- --port 8787 --data habitsync.json
# HABITSYNC_PORT / HABITSYNC_DATA also work; omit --data to stay in-memory
```

It binds to `127.0.0.1` by default — the API is unauthenticated, so exposing it is an
explicit choice. To let mobile/web/CLI clients on your network sync, opt in with
`--host 0.0.0.0` (or `HABITSYNC_HOST`). Request bodies over 1 MiB are rejected with `413`.

```
curl -s localhost:8787/sync -d '{"since":0,"habits":[{"id":"h1","name":"Read","updated_at":1}],"checkins":[{"id":"c1","habit_id":"h1","date":"2026-06-27","updated_at":1}]}'
curl -s localhost:8787/analytics
```

## License

MIT
