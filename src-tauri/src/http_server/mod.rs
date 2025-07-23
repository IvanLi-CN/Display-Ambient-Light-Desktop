use axum::{http::Method, routing::get, Router};
use serde::Serialize;
use tower_http::cors::{Any, CorsLayer};
use utoipa::{OpenApi, ToSchema};
use utoipa_swagger_ui::SwaggerUi;

pub mod api;
pub mod websocket;

/// HTTP服务器配置
#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub enable_cors: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 3030,
            enable_cors: true,
        }
    }
}

/// 应用状态，包含所有共享资源
#[derive(Clone)]
pub struct AppState {
    /// WebSocket连接管理器
    pub websocket_manager: websocket::WebSocketManager,
}

/// 标准API响应格式
#[derive(Serialize, ToSchema)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message),
        }
    }
}

/// API错误类型
#[derive(Serialize, ToSchema)]
pub struct ApiError {
    pub code: String,
    pub message: String,
}

impl ApiError {
    pub fn new(code: &str, message: &str) -> Self {
        Self {
            code: code.to_string(),
            message: message.to_string(),
        }
    }
}

/// OpenAPI文档定义
#[derive(OpenApi)]
#[openapi(
    paths(
        api::health::health_check,
        api::general::greet,
        api::general::ping,
        api::info::get_app_version,
        api::info::get_system_info,
        api::info::report_current_page,
        api::info::report_page,
        api::info::navigate_to_page,
        api::info::navigate_to_display_config,
        api::info::open_external_url,
        api::info::open_external_url_alt,
        api::led::send_colors,
        api::led::send_test_colors_to_board,
        api::led::get_data_send_mode,
        api::led::set_data_send_mode,
        api::led::enable_test_mode,
        api::led::disable_test_mode,
        api::led::get_test_mode_status,
        api::led::start_single_display_config,
        api::led::stop_single_display_config,
        api::led::set_active_strip_breathing,
        api::config::get_led_strip_configs,
        api::config::update_led_strip_configs,
        api::config::update_led_strip_length,
        api::config::update_led_strip_type,
        api::config::get_user_preferences,
        api::config::update_user_preferences,
        api::config::update_window_preferences,
        api::config::update_ui_preferences,
        api::config::get_theme,
        api::config::update_theme,
        api::config::get_view_scale,
        api::config::update_view_scale,
        api::config::get_night_mode_theme_enabled,
        api::config::get_night_mode_theme,
        api::config::get_current_language,
        api::display::get_displays,
        api::display::list_display_info,
        api::display::get_display_colors,
        api::device::get_boards,
        api::device::get_auto_start_status,
        api::device::set_auto_start_status,
        api::device::get_ambient_light_state,
    ),
    components(
        schemas(
            ApiResponse<String>,
            ApiError,
            api::general::GreetRequest,
            api::general::GreetResponse
        )
    ),
    tags(
        (name = "health", description = "健康检查相关API"),
        (name = "general", description = "通用API"),
        (name = "info", description = "应用信息相关API"),
        (name = "config", description = "配置管理相关API"),
        (name = "led", description = "LED控制相关API"),
        (name = "display", description = "显示器相关API"),
        (name = "device", description = "设备管理相关API"),
    ),
    info(
        title = "Ambient Light Control API",
        version = "2.0.0-alpha",
        description = "环境光控制应用的HTTP API接口",
        contact(
            name = "Ivan Li",
            url = "https://github.com/IvanLi-CN/Display-Ambient-Light-Desktop"
        )
    )
)]
pub struct ApiDoc;

/// 创建HTTP服务器
pub async fn create_server(config: ServerConfig) -> Result<Router, anyhow::Error> {
    // 获取全局WebSocket事件发布器的WebSocket管理器
    let websocket_publisher = crate::websocket_events::WebSocketEventPublisher::global().await;
    let app_state = AppState {
        websocket_manager: websocket_publisher.get_websocket_manager().clone(),
    };

    // 配置CORS
    let cors = if config.enable_cors {
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
            .allow_headers(Any)
    } else {
        CorsLayer::permissive()
    };

    // 创建路由
    let app = Router::new()
        // 健康检查
        .route("/health", get(api::health::health_check))
        // API v1 路由
        .nest("/api/v1", create_api_routes())
        // WebSocket路由
        .route("/ws", get(websocket::websocket_handler))
        // Swagger UI
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        // 中间件
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .layer(cors)
        .with_state(app_state);

    Ok(app)
}

/// 创建API v1路由
fn create_api_routes() -> Router<AppState> {
    Router::new()
        // 通用API
        .merge(api::general::create_routes())
        // 应用信息
        .nest("/info", api::info::create_routes())
        // 配置管理
        .nest("/config", api::config::create_routes())
        // LED控制
        .nest("/led", api::led::create_routes())
        // 显示器管理
        .nest("/display", api::display::create_routes())
        // 设备管理
        .nest("/device", api::device::create_routes())
}

/// 启动HTTP服务器
pub async fn start_server(config: ServerConfig) -> Result<(), anyhow::Error> {
    let app = create_server(config.clone()).await?;

    let addr = format!("{}:{}", config.host, config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    log::info!("🚀 HTTP服务器启动在 http://{}", addr);
    log::info!("📚 API文档地址: http://{}/swagger-ui", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
