use axum::{
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};

use tracing_subscriber::prelude::*;

use std::sync::LazyLock;

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
    // initialize tracing

    //#[cfg(not(debug_assertions))]
    //tracing_subscriber::fmt::init();

    //#[cfg(debug_assertions)]
    //{
    //    tracing_subscriber::registry()
    //        .with(tracing_subscriber::fmt::layer())
    //        //.with("debug,tokio::net=info")
    //        .with(
    //            tracing_subscriber::EnvFilter::from_default_env()
    //                .add_directive("debug,tokio::net=info".parse()?),
    //        )
    //        .init();
    //}

    #[cfg(not(debug_assertions))]
    tracing_subscriber::fmt::ini();

    #[cfg(debug_assertions)]
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    // build our application with a route
    let app = Router::new().route("/", get(root));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind(("0.0.0.0", *PORT)).await?;
    tracing::info!("Listening on {:?}", listener.local_addr()?);
    axum::serve(listener, app).await?;

    Ok(())
}

// basic handler that responds with a static string
async fn root() -> &'static str {
    "Hello, World!"
}
