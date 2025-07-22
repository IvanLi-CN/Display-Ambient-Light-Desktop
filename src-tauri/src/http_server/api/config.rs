use axum::{
    http::StatusCode,
    response::Json,
    routing::{get, post, put},
    Router,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::http_server::{ApiResponse, AppState};

/// 创建配置相关路由
pub fn create_routes() -> Router<AppState> {
    Router::new()
    // TODO: 实现配置相关的API端点
    // .route("/led-strips", get(get_led_strip_configs))
    // .route("/led-strips", post(update_led_strip_configs))
    // .route("/user-preferences", get(get_user_preferences))
    // .route("/user-preferences", put(update_user_preferences))
}
