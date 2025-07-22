/*!
 * 通用API端点
 * 包含问候、测试等通用功能
 */

use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::http_server::{ApiResponse, AppState};

/// 问候请求
#[derive(Debug, Deserialize, ToSchema)]
pub struct GreetRequest {
    /// 姓名
    pub name: String,
}

/// 问候响应
#[derive(Debug, Serialize, ToSchema)]
pub struct GreetResponse {
    /// 问候消息
    pub message: String,
}

/// 创建通用API路由
pub fn create_routes() -> Router<AppState> {
    Router::new()
        .route("/greet", post(greet))
        .route("/ping", get(ping))
}

/// 问候API
///
/// 接收姓名并返回问候消息
#[utoipa::path(
    post,
    path = "/api/v1/greet",
    tag = "general",
    request_body = GreetRequest,
    responses(
        (status = 200, description = "问候成功", body = ApiResponse<GreetResponse>),
        (status = 400, description = "请求参数错误")
    )
)]
pub async fn greet(
    State(_state): State<AppState>,
    Json(request): Json<GreetRequest>,
) -> Json<ApiResponse<GreetResponse>> {
    let message = format!(
        "Hello, {}! Welcome to Ambient Light Control API.",
        request.name
    );

    let response = GreetResponse { message };

    Json(ApiResponse::success(response))
}

/// Ping API
///
/// 简单的连通性测试
#[utoipa::path(
    get,
    path = "/api/v1/ping",
    tag = "general",
    responses(
        (status = 200, description = "Ping成功", body = ApiResponse<String>),
    )
)]
pub async fn ping(State(_state): State<AppState>) -> Json<ApiResponse<String>> {
    Json(ApiResponse::success("pong".to_string()))
}
