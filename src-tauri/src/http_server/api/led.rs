use axum::{
    http::StatusCode,
    response::Json,
    routing::{get, post, put},
    Router,
};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
    ambient_light::{self, BorderColors, LedStripConfig},
    http_server::{ApiResponse, AppState},
    led_data_sender::{DataSendMode, LedDataSender},
    led_status_manager::{LedStatusManager, LedStatusStats},
};

/// LED颜色发送请求
#[derive(Deserialize, ToSchema)]
pub struct SendColorsRequest {
    /// 字节偏移量
    pub offset: u16,
    /// 颜色数据缓冲区
    pub buffer: Vec<u8>,
}

/// 校准颜色发送请求
#[derive(Deserialize, ToSchema)]
pub struct SendCalibrationColorRequest {
    /// 红色分量 (0-255)
    pub r: u8,
    /// 绿色分量 (0-255)
    pub g: u8,
    /// 蓝色分量 (0-255)
    pub b: u8,
}

/// 测试颜色发送请求
#[derive(Deserialize, ToSchema)]
pub struct SendTestColorsRequest {
    /// 目标板地址
    pub board_address: String,
    /// 字节偏移量
    pub offset: u16,
    /// 颜色数据缓冲区
    pub buffer: Vec<u8>,
}

/// 单显示器配置发布请求
#[derive(Deserialize, ToSchema)]
pub struct SingleDisplayConfigRequest {
    /// LED灯带配置
    pub strips: Vec<LedStripConfig>,
    /// 边框颜色
    pub border_colors: BorderColors,
}

/// 呼吸灯设置请求
#[derive(Deserialize, ToSchema)]
pub struct BreathingStripRequest {
    /// 显示器ID
    pub display_id: u32,
    /// 边框（可选）
    pub border: Option<String>,
}

/// LED测试效果请求
#[derive(Deserialize, ToSchema)]
pub struct LedTestEffectRequest {
    /// 效果名称
    pub effect_name: String,
    /// 效果参数（可选）
    pub params: Option<serde_json::Value>,
}

/// 启动LED测试效果请求
#[derive(Deserialize, ToSchema)]
pub struct StartLedTestEffectRequest {
    /// 目标板地址
    #[serde(alias = "boardAddress")]
    pub board_address: String,
    /// 效果配置
    #[serde(alias = "effectConfig")]
    pub effect_config: serde_json::Value,
    /// 更新间隔（毫秒）
    #[serde(alias = "updateIntervalMs")]
    pub update_interval_ms: u32,
}

/// 停止LED测试效果请求
#[derive(Deserialize, ToSchema)]
pub struct StopLedTestEffectRequest {
    /// 目标板地址
    #[serde(alias = "boardAddress")]
    pub board_address: String,
    /// LED数量
    #[serde(alias = "ledCount")]
    pub led_count: u32,
    /// LED类型
    #[serde(alias = "ledType")]
    pub led_type: String,
}

/// 数据发送模式设置请求
#[derive(Deserialize, ToSchema)]
pub struct SetDataSendModeRequest {
    /// 数据发送模式
    pub mode: DataSendMode,
}

/// 发送LED颜色数据
#[utoipa::path(
    post,
    path = "/api/v1/led/colors",
    request_body = SendColorsRequest,
    responses(
        (status = 200, description = "颜色数据发送成功", body = ApiResponse<String>),
        (status = 500, description = "发送失败", body = ApiResponse<String>),
    ),
    tag = "led"
)]
pub async fn send_colors(
    Json(request): Json<SendColorsRequest>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    match ambient_light::LedColorsPublisher::send_colors(request.offset, request.buffer).await {
        Ok(_) => Ok(Json(ApiResponse::success(
            "Colors sent successfully".to_string(),
        ))),
        Err(e) => {
            log::error!("Failed to send colors: {e}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// 发送校准颜色数据（推荐用于校准模式）
#[utoipa::path(
    post,
    path = "/api/v1/led/calibration-color",
    request_body = SendCalibrationColorRequest,
    responses(
        (status = 200, description = "校准颜色发送成功", body = ApiResponse<String>),
        (status = 500, description = "发送失败", body = ApiResponse<String>),
    ),
    tag = "led"
)]
pub async fn send_calibration_color(
    Json(request): Json<SendCalibrationColorRequest>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    match ambient_light::LedColorsPublisher::send_calibration_color(request.r, request.g, request.b).await {
        Ok(_) => Ok(Json(ApiResponse::success(
            "Calibration color sent successfully".to_string(),
        ))),
        Err(e) => {
            log::error!("Failed to send calibration color: {e}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// 发送测试颜色到指定板
#[utoipa::path(
    post,
    path = "/api/v1/led/test-colors",
    request_body = SendTestColorsRequest,
    responses(
        (status = 200, description = "测试颜色发送成功", body = ApiResponse<String>),
        (status = 500, description = "发送失败", body = ApiResponse<String>),
    ),
    tag = "led"
)]
pub async fn send_test_colors_to_board(
    Json(request): Json<SendTestColorsRequest>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    let sender = LedDataSender::global().await;
    sender.set_mode(DataSendMode::StripConfig).await;
    sender
        .set_test_target(Some(request.board_address.clone()))
        .await;

    match sender
        .send_complete_led_data(request.offset, request.buffer, "StripConfig")
        .await
    {
        Ok(_) => Ok(Json(ApiResponse::success(
            "Test colors sent successfully".to_string(),
        ))),
        Err(e) => {
            log::error!("Failed to send test colors: {e}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// 获取LED状态统计信息
#[utoipa::path(
    get,
    path = "/api/v1/led/status",
    responses(
        (status = 200, description = "获取LED状态成功", body = ApiResponse<LedStatusStats>),
    ),
    tag = "led"
)]
pub async fn get_led_status() -> Result<Json<ApiResponse<LedStatusStats>>, StatusCode> {
    let status_manager = LedStatusManager::global().await;
    let status = status_manager.get_status().await;
    Ok(Json(ApiResponse::success(status)))
}

/// 获取LED数据发送模式
#[utoipa::path(
    get,
    path = "/api/v1/led/mode",
    responses(
        (status = 200, description = "获取发送模式成功", body = ApiResponse<DataSendMode>),
    ),
    tag = "led"
)]
pub async fn get_data_send_mode() -> Result<Json<ApiResponse<DataSendMode>>, StatusCode> {
    let sender = LedDataSender::global().await;
    let mode = sender.get_mode().await;
    Ok(Json(ApiResponse::success(mode)))
}

/// 设置LED数据发送模式
#[utoipa::path(
    put,
    path = "/api/v1/led/mode",
    request_body = SetDataSendModeRequest,
    responses(
        (status = 200, description = "设置发送模式成功", body = ApiResponse<String>),
    ),
    tag = "led"
)]
pub async fn set_data_send_mode(
    Json(request): Json<SetDataSendModeRequest>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    let sender = LedDataSender::global().await;
    sender.set_mode(request.mode).await;
    log::info!("LED data send mode set to: {}", request.mode);
    Ok(Json(ApiResponse::success(
        "Mode set successfully".to_string(),
    )))
}

/// 启用LED测试模式
#[utoipa::path(
    post,
    path = "/api/v1/led/enable-test-mode",
    responses(
        (status = 200, description = "测试模式启用成功", body = ApiResponse<String>),
        (status = 500, description = "启用失败", body = ApiResponse<String>),
    ),
    tag = "led"
)]
pub async fn enable_test_mode() -> Result<Json<ApiResponse<String>>, StatusCode> {
    let publisher = ambient_light::LedColorsPublisher::global().await;
    publisher.enable_test_mode().await;
    log::info!("LED test mode enabled");
    Ok(Json(ApiResponse::success(
        "Test mode enabled successfully".to_string(),
    )))
}

/// 禁用LED测试模式
#[utoipa::path(
    post,
    path = "/api/v1/led/disable-test-mode",
    responses(
        (status = 200, description = "测试模式禁用成功", body = ApiResponse<String>),
        (status = 500, description = "禁用失败", body = ApiResponse<String>),
    ),
    tag = "led"
)]
pub async fn disable_test_mode() -> Result<Json<ApiResponse<String>>, StatusCode> {
    let publisher = ambient_light::LedColorsPublisher::global().await;
    publisher.disable_test_mode().await;
    log::info!("LED test mode disabled");
    Ok(Json(ApiResponse::success(
        "Test mode disabled successfully".to_string(),
    )))
}

/// 获取LED测试模式状态
#[utoipa::path(
    get,
    path = "/api/v1/led/test-mode-status",
    responses(
        (status = 200, description = "获取测试模式状态成功", body = ApiResponse<bool>),
    ),
    tag = "led"
)]
pub async fn get_test_mode_status() -> Result<Json<ApiResponse<bool>>, StatusCode> {
    let publisher = ambient_light::LedColorsPublisher::global().await;
    let is_active = publisher.is_test_mode_active().await;
    Ok(Json(ApiResponse::success(is_active)))
}

/// 启动单屏配置发布器
#[utoipa::path(
    post,
    path = "/api/v1/led/start-single-display-config",
    request_body = SingleDisplayConfigRequest,
    responses(
        (status = 200, description = "单屏配置发布器启动成功", body = ApiResponse<String>),
        (status = 500, description = "启动失败", body = ApiResponse<String>),
    ),
    tag = "led"
)]
pub async fn start_single_display_config(
    Json(request): Json<SingleDisplayConfigRequest>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    let publisher = ambient_light::LedColorsPublisher::global().await;
    match publisher
        .start_single_display_config_mode(request.strips, request.border_colors)
        .await
    {
        Ok(_) => {
            log::info!("Single display config publisher started");
            Ok(Json(ApiResponse::success(
                "Single display config publisher started successfully".to_string(),
            )))
        }
        Err(e) => {
            log::error!("Failed to start single display config publisher: {e}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// 停止单屏配置发布器
#[utoipa::path(
    post,
    path = "/api/v1/led/stop-single-display-config",
    responses(
        (status = 200, description = "单屏配置发布器停止成功", body = ApiResponse<String>),
        (status = 500, description = "停止失败", body = ApiResponse<String>),
    ),
    tag = "led"
)]
pub async fn stop_single_display_config() -> Result<Json<ApiResponse<String>>, StatusCode> {
    let publisher = ambient_light::LedColorsPublisher::global().await;
    match publisher.stop_single_display_config_mode().await {
        Ok(_) => {
            log::info!("Single display config publisher stopped");
            Ok(Json(ApiResponse::success(
                "Single display config publisher stopped successfully".to_string(),
            )))
        }
        Err(e) => {
            log::error!("Failed to stop single display config publisher: {e}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// 设置活跃灯带用于呼吸效果
#[utoipa::path(
    post,
    path = "/api/v1/led/set-active-strip-breathing",
    request_body = BreathingStripRequest,
    responses(
        (status = 200, description = "设置呼吸效果成功", body = ApiResponse<String>),
        (status = 500, description = "设置失败", body = ApiResponse<String>),
    ),
    tag = "led"
)]
pub async fn set_active_strip_breathing(
    Json(request): Json<BreathingStripRequest>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    let publisher = ambient_light::LedColorsPublisher::global().await;
    let display_id = request.display_id;
    let border = request.border.clone();

    match publisher
        .set_active_strip_for_breathing(display_id, request.border)
        .await
    {
        Ok(_) => {
            log::info!(
                "Active strip for breathing set: display_id={display_id}, border={border:?}"
            );
            Ok(Json(ApiResponse::success(
                "Active strip for breathing set successfully".to_string(),
            )))
        }
        Err(e) => {
            log::error!("Failed to set active strip for breathing: {e}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// 启动LED测试效果
#[utoipa::path(
    post,
    path = "/api/v1/led/start-test-effect",
    request_body = StartLedTestEffectRequest,
    responses(
        (status = 200, description = "启动测试效果成功", body = ApiResponse<String>),
        (status = 500, description = "启动失败", body = ApiResponse<String>),
    ),
    tag = "led"
)]
pub async fn start_led_test_effect(
    Json(request): Json<StartLedTestEffectRequest>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    log::info!(
        "Starting LED test effect for board: {}",
        request.board_address
    );

    // 解析效果配置
    let config: crate::led_test_effects::TestEffectConfig =
        serde_json::from_value(request.effect_config).map_err(|e| {
            log::error!("Failed to parse effect config: {e}");
            StatusCode::BAD_REQUEST
        })?;

    // 获取测试效果管理器并启动效果
    let manager = crate::led_test_effects::LedTestEffectManager::global().await;
    match manager
        .start_test_effect(
            request.board_address.clone(),
            config,
            request.update_interval_ms,
        )
        .await
    {
        Ok(()) => Ok(Json(ApiResponse::success(format!(
            "LED test effect started successfully for board: {}",
            request.board_address
        )))),
        Err(e) => {
            log::error!("Failed to start LED test effect: {e}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// 停止LED测试效果
#[utoipa::path(
    post,
    path = "/api/v1/led/stop-test-effect",
    request_body = StopLedTestEffectRequest,
    responses(
        (status = 200, description = "停止测试效果成功", body = ApiResponse<String>),
        (status = 500, description = "停止失败", body = ApiResponse<String>),
    ),
    tag = "led"
)]
pub async fn stop_led_test_effect(
    Json(request): Json<StopLedTestEffectRequest>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    log::info!(
        "Stopping LED test effect for board: {}",
        request.board_address
    );

    // 获取测试效果管理器并停止效果
    let manager = crate::led_test_effects::LedTestEffectManager::global().await;
    match manager.stop_test_effect(&request.board_address).await {
        Ok(()) => Ok(Json(ApiResponse::success(format!(
            "LED test effect stopped successfully for board: {}",
            request.board_address
        )))),
        Err(e) => {
            log::error!("Failed to stop LED test effect: {e}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// 测试单屏配置模式
#[utoipa::path(
    post,
    path = "/api/v1/led/test-single-display-config",
    responses(
        (status = 200, description = "测试单屏配置模式成功", body = ApiResponse<String>),
        (status = 500, description = "测试失败", body = ApiResponse<String>),
    ),
    tag = "led"
)]
pub async fn test_single_display_config() -> Result<Json<ApiResponse<String>>, StatusCode> {
    log::info!("Testing single display config mode");

    // TODO: 实现单屏配置模式测试逻辑

    Ok(Json(ApiResponse::success(
        "Single display config mode test completed successfully".to_string(),
    )))
}

/// 测试LED数据发送器
#[utoipa::path(
    post,
    path = "/api/v1/led/test-data-sender",
    responses(
        (status = 200, description = "测试LED数据发送器成功", body = ApiResponse<String>),
        (status = 500, description = "测试失败", body = ApiResponse<String>),
    ),
    tag = "led"
)]
pub async fn test_led_data_sender() -> Result<Json<ApiResponse<String>>, StatusCode> {
    log::info!("Testing LED data sender");

    // TODO: 实现LED数据发送器测试逻辑

    Ok(Json(ApiResponse::success(
        "LED data sender test completed successfully".to_string(),
    )))
}

/// 创建LED控制相关路由
pub fn create_routes() -> Router<AppState> {
    Router::new()
        .route("/status", get(get_led_status))
        .route("/colors", post(send_colors))
        .route("/calibration-color", post(send_calibration_color))
        .route("/test-colors", post(send_test_colors_to_board))
        .route("/mode", get(get_data_send_mode))
        .route("/mode", put(set_data_send_mode))
        .route("/enable-test-mode", post(enable_test_mode))
        .route("/disable-test-mode", post(disable_test_mode))
        .route("/test-mode-status", get(get_test_mode_status))
        .route(
            "/start-single-display-config",
            post(start_single_display_config),
        )
        .route(
            "/stop-single-display-config",
            post(stop_single_display_config),
        )
        .route(
            "/set-active-strip-breathing",
            post(set_active_strip_breathing),
        )
        .route("/start-test-effect", post(start_led_test_effect))
        .route("/stop-test-effect", post(stop_led_test_effect))
        .route(
            "/test-single-display-config",
            post(test_single_display_config),
        )
        .route("/test-data-sender", post(test_led_data_sender))
}
