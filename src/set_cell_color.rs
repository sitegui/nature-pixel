use crate::cell_color::CellColor;
use crate::map::Map;
use crate::monitored_rwlock::MonitoredRwLock;
use crate::point::Point;
use crate::web_error::WebError;
use axum::extract::{Query, State};
use axum::Json;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub struct Request {
    x_index: usize,
    y_index: usize,
    color_index: usize,
}

#[derive(Debug, Serialize)]
pub struct Response {
    version_id: String,
}

pub async fn set_cell_color(
    Query(request): Query<Request>,
    State(map): State<Arc<MonitoredRwLock<Map>>>,
) -> Result<Json<Response>, WebError> {
    let mut map = map.write(module_path!());
    let point = Point::new(request.x_index, request.y_index);
    let color = CellColor::try_from_index(request.color_index)?;
    map.set_cell_color(point, color)
        .map_err(WebError::bad_request)?;

    Ok(Json(Response {
        version_id: map.version_id().to_string(),
    }))
}
