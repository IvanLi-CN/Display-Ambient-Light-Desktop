use axum::{
    http::StatusCode,
    response::Json,
    routing::{get, put},
    Router,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    ambient_light_state::{AmbientLightState, AmbientLightStateManager},
    auto_start::AutoStartManager,
    http_server::{ApiResponse, AppState},
    rpc::{BoardInfo, UdpRpc},
};

/// 自动启动设置请求
#[derive(Deserialize, ToSchema)]
pub struct SetAutoStartRequest {
    /// 是否启用自动启动
    pub enabled: bool,
}

/// 获取设备板列表
#[utoipa::path(
    get,
    path = "/api/v1/device/boards",
    responses(
        (status = 200, description = "获取设备板列表成功", body = ApiResponse<Vec<BoardInfo>>),
        (status = 500, description = "获取失败", body = ApiResponse<String>),
    ),
    tag = "device"
)]
pub async fn get_boards() -> Result<Json<ApiResponse<Vec<BoardInfo>>>, StatusCode> {
    match UdpRpc::global().await {
        Ok(udp_rpc) => {
            let boards = udp_rpc.get_boards().await;
            let boards = boards.into_iter().collect::<Vec<_>>();
            Ok(Json(ApiResponse::success(boards)))
        }
        Err(e) => {
            log::error!("Failed to get UDP RPC: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// 获取自动启动状态
#[utoipa::path(
    get,
    path = "/api/v1/device/auto-start",
    responses(
        (status = 200, description = "获取自动启动状态成功", body = ApiResponse<bool>),
        (status = 500, description = "获取失败", body = ApiResponse<String>),
    ),
    tag = "device"
)]
pub async fn get_auto_start_status() -> Result<Json<ApiResponse<bool>>, StatusCode> {
    match AutoStartManager::is_enabled() {
        Ok(enabled) => Ok(Json(ApiResponse::success(enabled))),
        Err(e) => {
            log::error!("Failed to check auto start status: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// 设置自动启动状态
#[utoipa::path(
    put,
    path = "/api/v1/device/auto-start",
    request_body = SetAutoStartRequest,
    responses(
        (status = 200, description = "设置自动启动状态成功", body = ApiResponse<String>),
        (status = 500, description = "设置失败", body = ApiResponse<String>),
    ),
    tag = "device"
)]
pub async fn set_auto_start_status(
    Json(request): Json<SetAutoStartRequest>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    let result = if request.enabled {
        AutoStartManager::enable()
    } else {
        AutoStartManager::disable()
    };

    match result {
        Ok(_) => Ok(Json(ApiResponse::success(
            "Auto start status updated successfully".to_string(),
        ))),
        Err(e) => {
            log::error!("Failed to set auto start status: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// 获取环境光状态
#[utoipa::path(
    get,
    path = "/api/v1/device/ambient-light-state",
    responses(
        (status = 200, description = "获取环境光状态成功", body = ApiResponse<AmbientLightState>),
        (status = 500, description = "获取失败", body = ApiResponse<String>),
    ),
    tag = "device"
)]
pub async fn get_ambient_light_state() -> Result<Json<ApiResponse<AmbientLightState>>, StatusCode> {
    let state_manager = AmbientLightStateManager::global().await;
    let state = state_manager.get_state().await;
    Ok(Json(ApiResponse::success(state)))
}

/// 创建设备相关路由
pub fn create_routes() -> Router<AppState> {
    Router::new()
        .route("/boards", get(get_boards))
        .route("/auto-start", get(get_auto_start_status))
        .route("/auto-start", put(set_auto_start_status))
        .route("/ambient-light-state", get(get_ambient_light_state))
}
