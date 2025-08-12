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

fn next_weekday_from(date: NaiveDate, target: Weekday) -> NaiveDate {
    let wd = date.weekday();
    let delta = (7 + target.num_days_from_monday() as i32 - wd.num_days_from_monday() as i32) % 7;
    let offset = if delta == 0 { 7 } else { delta };
    date + Duration::days(offset as i64)
}

fn this_weekday_from(date: NaiveDate, target: Weekday) -> NaiveDate {
    let wd = date.weekday();
    let delta = (7 + target.num_days_from_monday() as i32 - wd.num_days_from_monday() as i32) % 7;
    date + Duration::days(delta as i64)
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

async fn calc_next_day(
    Path(day): Path<String>,
    Query(tz): Query<TimezoneQuery>,
) -> impl IntoResponse {
    // Parse timezone (default to a specific TZ; adjust to your desired default)
    let tz: Result<Tz, _> = match tz.tz.as_deref() {
        None => Ok(chrono_tz::Canada::Mountain),
        Some(s) => Tz::from_str_insensitive(s),
    };
    let tz = match tz {
        Ok(tz) => tz,
        Err(e) => {
            let body = Json(json!({ "error": format!("invalid tz: {e}") }));
            return (StatusCode::BAD_REQUEST, body);
        }
    };

    // Parse weekday
    let target = match parse_weekday(&day.to_lowercase()) {
        Some(wd) => wd,
        None => {
            let body = Json(json!({
                "error": "expected a weekday like /next/saturday?tz=America/New_York or /next/sunday?tz=Canada/Eastern"
            }));
            return (StatusCode::BAD_REQUEST, body);
        }
    };

    // Compute now in requested tz
    let now_utc = Utc::now();
    let now = now_utc.with_timezone(&tz);

    // Compute next date for target weekday (strictly future)
    let today = now.date_naive();
    let next_date = next_weekday_from(today, target);

    // Keep the same local time-of-day as now
    let next_dt_local = tz
        .with_ymd_and_hms(
            next_date.year(),
            next_date.month(),
            next_date.day(),
            now.hour(),
            now.minute(),
            now.second(),
        )
        .single();

    let next_dt_local = match next_dt_local {
        Some(dt) => dt,
        None => {
            // If ambiguous or nonexistent (DST transition), fall back to midnight
            match tz
                .with_ymd_and_hms(
                    next_date.year(),
                    next_date.month(),
                    next_date.day(),
                    0,
                    0,
                    0,
                )
                .single()
            {
                Some(dt) => dt,
                None => {
                    let body = Json(json!({
                        "error": "failed to construct next datetime in timezone (possible DST transition issue)"
                    }));
                    return (StatusCode::INTERNAL_SERVER_ERROR, body);
                }
            }
        }
    };

    // UTC equivalent
    let next_dt_utc = next_dt_local.with_timezone(&Utc);

    // Common formats
    let rfc3339 = next_dt_local.to_rfc3339(); // 2025-08-13T15:04:05-06:00
    let rfc3339_micros = next_dt_local.to_rfc3339_opts(chrono::SecondsFormat::Micros, true);
    let rfc2822 = next_dt_local.to_rfc2822(); // Wed, 13 Aug 2025 15:04:05 -0600

    // ISO-8601 extended/basic, date-only, time-only
    let iso_extended = next_dt_local.format("%Y-%m-%dT%H:%M:%S%:z").to_string(); // 2025-08-13T15:04:05-06:00
    let iso_basic = next_dt_local.format("%Y%m%dT%H%M%S%z").to_string(); // 20250813T150405-0600
    let date_only = next_dt_local.format("%F").to_string(); // 2025-08-13
    let time_only = next_dt_local.format("%T").to_string(); // 15:04:05 (local time)
    let week_date = next_dt_local.format("%G-W%V-%u").to_string(); // 2025-W33-3
    let ordinal_date = next_dt_local.format("%Y-%j").to_string(); // 2025-225

    // Localized clarity
    let local_with_tzname = next_dt_local.format("%Y-%m-%d %H:%M:%S %Z%:z").to_string(); // 2025-08-13 15:04:05 MDT-06:00

    // UTC variants
    let utc_rfc3339 = next_dt_utc.to_rfc3339(); // Z suffix
    let utc_extended = next_dt_utc.format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let utc_basic = next_dt_utc.format("%Y%m%dT%H%M%SZ").to_string();
    let utc_rfc2822 = next_dt_utc.to_rfc2822();

    // Epoch timestamps
    let epoch_seconds = next_dt_utc.timestamp(); // i64 seconds
    let epoch_millis = next_dt_utc.timestamp_millis(); // i64 milliseconds
    let epoch_micros = next_dt_utc.timestamp_micros(); // i64 microseconds
    let epoch_nanos = next_dt_utc.timestamp_nanos_opt().unwrap_or(0);

    let body = Json(json!({
        "input": {
            "weekday": day,
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
    }));

    (StatusCode::OK, body)
}

async fn calc_this_day(
    Path(day): Path<String>,
    Query(tz): Query<TimezoneQuery>,
) -> impl IntoResponse {
    // Parse timezone (default to a specific TZ; adjust to your desired default)
    let tz: Result<Tz, _> = match tz.tz.as_deref() {
        None => Ok(chrono_tz::Canada::Mountain),
        Some(s) => Tz::from_str_insensitive(s),
    };
    let tz = match tz {
        Ok(tz) => tz,
        Err(e) => {
            let body = Json(json!({ "error": format!("invalid tz: {e}") }));
            return (StatusCode::BAD_REQUEST, body);
        }
    };

    // Parse weekday
    let target = match parse_weekday(&day.to_lowercase()) {
        Some(wd) => wd,
        None => {
            let body = Json(json!({
                "error": "expected a weekday like /next/saturday?tz=America/New_York or /next/sunday?tz=Canada/Eastern"
            }));
            return (StatusCode::BAD_REQUEST, body);
        }
    };

    // Compute now in requested tz
    let now_utc = Utc::now();
    let now = now_utc.with_timezone(&tz);

    // Compute next date for target weekday (strictly future)
    let today = now.date_naive();
    let next_date = this_weekday_from(today, target);

    // Keep the same local time-of-day as now
    let next_dt_local = tz
        .with_ymd_and_hms(
            next_date.year(),
            next_date.month(),
            next_date.day(),
            now.hour(),
            now.minute(),
            now.second(),
        )
        .single();

    let next_dt_local = match next_dt_local {
        Some(dt) => dt,
        None => {
            // If ambiguous or nonexistent (DST transition), fall back to midnight
            match tz
                .with_ymd_and_hms(
                    next_date.year(),
                    next_date.month(),
                    next_date.day(),
                    0,
                    0,
                    0,
                )
                .single()
            {
                Some(dt) => dt,
                None => {
                    let body = Json(json!({
                        "error": "failed to construct next datetime in timezone (possible DST transition issue)"
                    }));
                    return (StatusCode::INTERNAL_SERVER_ERROR, body);
                }
            }
        }
    };

    // UTC equivalent
    let next_dt_utc = next_dt_local.with_timezone(&Utc);

    // Common formats
    let rfc3339 = next_dt_local.to_rfc3339(); // 2025-08-13T15:04:05-06:00
    let rfc3339_micros = next_dt_local.to_rfc3339_opts(chrono::SecondsFormat::Micros, true);
    let rfc2822 = next_dt_local.to_rfc2822(); // Wed, 13 Aug 2025 15:04:05 -0600

    // ISO-8601 extended/basic, date-only, time-only
    let iso_extended = next_dt_local.format("%Y-%m-%dT%H:%M:%S%:z").to_string(); // 2025-08-13T15:04:05-06:00
    let iso_basic = next_dt_local.format("%Y%m%dT%H%M%S%z").to_string(); // 20250813T150405-0600
    let date_only = next_dt_local.format("%F").to_string(); // 2025-08-13
    let time_only = next_dt_local.format("%T").to_string(); // 15:04:05 (local time)
    let week_date = next_dt_local.format("%G-W%V-%u").to_string(); // 2025-W33-3
    let ordinal_date = next_dt_local.format("%Y-%j").to_string(); // 2025-225

    // Localized clarity
    let local_with_tzname = next_dt_local.format("%Y-%m-%d %H:%M:%S %Z%:z").to_string(); // 2025-08-13 15:04:05 MDT-06:00

    // UTC variants
    let utc_rfc3339 = next_dt_utc.to_rfc3339(); // Z suffix
    let utc_extended = next_dt_utc.format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let utc_basic = next_dt_utc.format("%Y%m%dT%H%M%SZ").to_string();
    let utc_rfc2822 = next_dt_utc.to_rfc2822();

    // Epoch timestamps
    let epoch_seconds = next_dt_utc.timestamp(); // i64 seconds
    let epoch_millis = next_dt_utc.timestamp_millis(); // i64 milliseconds
    let epoch_micros = next_dt_utc.timestamp_micros(); // i64 microseconds
    let epoch_nanos = next_dt_utc.timestamp_nanos_opt().unwrap_or(0);

    let body = Json(json!({
        "input": {
            "weekday": day,
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
    }));

    (StatusCode::OK, body)
}
