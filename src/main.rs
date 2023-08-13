mod cell;
mod cell_color;
mod config;
mod get_map;
mod map;
mod web_error;

use crate::config::Config;
use crate::get_map::get_map;
use crate::map::Map;
use anyhow::Result;
use axum::extract::FromRef;
use axum::{routing, Router, Server};
use std::sync::Arc;
use tower_http::services::ServeDir;

#[derive(Debug, FromRef, Clone)]
struct State {
    map: Arc<Map>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config::load()?;

    let state = State {
        map: Arc::new(Map::new(config.map_size)),
    };

    tracing_subscriber::fmt().init();

    let serve_dir = ServeDir::new("web");
    let app = Router::new()
        .route("/api/map", routing::get(get_map))
        .fallback_service(serve_dir)
        .with_state(state);

    let addr = ([0, 0, 0, 0], config.port).try_into()?;
    tracing::info!("Listening on {}", config.port);
    Server::bind(&addr).serve(app.into_make_service()).await?;

    Ok(())
}
