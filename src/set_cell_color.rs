use crate::cell_color::CellColor;
use crate::map::Map;
use crate::web_error::WebError;
use axum::extract::{Query, State};
use serde::Deserialize;
use std::sync::{Arc, RwLock};

#[derive(Debug, Deserialize)]
pub struct Request {
    x_index: usize,
    y_index: usize,
    color: CellColor,
}

pub async fn set_cell_color(
    Query(request): Query<Request>,
    State(map): State<Arc<RwLock<Map>>>,
) -> Result<(), WebError> {
    let mut map = map.write().unwrap();
    map.set_cell_color(request.x_index, request.y_index, request.color)
        .map_err(WebError::bad_request)?;

    Ok(())
}
