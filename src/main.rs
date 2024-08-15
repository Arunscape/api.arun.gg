use {
    axum::{
        http::StatusCode,
        response::IntoResponse,
        routing::{get, post},
        Json, Router,
    },
    rand::prelude::*,
    serde::{Deserialize, Serialize},
    serde_json::json,
    std::sync::LazyLock,
    tracing_subscriber::prelude::*,
};

static PORT: LazyLock<u16> = LazyLock::new(|| {
    if let Ok(s) = std::env::var("API_ARUN_GG_PORT") {
        if let Ok(port) = s.parse() {
            return port;
        }
    }

    let default = 3000;
    tracing::warn!(
        "No value provided for API_ARUN_GG_PORT. Defaulting to :{}",
        default
    );
    default
});

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    #[cfg(not(debug_assertions))]
    tracing_subscriber::fmt::init();

    #[cfg(debug_assertions)]
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let app = Router::new()
        .route("/", get(root))
        .route("/coin", get(flip_a_coin))
        .route("/random_number", get(random_number))
        .route("/random_colour", get(random_colour));
    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind(("0.0.0.0", *PORT)).await?;
    tracing::info!("Listening on {:?}", listener.local_addr()?);
    axum::serve(listener, app).await?;

    Ok(())
}

async fn root() -> &'static str {
    "Hello, World!"
}

async fn flip_a_coin() -> &'static str {
    let coin = {
        let mut rng = rand::thread_rng();
        rng.gen::<bool>()
    };

    if coin {
        "heads"
    } else {
        "tails"
    }
}

async fn random_number() -> impl IntoResponse {
    let mut rng = rand::thread_rng();
    rng.gen::<u128>().to_string()
}

async fn random_colour() -> impl IntoResponse {
    let mut rng = rand::thread_rng();

    let r: u8 = rng.gen();
    let g: u8 = rng.gen();
    let b: u8 = rng.gen();
    let a: u8 = rng.gen();
    let a_div = (a as f32 / 255.0 * 100.0).round() / 100.0;

    let rgba_str = format!("rgba({}, {}, {}, {})", r, g, b, a_div);
    let hex = format!("#{:02x}{:02x}{:02x}{:02x}", r, g, b, a);

    let res = json!({
        "rgba": {"r": r, "g": g, "b": b, "a": a_div.to_string()},
        "rgba_str": rgba_str,
        "hex": hex,
    });

    Json(res)
}
