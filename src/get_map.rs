use crate::cell_color::CellColor;
use crate::config::Config;
use crate::map::Map;
use axum::extract::{Query, State};
use axum::Json;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tokio::time;

#[derive(Debug, Deserialize)]
pub struct Request {
    last_version_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct Response {
    version_id: String,
    size: usize,
    colors: &'static [&'static str],
    cell_color_indexes: Vec<usize>,
}

pub async fn get_map(
    Query(request): Query<Request>,
    State(map): State<Arc<RwLock<Map>>>,
    State(config): State<Arc<Config>>,
) -> Json<Response> {
    let change_notifier;
    {
        let map_lock = map.read().unwrap();

        match request.last_version_id {
            Some(last_version_id) if last_version_id == map_lock.version_id() => {
                // Long pooling: wait for change
            }
            _ => return prepare_response(&map_lock),
        }

        change_notifier = map_lock.change_notifier().clone();
    }

    let _ = time::timeout(
        Duration::from_secs(config.long_pooling_seconds),
        change_notifier.notified(),
    )
    .await;

    prepare_response(&map.read().unwrap())
}

fn prepare_response(map: &Map) -> Json<Response> {
    Json(Response {
        version_id: map.version_id().to_string(),
        size: map.size(),
        colors: CellColor::CSS_STRINGS,
        cell_color_indexes: map
            .cells()
            .iter()
            .map(|cell| cell.color().as_index())
            .collect(),
    })
}
