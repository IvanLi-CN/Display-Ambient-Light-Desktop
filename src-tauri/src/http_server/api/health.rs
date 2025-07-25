use axum::{http::StatusCode, response::Json};
use serde::Serialize;
use utoipa::ToSchema;

use crate::http_server::ApiResponse;

/// 健康检查响应
#[derive(Serialize, ToSchema)]
pub struct HealthStatus {
    pub status: String,
    pub timestamp: String,
    pub version: String,
    pub uptime: u64,
}

/// 健康检查接口
#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "服务健康", body = ApiResponse<HealthStatus>),
    ),
    tag = "health"
)]
pub async fn health_check() -> Result<Json<ApiResponse<HealthStatus>>, StatusCode> {
    let health_status = HealthStatus {
        status: "healthy".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime: 0, // TODO: 实现真实的运行时间统计
    };

    Ok(Json(ApiResponse::success(health_status)))
}
