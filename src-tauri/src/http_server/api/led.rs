use axum::{
    http::StatusCode,
    response::Json,
    routing::{get, post, put},
    Router,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::http_server::{ApiResponse, AppState};

/// 创建LED控制相关路由
pub fn create_routes() -> Router<AppState> {
    Router::new()
    // TODO: 实现LED控制相关的API端点
    // .route("/colors", post(send_colors))
    // .route("/test", post(start_test_effect))
    // .route("/test", delete(stop_test_effect))
    // .route("/mode", get(get_data_send_mode))
    // .route("/mode", put(set_data_send_mode))
}
