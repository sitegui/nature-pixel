use crate::cell_color::CellColor;
use crate::map::Map;
use crate::web_error::WebError;
use axum::extract::{Query, State};
use axum::Json;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

#[derive(Debug, Deserialize)]
pub struct Request {
    x_index: usize,
    y_index: usize,
    color: CellColor,
}

#[derive(Debug, Serialize)]
pub struct Response {
    version_id: String,
}

pub async fn set_cell_color(
    Query(request): Query<Request>,
    State(map): State<Arc<RwLock<Map>>>,
) -> Result<Json<Response>, WebError> {
    let mut map = map.write().unwrap();
    map.set_cell_color(request.x_index, request.y_index, request.color)
        .map_err(WebError::bad_request)?;

    Ok(Json(Response {
        version_id: map.version_id().to_string(),
    }))
}
