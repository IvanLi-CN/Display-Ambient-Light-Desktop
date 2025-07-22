use axum::{
    http::StatusCode,
    response::Json,
    routing::{get, post, put},
    Router,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::http_server::{ApiResponse, AppState};

/// 创建设备相关路由
pub fn create_routes() -> Router<AppState> {
    Router::new()
    // TODO: 实现设备相关的API端点
    // .route("/boards", get(get_boards))
    // .route("/auto-start", get(get_auto_start_status))
    // .route("/auto-start", put(set_auto_start_status))
}
