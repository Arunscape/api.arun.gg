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
        .route("/ctof/{n}", get(celsius_to_farenheit))
        .route("/ftoc/{n}", get(farenheit_to_celsius));
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
    libarun::random::flip_a_coin()
}

async fn random_number() -> impl IntoResponse {
    libarun::random::random_number().to_string()
}

async fn random_colour() -> impl IntoResponse {
    let j = libarun::random::random_colour();
    Json(j)
}

async fn celsius_to_farenheit(Path(num): Path<f64>) -> impl IntoResponse {
    libarun::unit_conversion::celsius_to_farenheit(num).to_string()
}

async fn farenheit_to_celsius(Path(num): Path<f64>) -> impl IntoResponse {
    libarun::unit_conversion::farenheit_to_celsius(num).to_string()
}
