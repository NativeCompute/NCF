pub fn unix_now() -> i64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0)
}

fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe - 719468
}

fn num(s: &str) -> Option<i64> {
    if s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    s.parse().ok()
}

pub fn parse_time(s: &str) -> Option<i64> {
    let s = s.trim();
    if s.len() < 19 {
        return None;
    }
    let (date, rest) = s.split_at(10);
    let mut d = date.split('-');
    let (y, m, day) = (num(d.next()?)?, num(d.next()?)?, num(d.next()?)?);
    let rest = rest.strip_prefix(['T', ' '])?;
    let (clock, tail) = rest.split_at(8);
    let mut c = clock.split(':');
    let (h, mi, sec) = (num(c.next()?)?, num(c.next()?)?, num(c.next()?)?);
    let tail = tail.trim_start_matches(|ch: char| ch == '.' || ch.is_ascii_digit());
    let offset = match tail {
        "" | "Z" | "z" => 0,
        t => {
            let sign = if t.starts_with('-') { -1 } else if t.starts_with('+') { 1 } else { return None };
            let t = &t[1..];
            let (oh, om) = match t.split_once(':') {
                Some((a, b)) => (num(a)?, num(b)?),
                None if t.len() == 4 => (num(&t[..2])?, num(&t[2..])?),
                None => (num(t)?, 0),
            };
            sign * (oh * 3600 + om * 60)
        }
    };
    if !(1..=12).contains(&m) || !(1..=31).contains(&day) || h > 23 || mi > 59 || sec > 60 {
        return None;
    }
    Some(days_from_civil(y, m, day) * 86400 + h * 3600 + mi * 60 + sec - offset)
}

pub fn parse_http_date(s: &str) -> Option<i64> {
    let parts: Vec<&str> = s.split_whitespace().collect();
    if parts.len() < 5 {
        return None;
    }
    let months = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
    let m = months.iter().position(|x| *x == parts[2])? as i64 + 1;
    let iso = format!("{}-{m:02}-{:0>2}T{}Z", parts[3], parts[1], parts[4]);
    parse_time(&iso)
}

pub fn format_time(t: i64) -> String {
    let days = t.div_euclid(86400);
    let secs = t.rem_euclid(86400);
    let z = days + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z", secs / 3600, secs % 3600 / 60, secs % 60)
}

pub fn ago(seconds: i64) -> String {
    let s = seconds.max(0);
    if s < 60 {
        format!("{s}s ago")
    } else if s < 3600 {
        format!("{}m ago", s / 60)
    } else if s < 86400 {
        format!("{}h ago", s / 3600)
    } else {
        format!("{}d ago", s / 86400)
    }
}
