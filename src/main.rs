use anyhow::Result;
use axum::{Router, Server};
use std::env;
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() -> Result<()> {
    let port = env::var("PORT")
        .as_deref()
        .unwrap_or("8000")
        .parse::<u16>()?;

    tracing_subscriber::fmt().init();

    let serve_dir = ServeDir::new("web");
    let app = Router::new().fallback_service(serve_dir);

    let addr = ([0, 0, 0, 0], port).try_into()?;
    tracing::info!("Listening on {}", port);
    Server::bind(&addr).serve(app.into_make_service()).await?;

    Ok(())
}
