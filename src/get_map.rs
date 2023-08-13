use crate::cell_color::CellColor;
use crate::map::Map;
use axum::extract::State;
use axum::Json;
use serde::Serialize;
use std::sync::Arc;

#[derive(Debug, Serialize)]
pub struct Response {
    size: usize,
    colors: &'static [&'static str],
    cell_colors: Vec<usize>,
}

pub async fn get_map(State(map): State<Arc<Map>>) -> Json<Response> {
    Json(Response {
        size: map.size(),
        colors: CellColor::CSS_STRINGS,
        cell_colors: map
            .cells()
            .iter()
            .map(|cell| cell.color().as_index())
            .collect(),
    })
}
