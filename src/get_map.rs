use crate::cell_color::CellColor;
use crate::map::Map;
use axum::extract::{Query, State};
use axum::Json;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub struct Request {
    last_version_id: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct Response {
    version_id: u64,
    size: usize,
    colors: &'static [&'static str],
    cell_color_indexes: Vec<usize>,
}

pub async fn get_map(
    Query(request): Query<Request>,
    State(map): State<Arc<Map>>,
) -> Json<Response> {
    tracing::info!("version_id = {:?}", request.last_version_id);

    Json(Response {
        version_id: map.version_id(),
        size: map.size(),
        colors: CellColor::CSS_STRINGS,
        cell_color_indexes: map
            .cells()
            .iter()
            .map(|cell| cell.color().as_index())
            .collect(),
    })
}
