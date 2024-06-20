use {
    axum::{
        http::StatusCode,
        response::IntoResponse,
        routing::{get, post},
        Json, Router,
    },
    rand::prelude::*,
    serde::{Deserialize, Serialize},
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
        .route("/random_number", get(random_number));
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
