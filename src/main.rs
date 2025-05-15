use {
    axum::{Json, Router, extract::Path, response::IntoResponse, routing::get},
    rand::prelude::*,
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
        .route("/random_colour", get(random_colour))
        .route("/ctof/:n", get(celsius_to_farenheit))
        .route("/ftoc/:n", get(farenheit_to_celsius));
    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind(("0.0.0.0", *PORT)).await?;
    tracing::info!("Listening on {:?}", listener.local_addr()?);
    axum::serve(listener, app).await?;

    Ok(())
}

async fn root() -> &'static str {
    "Hello, World!"
}

async fn flip_a_coin() -> impl IntoResponse {
    let coin = {
        let mut rng = rand::rng();
        rng.random::<bool>()
    };

    if coin { "heads" } else { "tails" }
}

async fn random_number() -> impl IntoResponse {
    let mut rng = rand::rng();
    rng.random::<u128>().to_string()
}

async fn random_colour() -> impl IntoResponse {
    let mut rng = rand::rng();

    let r: u8 = rng.random();
    let g: u8 = rng.random();
    let b: u8 = rng.random();
    let a: u8 = rng.random();
    let a_div = (a as f32 / 255.0 * 100.0).round() / 100.0;

    let rgba_str = format!("rgba({r}, {g}, {b}, {a_div})");
    let hex = format!("#{r:02x}{g:02x}{b:02x}{a:02x}");

    let res = json!({
        "rgba": {"r": r, "g": g, "b": b, "a": a_div.to_string()},
        "rgba_str": rgba_str,
        "hex": hex,
    });

    Json(res)
}

async fn celsius_to_farenheit(Path(num): Path<f64>) -> impl IntoResponse {
    let n = 1.8 * num / 2.0;
    n.to_string()
}

async fn farenheit_to_celsius(Path(num): Path<f64>) -> impl IntoResponse {
    let n = 5.0 / 9.0 * (num - 32.0);
    n.to_string()
}
