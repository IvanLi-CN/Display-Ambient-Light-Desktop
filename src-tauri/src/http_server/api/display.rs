use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
    ambient_light::LedStripConfig,
    display::{DisplayConfig, DisplayManager, DisplayRegistry, DisplayState},
    http_server::{ApiResponse, AppState},
    led_color::LedColor,
    DisplayInfoWrapper, ScreenshotManager,
};

/// 显示器颜色查询参数
#[derive(Deserialize, ToSchema)]
pub struct DisplayColorsQuery {
    /// LED配置（JSON格式）
    pub led_configs: Option<String>,
}

/// 获取所有显示器状态
#[utoipa::path(
    get,
    path = "/api/v1/display",
    responses(
        (status = 200, description = "获取显示器状态成功", body = ApiResponse<Vec<DisplayState>>),
    ),
    tag = "display"
)]
pub async fn get_displays() -> Result<Json<ApiResponse<Vec<DisplayState>>>, StatusCode> {
    let display_manager = DisplayManager::global().await;
    let displays = display_manager.get_displays().await;
    Ok(Json(ApiResponse::success(displays)))
}

/// 获取显示器信息列表
#[utoipa::path(
    get,
    path = "/api/v1/display/info",
    responses(
        (status = 200, description = "获取显示器信息成功", body = ApiResponse<String>),
        (status = 500, description = "获取失败", body = ApiResponse<String>),
    ),
    tag = "display"
)]
pub async fn list_display_info() -> Result<Json<ApiResponse<String>>, StatusCode> {
    match display_info::DisplayInfo::all() {
        Ok(displays) => {
            let displays: Vec<DisplayInfoWrapper> =
                displays.iter().map(DisplayInfoWrapper).collect();
            match serde_json::to_string(&displays) {
                Ok(json_str) => Ok(Json(ApiResponse::success(json_str))),
                Err(e) => {
                    log::error!("Failed to serialize display info: {e}");
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
        }
        Err(e) => {
            log::error!("Failed to get display info: {e}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// 获取所有显示器配置（包括稳定ID信息）
#[utoipa::path(
    get,
    path = "/api/v1/display/configs",
    responses(
        (status = 200, description = "获取显示器配置成功", body = ApiResponse<Vec<DisplayConfig>>),
        (status = 500, description = "获取失败", body = ApiResponse<String>),
    ),
    tag = "display"
)]
pub async fn get_display_configs() -> Result<Json<ApiResponse<Vec<DisplayConfig>>>, StatusCode> {
    match DisplayRegistry::global().await {
        Ok(registry) => {
            let configs = registry.get_all_displays().await;
            Ok(Json(ApiResponse::success(configs)))
        }
        Err(e) => {
            log::error!("Failed to get display registry: {e}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// 获取指定显示器的颜色
#[utoipa::path(
    get,
    path = "/api/v1/display/{display_id}/colors",
    params(
        ("display_id" = u32, Path, description = "显示器ID")
    ),
    responses(
        (status = 200, description = "获取显示器颜色成功", body = ApiResponse<Vec<Vec<LedColor>>>),
        (status = 404, description = "显示器未找到", body = ApiResponse<String>),
        (status = 500, description = "获取失败", body = ApiResponse<String>),
    ),
    tag = "display"
)]
pub async fn get_display_colors(
    Path(display_id): Path<u32>,
    Query(query): Query<DisplayColorsQuery>,
) -> Result<Json<ApiResponse<Vec<Vec<LedColor>>>>, StatusCode> {
    let screenshot_manager = ScreenshotManager::global().await;
    let channels = screenshot_manager.channels.read().await;

    if let Some(rx) = channels.get(&display_id) {
        let rx = rx.read().await;
        let screenshot = rx.borrow().clone();

        // 如果提供了LED配置，使用它；否则使用默认配置
        let colors = if let Some(led_configs_str) = query.led_configs {
            match serde_json::from_str::<Vec<LedStripConfig>>(&led_configs_str) {
                Ok(led_configs) => screenshot.get_colors_by_led_configs(&led_configs).await,
                Err(e) => {
                    log::error!("Failed to parse LED configs: {e}");
                    return Err(StatusCode::BAD_REQUEST);
                }
            }
        } else {
            // 使用默认配置或返回空结果
            Vec::new()
        };

        Ok(Json(ApiResponse::success(colors)))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

/// 创建显示器相关路由
pub fn create_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(get_displays))
        .route("/info", get(list_display_info))
        .route("/configs", get(get_display_configs))
        .route("/:display_id/colors", get(get_display_colors))
}
