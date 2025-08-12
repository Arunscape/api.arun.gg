#[allow(dead_code)]
use {
    axum::{Json, Router, extract::Path, response::IntoResponse, routing::get},
    axum::{extract::Query, http::StatusCode},
    chrono::{Duration, Weekday, prelude::*},
    chrono_tz::Tz,
    serde::{Deserialize, Serialize},
    serde_json::json,
};

pub fn next() -> Router {
    Router::new()
        .route("/next/{day}", get(calc_next_day))
        .route("/this/{day}", get(calc_this_day))
}

#[derive(Serialize, Deserialize)]
struct TimezoneQuery {
    tz: Option<String>,
}

fn parse_weekday(s: &str) -> Option<Weekday> {
    match s {
        "monday" | "mon" | "m" => Some(Weekday::Mon),
        "tuesday" | "tues" | "tue" | "t" => Some(Weekday::Tue),
        "wednesday" | "wed" | "w" => Some(Weekday::Wed),
        "thursday" | "thurs" | "thur" | "th" | "r" => Some(Weekday::Thu),
        "friday" | "fri" | "f" => Some(Weekday::Fri),
        "saturday" | "sat" => Some(Weekday::Sat),
        "sunday" | "sun" => Some(Weekday::Sun),
        _ => None,
    }
}

fn parse_tz(q: &TimezoneQuery) -> Result<Tz, String> {
    let tz = match q.tz.as_deref() {
        None => Ok(chrono_tz::Canada::Mountain),
        Some(s) => Tz::from_str_insensitive(s),
    };
    tz.map_err(|e| format!("invalid tz: {e}"))
}

#[derive(Copy, Clone)]
enum Which {
    This,
    Next,
}

fn weekday_date_from(base: NaiveDate, target: Weekday, which: Which) -> NaiveDate {
    let wd = base.weekday();
    let delta = (7 + target.num_days_from_monday() as i32 - wd.num_days_from_monday() as i32) % 7;
    let offset = match which {
        Which::This => delta,
        Which::Next => {
            if delta == 0 {
                7
            } else {
                delta
            }
        }
    };
    base + Duration::days(offset as i64)
}

// Build a local datetime from date + time, with DST-safe fallback to midnight.
fn build_local_dt_safe(
    tz: Tz,
    date: NaiveDate,
    hms: (u32, u32, u32),
) -> Result<DateTime<Tz>, String> {
    let (h, m, s) = hms;
    if let Some(dt) = tz
        .with_ymd_and_hms(date.year(), date.month(), date.day(), h, m, s)
        .single()
    {
        return Ok(dt);
    }
    // fallback to midnight if ambiguous or nonexistent
    tz.with_ymd_and_hms(date.year(), date.month(), date.day(), 0, 0, 0)
        .single()
        .ok_or_else(|| {
            "failed to construct datetime in timezone (possible DST transition issue)".to_string()
        })
}

fn formats_payload(day_str: &str, tz: Tz, local_dt: DateTime<Tz>) -> serde_json::Value {
    let utc_dt = local_dt.with_timezone(&Utc);

    // Local formats
    let rfc3339 = local_dt.to_rfc3339();
    let rfc3339_micros = local_dt.to_rfc3339_opts(chrono::SecondsFormat::Micros, true);
    let rfc2822 = local_dt.to_rfc2822();
    let iso_extended = local_dt.format("%Y-%m-%dT%H:%M:%S%:z").to_string();
    let iso_basic = local_dt.format("%Y%m%dT%H%M%S%z").to_string();
    let date_only = local_dt.format("%F").to_string();
    let time_only = local_dt.format("%T").to_string();
    let week_date = local_dt.format("%G-W%V-%u").to_string();
    let ordinal_date = local_dt.format("%Y-%j").to_string();
    let local_with_tzname = local_dt.format("%Y-%m-%d %H:%M:%S %Z%:z").to_string();

    // UTC formats
    let utc_rfc3339 = utc_dt.to_rfc3339();
    let utc_extended = utc_dt.format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let utc_basic = utc_dt.format("%Y%m%dT%H%M%SZ").to_string();
    let utc_rfc2822 = utc_dt.to_rfc2822();

    // Epochs
    let epoch_seconds = utc_dt.timestamp();
    let epoch_millis = utc_dt.timestamp_millis();
    let epoch_micros = utc_dt.timestamp_micros();
    let epoch_nanos = utc_dt.timestamp_nanos_opt().unwrap_or(0);

    json!({
        "input": {
            "weekday": day_str,
            "timezone": tz.name(),
        },
        "local": {
            "rfc3339": rfc3339,
            "rfc3339_micros": rfc3339_micros,
            "rfc2822": rfc2822,
            "iso_extended": iso_extended,
            "iso_basic": iso_basic,
            "date_only": date_only,
            "time_only": time_only,
            "week_date": week_date,
            "ordinal_date": ordinal_date,
            "with_tzname": local_with_tzname,
        },
        "utc": {
            "rfc3339": utc_rfc3339,
            "rfc2822": utc_rfc2822,
            "iso_extended": utc_extended,
            "iso_basic": utc_basic,
        },
        "epoch": {
            "seconds": epoch_seconds,
            "milliseconds": epoch_millis,
            "microseconds": epoch_micros,
            "nanoseconds": epoch_nanos,
        }
    })
}

async fn handle_calc(day: String, tz_q: TimezoneQuery, which: Which) -> impl IntoResponse {
    // TZ
    let tz = match parse_tz(&tz_q) {
        Ok(tz) => tz,
        Err(msg) => return (StatusCode::BAD_REQUEST, Json(json!({ "error": msg }))),
    };

    // Weekday
    let target = match parse_weekday(&day.to_lowercase()) {
        Some(wd) => wd,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": "expected a weekday like /next/saturday?tz=America/New_York or /next/sunday?tz=Canada/Eastern"
                })),
            );
        }
    };

    // Now in tz
    let now_local = Utc::now().with_timezone(&tz);
    let today = now_local.date_naive();

    // Choose date by mode
    let target_date = weekday_date_from(today, target, which);

    // Keep same local time-of-day as now, with DST-safe fallback
    let local_dt = match build_local_dt_safe(
        tz,
        target_date,
        (now_local.hour(), now_local.minute(), now_local.second()),
    ) {
        Ok(dt) => dt,
        Err(msg) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": msg })),
            );
        }
    };

    let body = Json(formats_payload(&day, tz, local_dt));
    (StatusCode::OK, body)
}

async fn calc_next_day(
    Path(day): Path<String>,
    Query(tz): Query<TimezoneQuery>,
) -> impl IntoResponse {
    handle_calc(day, tz, Which::Next).await
}

async fn calc_this_day(
    Path(day): Path<String>,
    Query(tz): Query<TimezoneQuery>,
) -> impl IntoResponse {
    handle_calc(day, tz, Which::This).await
}
