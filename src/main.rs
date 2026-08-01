use std::collections::BTreeSet;
use std::io::Read;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use tiny_http::{Header, Method, Request, Response, Server};

use habitsync::analytics::{self, completion_rate, current_streak, longest_streak};
use habitsync::model::{Checkin, Habit};
use habitsync::store::Store;
use habitsync::sync;

const MAX_BODY: u64 = 1 << 20;

#[derive(Deserialize)]
struct SyncRequest {
    #[serde(default)]
    since: u64,
    #[serde(default)]
    habits: Vec<Habit>,
    #[serde(default)]
    checkins: Vec<Checkin>,
}

#[derive(Serialize)]
struct SyncResponse {
    cursor: u64,
    habits: Vec<Habit>,
    checkins: Vec<Checkin>,
}

#[derive(Serialize)]
struct HabitStats {
    id: String,
    name: String,
    current_streak: u32,
    longest_streak: u32,
    total: usize,
    rate_30d: f64,
}

fn main() {
    let port: u16 = arg("--port")
        .or_else(|| std::env::var("HABITSYNC_PORT").ok())
        .and_then(|p| p.parse().ok())
        .unwrap_or(8787);
    let data = arg("--data")
        .or_else(|| std::env::var("HABITSYNC_DATA").ok())
        .filter(|s| !s.is_empty())
        .map(PathBuf::from);
    let host = arg("--host")
        .or_else(|| std::env::var("HABITSYNC_HOST").ok())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "127.0.0.1".to_string());

    let store = match Store::open(data) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    };
    let server = match Server::http((host.as_str(), port)) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("could not bind {host}:{port}: {e}");
            std::process::exit(1);
        }
    };
    println!("habitsync listening on http://{host}:{port}");

    for request in server.incoming_requests() {
        handle(request, &store);
    }
}

fn handle(mut request: Request, store: &Store) {
    let method = request.method().clone();
    let url = request.url().to_string();
    let mut body = String::new();
    let _ = request.as_reader().take(MAX_BODY + 1).read_to_string(&mut body);

    let (status, payload) = if body.len() as u64 > MAX_BODY {
        (413, error_json("request body too large"))
    } else {
        route(store, &method, &url, &body)
    };
    let header = Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap();
    let response = Response::from_string(payload)
        .with_status_code(status)
        .with_header(header);
    let _ = request.respond(response);
}

fn route(store: &Store, method: &Method, url: &str, body: &str) -> (u16, String) {
    let path = url.split('?').next().unwrap_or(url);
    match (method, path) {
        (Method::Get, "/health") => (200, "{\"status\":\"ok\"}".to_string()),
        (Method::Post, "/sync") => match serde_json::from_str::<SyncRequest>(body) {
            Ok(req) => (200, serde_json::to_string(&do_sync(store, req)).unwrap()),
            Err(e) => (400, error_json(&e.to_string())),
        },
        (Method::Get, "/habits") => {
            let habits = store.read(|d| {
                let mut list: Vec<&Habit> = d.habits.values().filter(|h| !h.deleted).collect();
                list.sort_by(|a, b| a.name.cmp(&b.name));
                serde_json::to_string(&list).unwrap()
            });
            (200, habits)
        }
        (Method::Get, "/analytics") => (200, analytics_json(store)),
        _ => (404, error_json("not found")),
    }
}

fn do_sync(store: &Store, req: SyncRequest) -> SyncResponse {
    let pushed_habits: Vec<String> = req.habits.iter().map(|h| h.id.clone()).collect();
    let pushed_checkins: Vec<String> = req.checkins.iter().map(|c| c.id.clone()).collect();
    store.write(|d| {
        sync::apply(&mut d.habits, req.habits, &mut d.counter);
        sync::apply(&mut d.checkins, req.checkins, &mut d.counter);

        let mut habits = sync::changes_since(&d.habits, req.since);
        for id in pushed_habits {
            if let Some(current) = d.habits.get(&id) {
                if current.seq <= req.since && !habits.iter().any(|h| h.id == id) {
                    habits.push(current.clone());
                }
            }
        }

        let mut checkins = sync::changes_since(&d.checkins, req.since);
        for id in pushed_checkins {
            if let Some(current) = d.checkins.get(&id) {
                if current.seq <= req.since && !checkins.iter().any(|c| c.id == id) {
                    checkins.push(current.clone());
                }
            }
        }

        SyncResponse {
            habits,
            checkins,
            cursor: d.counter,
        }
    })
}

fn analytics_json(store: &Store) -> String {
    let today = (now_secs() / 86_400) as i64;
    let stats = store.read(|d| {
        let mut out = Vec::new();
        for habit in d.habits.values().filter(|h| !h.deleted) {
            let days: BTreeSet<i64> = d
                .checkins
                .values()
                .filter(|c| !c.deleted && c.habit_id == habit.id)
                .filter_map(|c| analytics::parse_date(&c.date))
                .collect();
            out.push(HabitStats {
                id: habit.id.clone(),
                name: habit.name.clone(),
                current_streak: current_streak(&days, today),
                longest_streak: longest_streak(&days),
                total: days.len(),
                rate_30d: completion_rate(&days, today, 30),
            });
        }
        out
    });
    serde_json::to_string(&stats).unwrap()
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn error_json(message: &str) -> String {
    serde_json::json!({ "error": message }).to_string()
}

fn arg(name: &str) -> Option<String> {
    let args: Vec<String> = std::env::args().collect();
    args.iter()
        .position(|a| a == name)
        .and_then(|i| args.get(i + 1))
        .cloned()
}
