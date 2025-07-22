use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::Json,
    routing::{delete, get, post, put},
    Router,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    ambient_light::{self, BorderColors, LedStripConfig},
    http_server::{ApiResponse, AppState},
    led_data_sender::{DataSendMode, LedDataSender},
    led_test_effects,
};

/// LED颜色发送请求
#[derive(Deserialize, ToSchema)]
pub struct SendColorsRequest {
    /// 字节偏移量
    pub offset: u16,
    /// 颜色数据缓冲区
    pub buffer: Vec<u8>,
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
            log::error!("Failed to send colors: {}", e);
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
            log::error!("Failed to send test colors: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
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

/// 创建LED控制相关路由
pub fn create_routes() -> Router<AppState> {
    Router::new()
        .route("/colors", post(send_colors))
        .route("/test-colors", post(send_test_colors_to_board))
        .route("/mode", get(get_data_send_mode))
        .route("/mode", put(set_data_send_mode))
}
