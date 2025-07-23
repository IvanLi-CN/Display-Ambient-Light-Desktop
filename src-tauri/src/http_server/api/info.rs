use axum::{
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
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

/// 页面信息报告请求
#[derive(Deserialize, ToSchema)]
pub struct ReportPageRequest {
    /// 页面信息
    pub page_info: String,
}

/// 导航请求
#[derive(Deserialize, ToSchema)]
pub struct NavigateRequest {
    /// 页面路径
    pub page: String,
}

/// 显示器配置导航请求
#[derive(Deserialize, ToSchema)]
pub struct NavigateDisplayConfigRequest {
    /// 显示器ID
    pub display_id: String,
}

/// 外部URL打开请求
#[derive(Deserialize, ToSchema)]
pub struct OpenUrlRequest {
    /// URL地址
    pub url: String,
}

/// 报告当前页面信息
#[utoipa::path(
    post,
    path = "/api/v1/info/current-page",
    request_body = ReportPageRequest,
    responses(
        (status = 200, description = "页面信息报告成功", body = ApiResponse<String>),
    ),
    tag = "info"
)]
pub async fn report_current_page(
    Json(request): Json<ReportPageRequest>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    log::info!("Current page reported: {}", request.page_info);
    Ok(Json(ApiResponse::success(
        "Page info reported successfully".to_string(),
    )))
}

/// 报告当前页面信息（备用端点）
#[utoipa::path(
    post,
    path = "/api/v1/info/report-page",
    request_body = ReportPageRequest,
    responses(
        (status = 200, description = "页面信息报告成功", body = ApiResponse<String>),
    ),
    tag = "info"
)]
pub async fn report_page(
    Json(request): Json<ReportPageRequest>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    log::info!("Page reported: {}", request.page_info);
    Ok(Json(ApiResponse::success(
        "Page reported successfully".to_string(),
    )))
}

/// 导航到指定页面
#[utoipa::path(
    post,
    path = "/api/v1/info/navigate",
    request_body = NavigateRequest,
    responses(
        (status = 200, description = "导航成功", body = ApiResponse<String>),
    ),
    tag = "info"
)]
pub async fn navigate_to_page(
    Json(request): Json<NavigateRequest>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    log::info!("Navigation requested to page: {}", request.page);

    // 在HTTP API模式下，导航通常由前端处理
    // 这里只是记录导航请求，实际导航由前端JavaScript处理
    Ok(Json(ApiResponse::success(
        "Navigation request logged successfully".to_string(),
    )))
}

/// 导航到显示器配置页面
#[utoipa::path(
    post,
    path = "/api/v1/info/navigate-display-config",
    request_body = NavigateDisplayConfigRequest,
    responses(
        (status = 200, description = "导航到显示器配置页面成功", body = ApiResponse<String>),
    ),
    tag = "info"
)]
pub async fn navigate_to_display_config(
    Json(request): Json<NavigateDisplayConfigRequest>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    log::info!(
        "Navigation requested to display config for display: {}",
        request.display_id
    );

    // 在HTTP API模式下，导航通常由前端处理
    Ok(Json(ApiResponse::success(
        "Display config navigation request logged successfully".to_string(),
    )))
}

/// 打开外部URL
#[utoipa::path(
    post,
    path = "/api/v1/info/open-url",
    request_body = OpenUrlRequest,
    responses(
        (status = 200, description = "外部URL打开成功", body = ApiResponse<String>),
    ),
    tag = "info"
)]
pub async fn open_external_url(
    Json(request): Json<OpenUrlRequest>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    log::info!("External URL open requested: {}", request.url);

    // 在HTTP API模式下，记录URL打开请求
    // 实际的URL打开通常由前端JavaScript处理
    Ok(Json(ApiResponse::success(
        "External URL open request logged successfully".to_string(),
    )))
}

/// 打开外部URL（备用端点）
#[utoipa::path(
    post,
    path = "/api/v1/info/open-external-url",
    request_body = OpenUrlRequest,
    responses(
        (status = 200, description = "外部URL打开成功", body = ApiResponse<String>),
    ),
    tag = "info"
)]
pub async fn open_external_url_alt(
    Json(request): Json<OpenUrlRequest>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    // 调用主要的外部URL打开函数
    open_external_url(Json(request)).await
}

/// 创建信息相关路由
pub fn create_routes() -> Router<AppState> {
    Router::new()
        .route("/version", get(get_app_version))
        .route("/system", get(get_system_info))
        .route("/current-page", post(report_current_page))
        .route("/report-page", post(report_page))
        .route("/navigate", post(navigate_to_page))
        .route("/navigate-display-config", post(navigate_to_display_config))
        .route("/open-url", post(open_external_url))
        .route("/open-external-url", post(open_external_url_alt))
}
