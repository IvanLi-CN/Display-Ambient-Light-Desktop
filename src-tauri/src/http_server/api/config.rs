use axum::{
    extract::Path,
    http::StatusCode,
    response::Json,
    routing::{get, post, put},
    Router,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    ambient_light::{self, Border, ColorCalibration, LedStripConfigGroup, LedType},
    http_server::{ApiResponse, AppState},
    user_preferences::{UserPreferences, UserPreferencesManager},
};

/// LED灯带长度更新请求
#[derive(Deserialize, ToSchema)]
pub struct UpdateLedStripLenRequest {
    /// 显示器ID
    pub display_id: u32,
    /// 边框
    pub border: Border,
    /// LED数量变化（正数增加，负数减少）
    pub delta_len: i8,
}

/// LED灯带类型更新请求
#[derive(Deserialize, ToSchema)]
pub struct UpdateLedStripTypeRequest {
    /// 显示器ID
    pub display_id: u32,
    /// 边框
    pub border: Border,
    /// LED类型
    pub led_type: LedType,
}

/// 主题更新请求
#[derive(Deserialize, ToSchema)]
pub struct UpdateThemeRequest {
    /// 主题名称
    pub theme: String,
}

/// 获取LED灯带配置
#[utoipa::path(
    get,
    path = "/api/v1/config/led-strips",
    responses(
        (status = 200, description = "获取LED灯带配置成功", body = ApiResponse<LedStripConfigGroup>),
        (status = 500, description = "获取失败", body = ApiResponse<String>),
    ),
    tag = "config"
)]
pub async fn get_led_strip_configs() -> Result<Json<ApiResponse<LedStripConfigGroup>>, StatusCode> {
    let config_manager = ambient_light::ConfigManager::global().await;
    let config = config_manager.configs().await;
    Ok(Json(ApiResponse::success(config)))
}

/// 更新LED灯带配置
#[utoipa::path(
    post,
    path = "/api/v1/config/led-strips",
    request_body = LedStripConfigGroup,
    responses(
        (status = 200, description = "更新LED灯带配置成功", body = ApiResponse<String>),
        (status = 500, description = "更新失败", body = ApiResponse<String>),
    ),
    tag = "config"
)]
pub async fn update_led_strip_configs(
    Json(config): Json<LedStripConfigGroup>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    let config_manager = ambient_light::ConfigManager::global().await;
    match config_manager.update(&config).await {
        Ok(_) => Ok(Json(ApiResponse::success(
            "LED strip configs updated successfully".to_string(),
        ))),
        Err(e) => {
            log::error!("Failed to update LED strip configs: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// 更新LED灯带长度
#[utoipa::path(
    put,
    path = "/api/v1/config/led-strips/length",
    request_body = UpdateLedStripLenRequest,
    responses(
        (status = 200, description = "更新LED灯带长度成功", body = ApiResponse<String>),
        (status = 500, description = "更新失败", body = ApiResponse<String>),
    ),
    tag = "config"
)]
pub async fn update_led_strip_length(
    Json(request): Json<UpdateLedStripLenRequest>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    let config_manager = ambient_light::ConfigManager::global().await;
    match config_manager
        .patch_led_strip_len(request.display_id, request.border, request.delta_len)
        .await
    {
        Ok(_) => Ok(Json(ApiResponse::success(
            "LED strip length updated successfully".to_string(),
        ))),
        Err(e) => {
            log::error!("Failed to update LED strip length: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// 获取用户偏好设置
#[utoipa::path(
    get,
    path = "/api/v1/config/user-preferences",
    responses(
        (status = 200, description = "获取用户偏好设置成功", body = ApiResponse<UserPreferences>),
        (status = 500, description = "获取失败", body = ApiResponse<String>),
    ),
    tag = "config"
)]
pub async fn get_user_preferences() -> Result<Json<ApiResponse<UserPreferences>>, StatusCode> {
    let preferences_manager = UserPreferencesManager::global().await;
    let preferences = preferences_manager.get_preferences().await;
    Ok(Json(ApiResponse::success(preferences)))
}

/// 更新主题
#[utoipa::path(
    put,
    path = "/api/v1/config/theme",
    request_body = UpdateThemeRequest,
    responses(
        (status = 200, description = "更新主题成功", body = ApiResponse<String>),
        (status = 500, description = "更新失败", body = ApiResponse<String>),
    ),
    tag = "config"
)]
pub async fn update_theme(
    Json(request): Json<UpdateThemeRequest>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    let preferences_manager = UserPreferencesManager::global().await;
    match preferences_manager.update_theme(request.theme).await {
        Ok(_) => Ok(Json(ApiResponse::success(
            "Theme updated successfully".to_string(),
        ))),
        Err(e) => {
            log::error!("Failed to update theme: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// 创建配置相关路由
pub fn create_routes() -> Router<AppState> {
    Router::new()
        .route("/led-strips", get(get_led_strip_configs))
        .route("/led-strips", post(update_led_strip_configs))
        .route("/led-strips/length", put(update_led_strip_length))
        .route("/user-preferences", get(get_user_preferences))
        .route("/theme", put(update_theme))
}
