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
    // 这里将包含对现有管理器的引用
    // 例如: ambient_light_manager, display_manager 等
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
        api::info::get_app_version,
    ),
    components(
        schemas(ApiResponse<String>, ApiError)
    ),
    tags(
        (name = "health", description = "健康检查相关API"),
        (name = "info", description = "应用信息相关API"),
        (name = "config", description = "配置管理相关API"),
        (name = "led", description = "LED控制相关API"),
        (name = "display", description = "显示器相关API"),
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
    let app_state = AppState {};

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
