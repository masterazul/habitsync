use std::collections::BTreeSet;

pub fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let mp = if m > 2 { m - 3 } else { m + 9 };
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe - 719468
}

pub fn parse_date(text: &str) -> Option<i64> {
    let mut parts = text.split('-');
    let y = parts.next()?.parse::<i64>().ok()?;
    let m = parts.next()?.parse::<i64>().ok()?;
    let d = parts.next()?.parse::<i64>().ok()?;
    if parts.next().is_some() || !(1..=12).contains(&m) || !(1..=31).contains(&d) {
        return None;
    }
    Some(days_from_civil(y, m, d))
}

pub fn current_streak(days: &BTreeSet<i64>, today: i64) -> u32 {
    let mut day = if days.contains(&today) {
        today
    } else {
        today - 1
    };
    let mut streak = 0;
    while days.contains(&day) {
        streak += 1;
        day -= 1;
    }
    streak
}

pub fn longest_streak(days: &BTreeSet<i64>) -> u32 {
    let mut best = 0;
    let mut run = 0;
    let mut prev: Option<i64> = None;
    for &day in days {
        run = match prev {
            Some(p) if day == p + 1 => run + 1,
            _ => 1,
        };
        best = best.max(run);
        prev = Some(day);
    }
    best
}

pub fn completion_rate(days: &BTreeSet<i64>, today: i64, window: i64) -> f64 {
    if window <= 0 {
        return 0.0;
    }
    let start = today - window + 1;
    let hits = days.iter().filter(|&&d| d >= start && d <= today).count();
    hits as f64 / window as f64
}
