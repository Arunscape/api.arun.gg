#[allow(dead_code)]
use {
    axum::{Json, Router, extract::Path, response::IntoResponse, routing::get},
    serde_json::json,
    std::sync::LazyLock,
};
use {
    axum::{
        Json, Router,
        extract::{Path, Query},
        http::{Response, StatusCode},
        response::IntoResponse,
        routing::get,
    },
    chrono::prelude::*,
    chrono_tz::Tz,
    serde::{Deserialize, Serialize},
    serde_json::json,
    std::sync::LazyLock,
};

pub fn next() -> Router {
    Router::new().route("/{day}", get(calc_next_day))
}

#[derive(Serialize, Deserialize)]
struct TimezoneQuery {
    tz: Option<String>,
}

async fn calc_next_day(Path(day): Path<String>, tz: Query<TimezoneQuery>) -> impl IntoResponse {
    let tz = match &tz.tz {
        None => Ok(Tz::Canada__Mountain),
        Some(tz) => Tz::from_str_insensitive(tz),
    };

    if tz.is_err() {
        return (StatusCode::BAD_REQUEST, anyhow::anyhow!(tz));
    }

    let now = Utc::now().with_timezone(tz);

    let res = match day.to_lowercase() {
        "monday" => // get next monday
        "tuesday" => // get next
        "wednesday" => // get next 
        "thursday" => // get next 
        "friday" => // get next 
        "saturday" => // get next 
        "sunday" => // get next 
        _ => Err(anyhow::anyhow!(
            "expected a weekday like /next/saturday?tz=America/New_York , /next/sunday?tz=Canada/Eastern"
        )),
    };

    if res.is_err() {
        return (StatusCode::BAD_REQUEST, anyhow::anyhow!(res));
    }

    let res = res.unwrap();

    let res = Json(json! {
        "iso8601": res.format!("YYYY-MM-DD")
    });

    res
}
