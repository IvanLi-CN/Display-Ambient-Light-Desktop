use axum::{http::StatusCode, response::Json, routing::get, Router};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::http_server::{ApiResponse, AppState};

/// 应用版本信息
#[derive(Serialize, ToSchema)]
pub struct AppVersionInfo {
    pub version: String,
    pub is_dev: bool,
    pub build_time: Option<String>,
    pub git_hash: Option<String>,
}

/// 系统信息
#[derive(Serialize, ToSchema)]
pub struct SystemInfo {
    pub os: String,
    pub arch: String,
    pub hostname: String,
}

/// 获取应用版本信息
#[utoipa::path(
    get,
    path = "/api/v1/info/version",
    responses(
        (status = 200, description = "应用版本信息", body = ApiResponse<AppVersionInfo>),
    ),
    tag = "info"
)]
pub async fn get_app_version() -> Result<Json<ApiResponse<AppVersionInfo>>, StatusCode> {
    let version_info = AppVersionInfo {
        version: env!("CARGO_PKG_VERSION").to_string(),
        is_dev: cfg!(debug_assertions),
        build_time: option_env!("BUILD_TIME").map(|s| s.to_string()),
        git_hash: option_env!("GIT_HASH").map(|s| s.to_string()),
    };

    Ok(Json(ApiResponse::success(version_info)))
}

/// 获取系统信息
#[utoipa::path(
    get,
    path = "/api/v1/info/system",
    responses(
        (status = 200, description = "系统信息", body = ApiResponse<SystemInfo>),
    ),
    tag = "info"
)]
pub async fn get_system_info() -> Result<Json<ApiResponse<SystemInfo>>, StatusCode> {
    let system_info = SystemInfo {
        os: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
        hostname: hostname::get()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string(),
    };

    Ok(Json(ApiResponse::success(system_info)))
}

/// 创建信息相关路由
pub fn create_routes() -> Router<AppState> {
    Router::new()
        .route("/version", get(get_app_version))
        .route("/system", get(get_system_info))
}
