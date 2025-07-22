use axum::{
    http::StatusCode,
    response::Json,
    routing::{get, post, put},
    Router,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::http_server::{ApiResponse, AppState};

/// 创建显示器相关路由
pub fn create_routes() -> Router<AppState> {
    Router::new()
    // TODO: 实现显示器相关的API端点
    // .route("/", get(get_displays))
    // .route("/info", get(list_display_info))
    // .route("/:id/colors", get(get_display_colors))
}
