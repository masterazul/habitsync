# habitsync

[![CI](https://github.com/masterazul/habitsync/actions/workflows/ci.yml/badge.svg)](https://github.com/masterazul/habitsync/actions/workflows/ci.yml)
[![Security](https://github.com/masterazul/habitsync/actions/workflows/security.yml/badge.svg)](https://github.com/masterazul/habitsync/actions/workflows/security.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
![Rust](https://img.shields.io/badge/rust-2021-orange.svg)

Built for the app that still has to work on a plane. The client writes to its own storage
with no network at all; when the signal comes back it pushes everything in one POST and
receives only what changed since its last cursor.

Ties are where these things usually go wrong. Two taps in the same second are routine on a
phone, so a delete beats an edit on an identical timestamp — undoing something should not
lose to the thing it was undoing.

There is no database to operate. State is a JSON file, written atomically, and the server
ships as one static binary of about 500 KB.

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

## Hardening

The pipeline is scoped, not just green.

- Actions run from a commit digest. A tag can be moved; a digest cannot.
- Workflows declare `permissions: contents: read`, so `GITHUB_TOKEN` has nothing to write with.
- `cargo audit --deny warnings` runs on every push and again weekly. Dependabot watches the
  crates and the pinned digests alike.
- `#![forbid(unsafe_code)]` on both crate roots — enforced by the compiler, not by habit.
- Release builds keep overflow checks. Paired with `panic = "abort"`, an overflow stops the
  process instead of wrapping into a wrong answer.
- `gitleaks` reads the full history on every push.

## License

MIT
