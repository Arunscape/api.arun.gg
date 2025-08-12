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
    Router::new().route("/{day}", get(calc_next_day))
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
    let now = now_utc.with_timezone(&tz); // timezone-aware DateTime<Tz>[10]

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
        .single(); // handles ambiguous/nonexistent local times due to DST

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
                    let body = Json(
                        json!({ "error": "failed to construct next datetime in timezone (possible DST transition issue)" }),
                    );
                    return (StatusCode::INTERNAL_SERVER_ERROR, body);
                }
            }
        }
    };

    // Format as ISO-8601
    // Option A: full ISO-8601 with offset using to_rfc3339()[10]
    let iso8601 = next_dt_local.to_rfc3339();

    // Option B: strftime "%+" is equivalent to %Y-%m-%dT%H:%M:%S%.f%:z[1][10]
    // let iso8601 = next_dt_local.format("%+").to_string();

    let body = Json(json!({ "iso8601": iso8601 }));
    (StatusCode::OK, body)
}
