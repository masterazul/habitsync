use std::collections::{BTreeMap, BTreeSet};

use habitsync::analytics::{
    completion_rate, current_streak, days_from_civil, longest_streak, parse_date,
};
use habitsync::model::Habit;
use habitsync::store::Store;
use habitsync::sync::{apply, changes_since};

fn habit(id: &str, name: &str, updated: u64) -> Habit {
    Habit {
        id: id.to_string(),
        name: name.to_string(),
        schedule: String::new(),
        updated_at: updated,
        deleted: false,
        seq: 0,
    }
}

#[test]
fn last_write_wins_and_counter_advances() {
    let mut store: BTreeMap<String, Habit> = BTreeMap::new();
    let mut counter = 0;

    assert_eq!(
        apply(
            &mut store,
            vec![habit("a", "run", 10), habit("b", "read", 10)],
            &mut counter
        ),
        2
    );
    assert_eq!(counter, 2);

    assert_eq!(
        apply(&mut store, vec![habit("a", "stale", 5)], &mut counter),
        0
    );
    assert_eq!(store["a"].name, "run");

    assert_eq!(
        apply(&mut store, vec![habit("a", "fresh", 20)], &mut counter),
        1
    );
    assert_eq!(store["a"].name, "fresh");
    assert_eq!(counter, 3);
}

#[test]
fn delta_respects_cursor() {
    let mut store: BTreeMap<String, Habit> = BTreeMap::new();
    let mut counter = 0;
    apply(&mut store, vec![habit("a", "run", 10)], &mut counter);
    apply(&mut store, vec![habit("b", "read", 10)], &mut counter);

    assert_eq!(changes_since(&store, 0).len(), 2);
    let tail = changes_since(&store, 1);
    assert_eq!(tail.len(), 1);
    assert_eq!(tail[0].id, "b");
    assert_eq!(changes_since(&store, 2).len(), 0);
}

#[test]
fn store_persists_across_reopen() {
    let mut path = std::env::temp_dir();
    path.push(format!("habitsync-store-{}.json", std::process::id()));
    let _ = std::fs::remove_file(&path);

    {
        let store = Store::open(Some(path.clone())).unwrap();
        store.write(|d| {
            d.counter = 7;
            d.habits.insert("h1".to_string(), habit("h1", "Read", 3));
        });
    }

    let reopened = Store::open(Some(path.clone())).unwrap();
    reopened.read(|d| {
        assert_eq!(d.counter, 7);
        assert_eq!(d.habits.get("h1").map(|h| h.name.as_str()), Some("Read"));
    });

    let _ = std::fs::remove_file(&path);
}

#[test]
fn parses_dates() {
    assert_eq!(parse_date("2026-06-27"), Some(days_from_civil(2026, 6, 27)));
    assert_eq!(parse_date("2026-13-01"), None);
    assert_eq!(parse_date("garbage"), None);
    assert_eq!(parse_date("2026-02-31"), None);
    assert_eq!(parse_date("2026-02-29"), None);
    assert!(parse_date("2024-02-29").is_some());
}

#[test]
fn computes_streaks_and_rate() {
    let base = days_from_civil(2026, 6, 27);
    let days: BTreeSet<i64> = [base, base - 1, base - 2, base - 5, base - 6]
        .into_iter()
        .collect();

    assert_eq!(current_streak(&days, base), 3);
    assert_eq!(current_streak(&days, base + 2), 0);
    assert_eq!(longest_streak(&days), 3);
    assert!((completion_rate(&days, base, 7) - 5.0 / 7.0).abs() < 1e-9);
}

#[test]
fn tombstone_wins_and_propagates() {
    let mut store: BTreeMap<String, Habit> = BTreeMap::new();
    let mut counter = 0;
    apply(&mut store, vec![habit("a", "run", 10)], &mut counter);

    let mut tomb = habit("a", "run", 20);
    tomb.deleted = true;
    apply(&mut store, vec![tomb], &mut counter);

    assert!(store["a"].deleted);
    let delta = changes_since(&store, 1);
    assert_eq!(delta.len(), 1);
    assert!(delta[0].deleted);
}

#[test]
fn delete_wins_an_exact_timestamp_tie() {
    let mut store: BTreeMap<String, Habit> = BTreeMap::new();
    let mut counter = 0;
    apply(&mut store, vec![habit("a", "run", 500)], &mut counter);

    let mut tomb = habit("a", "run", 500);
    tomb.deleted = true;
    assert_eq!(apply(&mut store, vec![tomb], &mut counter), 1);
    assert!(store["a"].deleted);

    let mut revive = habit("a", "run", 500);
    revive.deleted = false;
    assert_eq!(apply(&mut store, vec![revive], &mut counter), 0);
    assert!(store["a"].deleted);
}

#[test]
fn current_streak_counts_grace_day() {
    let base = days_from_civil(2026, 6, 27);
    let yesterday: BTreeSet<i64> = [base - 1, base - 2].into_iter().collect();
    assert_eq!(current_streak(&yesterday, base), 2);

    let gap: BTreeSet<i64> = [base - 2].into_iter().collect();
    assert_eq!(current_streak(&gap, base), 0);
}
