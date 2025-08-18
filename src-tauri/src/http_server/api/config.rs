use axum::{
    http::StatusCode,
    response::Json,
    routing::{get, post, put},
    Router,
};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
    ambient_light::{self, Border, ColorCalibration, LedStripConfigGroup, LedType},
    http_server::{ApiResponse, AppState},
    language_manager::LanguageManager,
    user_preferences::{UIPreferences, UserPreferences, UserPreferencesManager, WindowPreferences},
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

/// LED灯带反转请求
#[derive(Deserialize, ToSchema)]
pub struct ReverseLedStripRequest {
    /// 显示器ID
    pub display_id: u32,
    /// 边框
    pub border: Border,
}

/// 主题更新请求
#[derive(Deserialize, ToSchema)]
pub struct UpdateThemeRequest {
    /// 主题名称
    pub theme: String,
}

/// 更新视图缩放请求
#[derive(Deserialize, ToSchema)]
pub struct UpdateViewScaleRequest {
    /// 缩放比例
    pub scale: f64,
}

/// 用户偏好设置更新请求
#[derive(Deserialize, ToSchema)]
pub struct UpdateUserPreferencesRequest {
    /// 用户偏好设置
    pub preferences: UserPreferences,
}

/// 窗口偏好设置更新请求
#[derive(Deserialize, ToSchema)]
pub struct UpdateWindowPreferencesRequest {
    /// 窗口偏好设置
    pub window_prefs: WindowPreferences,
}

/// UI偏好设置更新请求
#[derive(Deserialize, ToSchema)]
pub struct UpdateUIPreferencesRequest {
    /// UI偏好设置
    pub ui_prefs: UIPreferences,
}

/// 全局颜色校准更新请求
#[derive(Deserialize, ToSchema)]
pub struct UpdateGlobalColorCalibrationRequest {
    /// 颜色校准设置
    pub calibration: ColorCalibration,
}

/// 语言设置更新请求
#[derive(Deserialize, ToSchema)]
pub struct UpdateLanguageRequest {
    /// 语言代码 (zh-CN, en-US)
    pub language: String,
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
    let config_manager_v2 = ambient_light::ConfigManagerV2::global().await;
    let v2_config = config_manager_v2.get_config().await;

    // 转换为v1格式以保持API兼容性
    let adapter = ambient_light::PublisherAdapter::new(config_manager_v2.get_display_registry());
    match adapter.convert_v2_to_v1_config(&v2_config).await {
        Ok(v1_config) => Ok(Json(ApiResponse::success(v1_config))),
        Err(e) => {
            log::error!("Failed to convert v2 config to v1: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
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
    Json(v1_config): Json<LedStripConfigGroup>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    let config_manager_v2 = ambient_light::ConfigManagerV2::global().await;
    let adapter = ambient_light::PublisherAdapter::new(config_manager_v2.get_display_registry());

    // 转换v1配置为v2格式
    match adapter.convert_v1_to_v2_config(&v1_config).await {
        Ok(v2_config) => match config_manager_v2.update_config(v2_config).await {
            Ok(_) => Ok(Json(ApiResponse::success(
                "LED strip configs updated successfully".to_string(),
            ))),
            Err(e) => {
                log::error!("Failed to update LED strip configs: {e}");
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        },
        Err(e) => {
            log::error!("Failed to convert v1 config to v2: {e}");
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
    let config_manager_v2 = ambient_light::ConfigManagerV2::global().await;

    // 获取当前配置
    let mut v2_config = config_manager_v2.get_config().await;

    // 通过显示器注册管理器获取内部ID
    let display_registry = config_manager_v2.get_display_registry();
    let internal_id = match display_registry
        .get_internal_id_by_display_id(request.display_id)
        .await
    {
        Ok(id) => id,
        Err(e) => {
            log::error!(
                "Failed to get internal ID for display {}: {}",
                request.display_id,
                e
            );
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // 查找并更新对应的灯带
    let mut found = false;
    for strip in &mut v2_config.strips {
        if strip.display_internal_id == internal_id && strip.border == request.border {
            let new_len = (strip.len as i32 + request.delta_len as i32).max(0) as usize;
            strip.len = new_len;
            found = true;
            break;
        }
    }

    if !found {
        log::error!(
            "LED strip not found for display {} border {:?}",
            request.display_id,
            request.border
        );
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    // 重新生成mappers
    v2_config.generate_mappers();

    // 保存配置
    match config_manager_v2.update_config(v2_config).await {
        Ok(_) => Ok(Json(ApiResponse::success(
            "LED strip length updated successfully".to_string(),
        ))),
        Err(e) => {
            log::error!("Failed to update LED strip length: {e}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// 反转LED灯带
#[utoipa::path(
    put,
    path = "/api/v1/config/led-strips/reverse",
    request_body = ReverseLedStripRequest,
    responses(
        (status = 200, description = "反转LED灯带成功", body = ApiResponse<String>),
        (status = 404, description = "未找到指定的LED灯带", body = ApiResponse<String>),
        (status = 500, description = "反转失败", body = ApiResponse<String>),
    ),
    tag = "config"
)]
pub async fn reverse_led_strip(
    Json(request): Json<ReverseLedStripRequest>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    let config_manager = ambient_light::ConfigManager::global().await;

    match config_manager
        .reverse_led_strip_part(request.display_id, request.border)
        .await
    {
        Ok(_) => {
            log::info!(
                "LED strip reversed successfully: display_id={}, border={:?}",
                request.display_id,
                request.border
            );
            Ok(Json(ApiResponse::success(
                "LED strip reversed successfully".to_string(),
            )))
        }
        Err(e) => {
            log::error!("Failed to reverse LED strip: {e}");
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
            log::error!("Failed to update theme: {e}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// 更新LED灯带类型
#[utoipa::path(
    put,
    path = "/api/v1/config/led-strips/type",
    request_body = UpdateLedStripTypeRequest,
    responses(
        (status = 200, description = "更新LED灯带类型成功", body = ApiResponse<String>),
        (status = 500, description = "更新失败", body = ApiResponse<String>),
    ),
    tag = "config"
)]
pub async fn update_led_strip_type(
    Json(request): Json<UpdateLedStripTypeRequest>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    let config_manager_v2 = ambient_light::ConfigManagerV2::global().await;

    // 获取当前配置
    let mut v2_config = config_manager_v2.get_config().await;

    // 通过显示器注册管理器获取内部ID
    let display_registry = config_manager_v2.get_display_registry();
    let internal_id = match display_registry
        .get_internal_id_by_display_id(request.display_id)
        .await
    {
        Ok(id) => id,
        Err(e) => {
            log::error!(
                "Failed to get internal ID for display {}: {}",
                request.display_id,
                e
            );
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // 查找并更新对应的灯带
    let mut found = false;
    for strip in &mut v2_config.strips {
        if strip.display_internal_id == internal_id && strip.border == request.border {
            strip.led_type = request.led_type;
            found = true;
            break;
        }
    }

    if !found {
        log::error!(
            "LED strip not found for display {} border {:?}",
            request.display_id,
            request.border
        );
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    // 重新生成mappers
    v2_config.generate_mappers();

    // 保存配置
    match config_manager_v2.update_config(v2_config).await {
        Ok(_) => Ok(Json(ApiResponse::success(
            "LED strip type updated successfully".to_string(),
        ))),
        Err(e) => {
            log::error!("Failed to update LED strip type: {e}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// 获取夜间模式主题启用状态
#[utoipa::path(
    get,
    path = "/api/v1/config/night-mode-theme-enabled",
    responses(
        (status = 200, description = "获取夜间模式主题启用状态成功", body = ApiResponse<bool>),
    ),
    tag = "config"
)]
pub async fn get_night_mode_theme_enabled() -> Result<Json<ApiResponse<bool>>, StatusCode> {
    let preferences_manager = UserPreferencesManager::global().await;
    let enabled = preferences_manager.get_night_mode_theme_enabled().await;
    Ok(Json(ApiResponse::success(enabled)))
}

/// 获取夜间模式主题
#[utoipa::path(
    get,
    path = "/api/v1/config/night-mode-theme",
    responses(
        (status = 200, description = "获取夜间模式主题成功", body = ApiResponse<String>),
    ),
    tag = "config"
)]
pub async fn get_night_mode_theme() -> Result<Json<ApiResponse<String>>, StatusCode> {
    let preferences_manager = UserPreferencesManager::global().await;
    let theme = preferences_manager.get_night_mode_theme().await;
    Ok(Json(ApiResponse::success(theme)))
}

/// 获取当前语言设置
#[utoipa::path(
    get,
    path = "/api/v1/config/current-language",
    responses(
        (status = 200, description = "获取当前语言成功", body = ApiResponse<String>),
    ),
    tag = "config"
)]
pub async fn get_current_language() -> Result<Json<ApiResponse<String>>, StatusCode> {
    let language_manager = LanguageManager::global().await;
    let language = language_manager.get_language().await;
    Ok(Json(ApiResponse::success(language)))
}

/// 设置当前语言
#[utoipa::path(
    put,
    path = "/api/v1/config/current-language",
    request_body = UpdateLanguageRequest,
    responses(
        (status = 200, description = "设置语言成功", body = ApiResponse<String>),
        (status = 500, description = "设置失败", body = ApiResponse<String>),
    ),
    tag = "config"
)]
pub async fn set_current_language(
    Json(request): Json<UpdateLanguageRequest>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    let language_manager = LanguageManager::global().await;

    match language_manager
        .set_language(request.language.clone())
        .await
    {
        Ok(_) => {
            log::info!("Language set to: {}", request.language);
            Ok(Json(ApiResponse::success(
                "Language set successfully".to_string(),
            )))
        }
        Err(e) => {
            log::error!("Failed to set language: {e}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// 获取主题
#[utoipa::path(
    get,
    path = "/api/v1/config/theme",
    responses(
        (status = 200, description = "获取主题成功", body = ApiResponse<String>),
    ),
    tag = "config"
)]
pub async fn get_theme() -> Result<Json<ApiResponse<String>>, StatusCode> {
    let preferences_manager = UserPreferencesManager::global().await;
    let preferences = preferences_manager.get_preferences().await;
    Ok(Json(ApiResponse::success(preferences.ui.theme)))
}

/// 获取视图缩放
#[utoipa::path(
    get,
    path = "/api/v1/config/view-scale",
    responses(
        (status = 200, description = "获取视图缩放成功", body = ApiResponse<f64>),
    ),
    tag = "config"
)]
pub async fn get_view_scale() -> Result<Json<ApiResponse<f64>>, StatusCode> {
    let preferences_manager = UserPreferencesManager::global().await;
    let preferences = preferences_manager.get_preferences().await;
    Ok(Json(ApiResponse::success(preferences.ui.view_scale)))
}

/// 更新视图缩放
#[utoipa::path(
    put,
    path = "/api/v1/config/view-scale",
    request_body = UpdateViewScaleRequest,
    responses(
        (status = 200, description = "更新视图缩放成功", body = ApiResponse<String>),
        (status = 500, description = "更新失败", body = ApiResponse<String>),
    ),
    tag = "config"
)]
pub async fn update_view_scale(
    Json(request): Json<UpdateViewScaleRequest>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    let preferences_manager = UserPreferencesManager::global().await;
    match preferences_manager.update_view_scale(request.scale).await {
        Ok(_) => Ok(Json(ApiResponse::success(
            "View scale updated successfully".to_string(),
        ))),
        Err(e) => {
            log::error!("Failed to update view scale: {e}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// 更新全局颜色校准
#[utoipa::path(
    put,
    path = "/api/v1/config/global-color-calibration",
    request_body = UpdateGlobalColorCalibrationRequest,
    responses(
        (status = 200, description = "更新全局颜色校准成功", body = ApiResponse<String>),
        (status = 500, description = "更新失败", body = ApiResponse<String>),
    ),
    tag = "config"
)]
pub async fn update_global_color_calibration(
    Json(request): Json<UpdateGlobalColorCalibrationRequest>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    let config_manager_v2 = ambient_light::ConfigManagerV2::global().await;
    match config_manager_v2
        .update_color_calibration(request.calibration)
        .await
    {
        Ok(_) => Ok(Json(ApiResponse::success(
            "Global color calibration updated successfully".to_string(),
        ))),
        Err(e) => {
            log::error!("Failed to update global color calibration: {e}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// 更新用户偏好设置
#[utoipa::path(
    put,
    path = "/api/v1/config/user-preferences",
    request_body = UpdateUserPreferencesRequest,
    responses(
        (status = 200, description = "更新用户偏好设置成功", body = ApiResponse<String>),
        (status = 500, description = "更新失败", body = ApiResponse<String>),
    ),
    tag = "config"
)]
pub async fn update_user_preferences(
    Json(request): Json<UpdateUserPreferencesRequest>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    let preferences_manager = UserPreferencesManager::global().await;
    match preferences_manager
        .update_preferences(request.preferences)
        .await
    {
        Ok(_) => Ok(Json(ApiResponse::success(
            "User preferences updated successfully".to_string(),
        ))),
        Err(e) => {
            log::error!("Failed to update user preferences: {e}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// 更新窗口偏好设置
#[utoipa::path(
    put,
    path = "/api/v1/config/window-preferences",
    request_body = UpdateWindowPreferencesRequest,
    responses(
        (status = 200, description = "更新窗口偏好设置成功", body = ApiResponse<String>),
        (status = 500, description = "更新失败", body = ApiResponse<String>),
    ),
    tag = "config"
)]
pub async fn update_window_preferences(
    Json(request): Json<UpdateWindowPreferencesRequest>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    let preferences_manager = UserPreferencesManager::global().await;
    match preferences_manager
        .update_window_preferences(request.window_prefs)
        .await
    {
        Ok(_) => Ok(Json(ApiResponse::success(
            "Window preferences updated successfully".to_string(),
        ))),
        Err(e) => {
            log::error!("Failed to update window preferences: {e}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// 更新UI偏好设置
#[utoipa::path(
    put,
    path = "/api/v1/config/ui-preferences",
    request_body = UpdateUIPreferencesRequest,
    responses(
        (status = 200, description = "更新UI偏好设置成功", body = ApiResponse<String>),
        (status = 500, description = "更新失败", body = ApiResponse<String>),
    ),
    tag = "config"
)]
pub async fn update_ui_preferences(
    Json(request): Json<UpdateUIPreferencesRequest>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    let preferences_manager = UserPreferencesManager::global().await;
    match preferences_manager
        .update_ui_preferences(request.ui_prefs)
        .await
    {
        Ok(_) => Ok(Json(ApiResponse::success(
            "UI preferences updated successfully".to_string(),
        ))),
        Err(e) => {
            log::error!("Failed to update UI preferences: {e}");
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
        .route("/led-strips/type", put(update_led_strip_type))
        .route("/led-strips/reverse", put(reverse_led_strip))
        .route("/user-preferences", get(get_user_preferences))
        .route("/user-preferences", put(update_user_preferences))
        .route("/window-preferences", put(update_window_preferences))
        .route("/ui-preferences", put(update_ui_preferences))
        .route("/theme", get(get_theme))
        .route("/theme", put(update_theme))
        .route("/view-scale", get(get_view_scale))
        .route("/view-scale", put(update_view_scale))
        .route(
            "/global-color-calibration",
            put(update_global_color_calibration),
        )
        .route(
            "/night-mode-theme-enabled",
            get(get_night_mode_theme_enabled),
        )
        .route("/night-mode-theme", get(get_night_mode_theme))
        .route(
            "/current-language",
            get(get_current_language).put(set_current_language),
        )
}
