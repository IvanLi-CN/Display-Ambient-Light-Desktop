// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod ambient_light;
mod ambient_light_state;
mod auto_start;
mod display;
mod http_server;
mod language_manager;
mod led_color;
mod led_data_processor;
mod led_data_sender;
mod led_preview_state;
mod led_status_manager;
mod led_test_effects;
mod rpc;
mod screen_stream;
mod screenshot;
mod screenshot_manager;
mod user_preferences;
mod volume;
mod websocket_events;

#[cfg(test)]
mod tests;

use display::DisplayManager;
use display_info::DisplayInfo;
use paris::{error, info, warn};
use rpc::UdpRpc;
use screenshot_manager::ScreenshotManager;

use tauri::{
    http::{Request, Response},
    menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager, Runtime,
};
use user_preferences::UserPreferencesManager;

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use volume::VolumeManager;

// Global static variables for LED test effect management
#[allow(dead_code)]
static EFFECT_HANDLE: tokio::sync::OnceCell<Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>> =
    tokio::sync::OnceCell::const_new();
#[allow(dead_code)]
static CANCEL_TOKEN: tokio::sync::OnceCell<
    Arc<RwLock<Option<tokio_util::sync::CancellationToken>>>,
> = tokio::sync::OnceCell::const_new();
#[derive(Serialize, Deserialize)]
#[serde(remote = "DisplayInfo")]
struct DisplayInfoDef {
    pub id: u32,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub rotation: f32,
    pub scale_factor: f32,
    pub is_primary: bool,
    pub frequency: f32,
    #[serde(skip, default = "_default_cg_display")]
    pub raw_handle: core_graphics::display::CGDisplay,
}

fn _default_cg_display() -> core_graphics::display::CGDisplay {
    // Default display for serde deserialization
    core_graphics::display::CGDisplay::main()
}

#[derive(Serialize)]
struct DisplayInfoWrapper<'a>(#[serde(with = "DisplayInfoDef")] &'a DisplayInfo);

// Tauri commands removed - using HTTP API only

#[derive(Serialize)]
#[allow(dead_code)]
struct AppVersion {
    version: String,
    is_dev: bool,
}

// Removed update_display_preferences - feature not implemented

// Removed update_last_visited_page - feature not implemented

async fn update_tray_menu_internal<R: Runtime>(app_handle: &tauri::AppHandle<R>) {
    info!("Updating tray menu...");

    // Get current states
    let state_manager = ambient_light_state::AmbientLightStateManager::global().await;
    let ambient_light_enabled = state_manager.is_enabled().await;
    let auto_start_enabled = auto_start::AutoStartManager::is_enabled().unwrap_or(false);

    info!(
        "Updating menu item states - Ambient light: {}, Auto start: {}",
        ambient_light_enabled, auto_start_enabled
    );

    // Recreate the menu with updated states
    if let Ok(new_menu) = create_tray_menu(app_handle).await {
        if let Some(tray) = app_handle.tray_by_id("main") {
            match tray.set_menu(Some(new_menu)) {
                Ok(_) => info!("Tray menu updated successfully with new checked states"),
                Err(e) => error!("Failed to update tray menu: {}", e),
            }
        } else {
            error!("Tray not found when trying to update menu");
        }
    } else {
        error!("Failed to create new tray menu");
    }
}

async fn create_tray_menu<R: Runtime>(app: &tauri::AppHandle<R>) -> tauri::Result<Menu<R>> {
    let state_manager = ambient_light_state::AmbientLightStateManager::global().await;
    let ambient_light_enabled = state_manager.is_enabled().await;
    let led_preview_manager = led_preview_state::LedPreviewStateManager::global().await;
    let led_preview_enabled = led_preview_manager.is_enabled().await;
    let auto_start_enabled = auto_start::AutoStartManager::is_enabled().unwrap_or(false);

    info!(
        "Creating tray menu - Ambient light: {}, LED preview: {}, Auto start: {}",
        ambient_light_enabled, led_preview_enabled, auto_start_enabled
    );

    // Get current language
    let language_manager = language_manager::LanguageManager::global().await;
    let current_language = language_manager.get_language().await;
    let t = |key: &str| language_manager::TrayTranslations::get_text(&current_language, key);

    // Create menu items
    let ambient_light_item = CheckMenuItem::with_id(
        app,
        "toggle_ambient_light",
        t("ambient_light"),
        true,
        ambient_light_enabled,
        None::<&str>,
    )?;

    let led_preview_item = CheckMenuItem::with_id(
        app,
        "toggle_led_preview",
        t("led_preview"),
        true,
        led_preview_enabled,
        None::<&str>,
    )?;

    let separator1 = PredefinedMenuItem::separator(app)?;

    let info_item = MenuItem::with_id(app, "show_info", t("info"), true, None::<&str>)?;
    let led_config_item = MenuItem::with_id(
        app,
        "show_led_config",
        t("led_configuration"),
        true,
        None::<&str>,
    )?;
    let white_balance_item = MenuItem::with_id(
        app,
        "show_white_balance",
        t("white_balance"),
        true,
        None::<&str>,
    )?;
    let led_test_item = MenuItem::with_id(app, "show_led_test", t("led_test"), true, None::<&str>)?;
    let settings_item = MenuItem::with_id(app, "show_settings", t("settings"), true, None::<&str>)?;

    let separator2 = PredefinedMenuItem::separator(app)?;

    let auto_start_item = CheckMenuItem::with_id(
        app,
        "toggle_auto_start",
        t("auto_start"),
        true,
        auto_start_enabled,
        None::<&str>,
    )?;

    let separator3 = PredefinedMenuItem::separator(app)?;

    let about_item = MenuItem::with_id(app, "show_about", t("about"), true, None::<&str>)?;
    let show_item = MenuItem::with_id(app, "show_window", t("show_window"), true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", t("quit"), true, None::<&str>)?;

    // Build the menu
    let menu = Menu::with_items(
        app,
        &[
            &ambient_light_item,
            &led_preview_item,
            &separator1,
            &info_item,
            &led_config_item,
            &white_balance_item,
            &led_test_item,
            &settings_item,
            &separator2,
            &auto_start_item,
            &separator3,
            &about_item,
            &show_item,
            &quit_item,
        ],
    )?;

    Ok(menu)
}

async fn handle_menu_event<R: Runtime>(app: &tauri::AppHandle<R>, event: tauri::menu::MenuEvent) {
    match event.id().as_ref() {
        "toggle_ambient_light" => {
            let state_manager = ambient_light_state::AmbientLightStateManager::global().await;
            if let Ok(new_state) = state_manager.toggle().await {
                info!("Ambient light toggled to: {}", new_state);

                // Emit event to notify frontend of state change
                let current_state = state_manager.get_state().await;
                app.emit("ambient_light_state_changed", current_state)
                    .unwrap();

                // Immediately update tray menu to reflect new state
                update_tray_menu_internal(app).await;
            }
        }
        "toggle_led_preview" => {
            let led_preview_manager = led_preview_state::LedPreviewStateManager::global().await;
            if let Ok(new_state) = led_preview_manager.toggle().await {
                info!("LED preview toggled to: {}", new_state);

                // Immediately update tray menu to reflect new state
                update_tray_menu_internal(app).await;
            }
        }
        "show_info" => {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
                let _ = window.eval("window.location.hash = '#/info'");
            }
        }
        "show_led_config" => {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
                let _ = window.emit("navigate", "/led-strips-configuration");
            }
        }
        "show_white_balance" => {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
                let _ = window.eval("window.location.hash = '#/white-balance'");
            }
        }
        "show_led_test" => {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
                let _ = window.eval("window.location.hash = '#/led-strip-test'");
            }
        }
        "show_settings" => {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
                let _ = window.eval("window.location.hash = '#/settings'");
            }
        }
        "toggle_auto_start" => {
            if let Ok(new_state) = auto_start::AutoStartManager::toggle() {
                info!("Auto start toggled to: {}", new_state);
                // Immediately update tray menu to reflect new state
                update_tray_menu_internal(app).await;
            }
        }
        "show_about" => {
            // 简单的关于对话框
            info!(
                "About: Ambient Light Control v{}",
                env!("CARGO_PKG_VERSION")
            );
            // 可以在这里添加更复杂的关于窗口逻辑
        }
        "show_window" => {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }
        "quit" => {
            app.exit(0);
        }
        _ => {}
    }
}

async fn handle_tray_event<R: Runtime>(app: &tauri::AppHandle<R>, event: TrayIconEvent) {
    if let TrayIconEvent::Click {
        button: MouseButton::Left,
        button_state: MouseButtonState::Up,
        ..
    } = event
    {
        // Left click to show/hide window
        if let Some(window) = app.get_webview_window("main") {
            if window.is_visible().unwrap_or(false) {
                let _ = window.hide();
            } else {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }
    }
}

// Helper function to extract page name from URL
fn extract_page_from_url(url: &str) -> Option<String> {
    // Handle navigation requests: ambient-light://navigate/page_name or ambient-light://navigate/page_name/display/id
    let nav_re =
        regex::Regex::new(r"ambient-light://navigate/([a-zA-Z0-9\-_]+)(?:/display/(\d+))?")
            .unwrap();
    if let Some(captures) = nav_re.captures(url) {
        let page_name = &captures[1];
        let display_id = captures.get(2).map(|m| m.as_str());

        if let Some(display_id) = display_id {
            // For display-specific pages, create a combined page identifier
            Some(format!("{page_name}-display-{display_id}"))
        } else {
            Some(page_name.to_string())
        }
    } else {
        None
    }
}

// Protocol handler for ambient-light://
fn handle_ambient_light_protocol<R: Runtime>(
    ctx: tauri::UriSchemeContext<R>,
    request: Request<Vec<u8>>,
) -> Response<Vec<u8>> {
    let url = request.uri();
    info!("Handling ambient-light protocol request: {}", url);

    // Parse the URL to extract parameters
    let url_str = url.to_string();

    // Handle navigation requests: ambient-light://navigate/page_name or ambient-light://navigate/page_name/display/id
    let nav_re =
        regex::Regex::new(r"ambient-light://navigate/([a-zA-Z0-9\-_]+)(?:/display/(\d+))?")
            .unwrap();
    if let Some(captures) = nav_re.captures(&url_str) {
        let page_name = &captures[1];
        let display_id = captures.get(2).map(|m| m.as_str());

        if let Some(display_id) = display_id {
            info!(
                "Navigation request to page: {} with display: {}",
                page_name, display_id
            );
        } else {
            info!("Navigation request to page: {}", page_name);
        }

        // Get the app handle and navigate to the requested page
        let app_handle = ctx.app_handle();
        if let Some(window) = app_handle.get_webview_window("main") {
            let route = if let Some(display_id) = display_id {
                // Handle display-specific navigation
                if page_name == "led-strips-configuration" || page_name == "led-config" {
                    format!("/led-strips-configuration/display/{display_id}")
                } else {
                    match page_name {
                        "info" => "/info".to_string(),
                        "white-balance" | "color-calibration" => "/color-calibration".to_string(),
                        "led-strip-test" | "led-test" => "/led-strip-test".to_string(),
                        "settings" => "/settings".to_string(),
                        _ => "/info".to_string(), // Default to info page
                    }
                }
            } else {
                match page_name {
                    "info" => "/info",
                    "led-strips-configuration" | "led-config" => "/led-strips-configuration",
                    "white-balance" | "color-calibration" => "/color-calibration",
                    "led-strip-test" | "led-test" => "/led-strip-test",
                    "settings" => "/settings",
                    _ => "/info", // Default to info page
                }
                .to_string()
            };

            let _ = window.show();
            let _ = window.set_focus();

            // Use the new event-driven navigation system
            let _ = window.emit("navigate", &route);
            info!("URL scheme navigation event emitted: {}", route);
        }

        let response_body = "Navigation request received";

        return Response::builder()
            .status(200)
            .body(response_body.as_bytes().to_vec())
            .unwrap();
    }

    // Handle screenshot requests: ambient-light://displays/id?width=w&height=h
    let screenshot_re =
        regex::Regex::new(r"ambient-light://displays/(\d+)\?width=(\d+)&height=(\d+)").unwrap();

    if let Some(captures) = screenshot_re.captures(&url_str) {
        let display_id: u32 = captures[1].parse().unwrap_or(0);
        let width: u32 = captures[2].parse().unwrap_or(400);
        let height: u32 = captures[3].parse().unwrap_or(300);

        // info!("Efficient screenshot request for display {}, {}x{}", display_id, width, height);

        // Optimized screenshot processing with much smaller intermediate size
        // info!("Screenshot request received: display_id={}, width={}, height={}", display_id, width, height);

        let screenshot_data = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let screenshot_manager = ScreenshotManager::global().await;
                let channels = screenshot_manager.channels.read().await;

                if let Some(rx) = channels.get(&display_id) {
                    let rx = rx.read().await;
                    let screenshot = rx.borrow().clone();
                    let bytes = screenshot.bytes.clone();

                    // Use much smaller intermediate resolution for performance
                    let intermediate_width = 800; // Much smaller than original 5120
                    let intermediate_height = 450; // Much smaller than original 2880

                    // Convert BGRA to RGBA format
                    let mut rgba_bytes = bytes.as_ref().clone();
                    for chunk in rgba_bytes.chunks_exact_mut(4) {
                        chunk.swap(0, 2); // Swap B and R channels
                    }

                    let image_result = image::RgbaImage::from_raw(
                        screenshot.width as u32,
                        screenshot.height as u32,
                        rgba_bytes,
                    );

                    if let Some(img) = image_result {
                        // Step 1: Fast downscale to intermediate size
                        let intermediate_image = image::imageops::resize(
                            &img,
                            intermediate_width,
                            intermediate_height,
                            image::imageops::FilterType::Nearest, // Fastest possible
                        );

                        // Step 2: Scale to final target size
                        let final_image =
                            if width == intermediate_width && height == intermediate_height {
                                intermediate_image
                            } else {
                                image::imageops::resize(
                                    &intermediate_image,
                                    width,
                                    height,
                                    image::imageops::FilterType::Triangle,
                                )
                            };

                        let raw_data = final_image.into_raw();
                        // info!("Efficient resize completed: {}x{}, {} bytes", width, height, raw_data.len());
                        Ok(raw_data)
                    } else {
                        error!("Failed to create image from raw bytes");
                        Err("Failed to create image from raw bytes".to_string())
                    }
                } else {
                    error!("Display {} not found", display_id);
                    Err(format!("Display {display_id} not found"))
                }
            })
        });

        match screenshot_data {
            Ok(data) => Response::builder()
                .header("Content-Type", "application/octet-stream")
                .header("Access-Control-Allow-Origin", "*")
                .header("X-Image-Width", width.to_string())
                .header("X-Image-Height", height.to_string())
                .body(data)
                .unwrap_or_else(|_| {
                    Response::builder()
                        .status(500)
                        .body("Failed to build response".as_bytes().to_vec())
                        .unwrap()
                }),
            Err(e) => {
                error!("Failed to get screenshot: {}", e);
                Response::builder()
                    .status(500)
                    .body(format!("Error: {e}").into_bytes())
                    .unwrap()
            }
        }
    } else {
        warn!("Invalid ambient-light URL format: {}", url_str);
        Response::builder()
            .status(400)
            .body("Invalid URL format".as_bytes().to_vec())
            .unwrap()
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();

    // 初始化新的稳定显示器ID系统
    log::info!("🚀 初始化稳定显示器ID系统...");
    let _config_manager_v2 = ambient_light::ConfigManagerV2::global().await;
    log::info!("✅ 稳定显示器ID系统初始化完成");

    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    let mut target_page: Option<String> = None;
    let mut display_id: Option<String> = None;
    let mut headless_mode = false;
    let mut browser_mode = false;

    // Look for --page, --display, --headless, --browser, and --test-single-display-config arguments
    let mut _test_single_display_config = false;
    for i in 0..args.len() {
        if args[i] == "--page" && i + 1 < args.len() {
            target_page = Some(args[i + 1].clone());
            info!("Command line argument detected: --page {}", args[i + 1]);
        } else if args[i] == "--display" && i + 1 < args.len() {
            display_id = Some(args[i + 1].clone());
            info!("Command line argument detected: --display {}", args[i + 1]);
        } else if args[i] == "--headless" {
            headless_mode = true;
            info!("Command line argument detected: --headless");
        } else if args[i] == "--browser" {
            browser_mode = true;
            info!("Command line argument detected: --browser");
        } else if args[i] == "--test-single-display-config" {
            _test_single_display_config = true;
            info!("Command line argument detected: --test-single-display-config");
        }
    }

    // Check environment variables
    if !headless_mode && std::env::var("AMBIENT_LIGHT_HEADLESS").is_ok() {
        headless_mode = true;
        info!("Environment variable detected: AMBIENT_LIGHT_HEADLESS");
    }

    if !browser_mode && std::env::var("AMBIENT_LIGHT_BROWSER").is_ok() {
        browser_mode = true;
        info!("Environment variable detected: AMBIENT_LIGHT_BROWSER");
    }

    // In development mode, also check environment variables for navigation
    if target_page.is_none() {
        if let Ok(env_page) = std::env::var("TAURI_DEV_PAGE") {
            target_page = Some(env_page.clone());
            info!("Environment variable detected: TAURI_DEV_PAGE={}", env_page);
        }
    }
    if display_id.is_none() {
        if let Ok(env_display) = std::env::var("TAURI_DEV_DISPLAY") {
            display_id = Some(env_display.clone());
            info!(
                "Environment variable detected: TAURI_DEV_DISPLAY={}",
                env_display
            );
        }
    }

    // If both page and display are specified, combine them
    if let (Some(page), Some(display)) = (&target_page, &display_id) {
        if page == "led-strips-configuration" || page == "led-config" {
            target_page = Some(format!("led-config-display-{display}"));
            info!(
                "Combined navigation target: {}",
                target_page.as_ref().unwrap()
            );
        }
    }

    // 启动HTTP服务器
    let http_config = http_server::ServerConfig {
        host: "127.0.0.1".to_string(),
        port: 3030,
        enable_cors: true,
        serve_static_files: false,
        static_files_path: None,
    };

    // 在后台启动HTTP服务器
    let _http_server_handle = {
        let config = http_config.clone();
        tokio::spawn(async move {
            info!("🚀 正在启动HTTP服务器...");
            match http_server::start_server(config).await {
                Ok(_) => {
                    info!("✅ HTTP服务器启动成功");
                }
                Err(e) => {
                    error!("❌ HTTP服务器启动失败: {}", e);
                    panic!("HTTP服务器启动失败: {e}");
                }
            }
        })
    };

    // Initialize display info (removed debug output)

    tokio::spawn(async move {
        info!("🖥️ Starting screenshot manager...");

        // Test display detection first
        info!("🔍 Testing display detection...");
        match DisplayInfo::all() {
            Ok(displays) => {
                info!(
                    "✅ Display detection successful: {} displays found",
                    displays.len()
                );
                for (i, display) in displays.iter().enumerate() {
                    info!(
                        "  Display {}: ID={}, Scale={}",
                        i, display.id, display.scale_factor
                    );
                }
            }
            Err(e) => {
                error!("❌ Display detection failed: {}", e);
            }
        }

        let screenshot_manager = ScreenshotManager::global().await;
        info!("📱 Screenshot manager instance obtained, calling start()...");
        match screenshot_manager.start().await {
            Ok(_) => {
                info!("✅ Screenshot manager started successfully");
            }
            Err(e) => {
                error!("❌ Failed to start screenshot manager: {}", e);
            }
        }
        info!("🏁 Screenshot manager startup task completed");
    });

    tokio::spawn(async move {
        info!("💡 Starting LED color publisher...");

        // Add a small delay to avoid initialization conflicts
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        info!("⏰ LED color publisher delay completed, proceeding...");

        let led_color_publisher = ambient_light::LedColorsPublisher::global().await;
        info!("📦 LED color publisher instance obtained");

        // Add timeout to prevent infinite blocking
        match tokio::time::timeout(
            std::time::Duration::from_secs(30),
            led_color_publisher.start(),
        )
        .await
        {
            Ok(_) => {
                info!("✅ LED color publisher started successfully");
            }
            Err(_) => {
                error!("❌ LED color publisher start() timed out after 30 seconds");
                error!("💡 This indicates a blocking issue in the start() method");
            }
        }
    });

    // WebSocket server will be started in the Tauri setup hook

    let _volume = VolumeManager::global().await;

    // 如果是无头模式，只运行后端服务，不启动GUI
    if headless_mode {
        info!("🚀 Running in headless mode - HTTP API only");
        info!("📡 HTTP API server: http://127.0.0.1:3030");
        info!("🔌 WebSocket server: ws://127.0.0.1:8765");
        info!("📖 API documentation: http://127.0.0.1:3030/swagger-ui/");
        info!("💡 Press Ctrl+C to stop the server");

        // 启动WebSocket服务器
        tokio::spawn(async move {
            if let Err(e) = start_websocket_server().await {
                error!("Failed to start WebSocket server: {}", e);
            }
        });

        // 在无头模式下保持程序运行
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    }

    // 如果是浏览器模式，启动后端服务（不启动GUI）
    if browser_mode {
        info!("🌐 Running in browser mode - Backend only");
        info!("� HTTP API server: http://127.0.0.1:3030");
        info!("🔌 WebSocket server: ws://127.0.0.1:8765");
        info!("🌐 Web interface: Start frontend dev server with 'npm run dev'");
        info!("� Then access http://localhost:1420 in your browser");
        info!("💡 Press Ctrl+C to stop the server");

        // 启动WebSocket服务器
        tokio::spawn(async move {
            if let Err(e) = start_websocket_server().await {
                error!("Failed to start WebSocket server: {}", e);
            }
        });

        // 在浏览器模式下保持程序运行
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_deep_link::init())
        // Tauri invoke handlers removed - using HTTP API only
        .register_uri_scheme_protocol("ambient-light", handle_ambient_light_protocol)
        .on_menu_event(|app, event| {
            let app_handle = app.clone();
            tauri::async_runtime::spawn(async move {
                handle_menu_event(&app_handle, event).await;
            });
        })
        .setup(move |app| {
            // Setup deep link event listener
            {
                use tauri_plugin_deep_link::DeepLinkExt;
                let app_handle = app.handle().clone();
                app.deep_link().on_open_url(move |event| {
                    let urls = event.urls();
                    info!("Deep link received: {:?}", urls);
                    for url in urls {
                        if let Some(page) = extract_page_from_url(url.as_ref()) {
                            info!("Navigating to page: {}", page);
                            let app_handle_clone = app_handle.clone();
                            tauri::async_runtime::spawn(async move {
                                // 简单的导航实现
                                info!("Deep link navigation to page: {}", page);
                                if let Some(window) = app_handle_clone.get_webview_window("main") {
                                    let route = match page.as_str() {
                                        "led-strips-configuration" => "/led-strips-configuration",
                                        "info" => "/info",
                                        "settings" => "/settings",
                                        _ => "/",
                                    };
                                    if let Err(e) =
                                        window.eval(format!("window.location.hash = '{route}'"))
                                    {
                                        error!("Failed to navigate via deep link: {}", e);
                                    }
                                } else {
                                    error!("Main window not found for deep link navigation");
                                }
                            });
                        }
                    }
                });
            }

            // Restore window state from user preferences
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                if let Some(main_window) = app_handle.get_webview_window("main") {
                    let preferences_manager = UserPreferencesManager::global().await;
                    let preferences = preferences_manager.get_preferences().await;

                    // Restore window size (using logical pixels to avoid DPI scaling issues)
                    if let Err(e) = main_window.set_size(tauri::Size::Logical(tauri::LogicalSize {
                        width: preferences.window.width,
                        height: preferences.window.height,
                    })) {
                        warn!("Failed to restore window size: {}", e);
                    }

                    // Restore window position if available (using logical pixels)
                    if let (Some(x), Some(y)) = (preferences.window.x, preferences.window.y) {
                        if let Err(e) = main_window
                            .set_position(tauri::Position::Logical(tauri::LogicalPosition { x, y }))
                        {
                            warn!("Failed to restore window position: {}", e);
                        }
                    }

                    // Restore maximized state
                    if preferences.window.maximized {
                        if let Err(e) = main_window.maximize() {
                            warn!("Failed to maximize window: {}", e);
                        }
                    }

                    info!("Window state restored from preferences");
                }
            });

            // Setup window event listeners for state persistence
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                if let Some(main_window) = app_handle.get_webview_window("main") {
                    let preferences_manager = UserPreferencesManager::global().await;

                    // Listen for window resize events
                    let preferences_manager_clone = preferences_manager;
                    let main_window_clone = main_window.clone();
                    main_window.on_window_event(move |event| {
                        let prefs_manager = preferences_manager_clone;
                        let window = main_window_clone.clone();

                        // Clone the event data to move into async task
                        match event {
                            tauri::WindowEvent::Resized(_size) => {
                                tauri::async_runtime::spawn(async move {
                                    // Get current logical size to avoid DPI scaling issues
                                    if let Ok(logical_size) = window.inner_size() {
                                        let scale_factor = window.scale_factor().unwrap_or(1.0);
                                        let logical_size =
                                            logical_size.to_logical::<f64>(scale_factor);
                                        if let Err(e) = prefs_manager
                                            .update_window_size(
                                                logical_size.width,
                                                logical_size.height,
                                            )
                                            .await
                                        {
                                            warn!("Failed to save window size: {}", e);
                                        }
                                    }
                                });
                            }
                            tauri::WindowEvent::Moved(_position) => {
                                tauri::async_runtime::spawn(async move {
                                    // Get current logical position
                                    if let Ok(position) = window.outer_position() {
                                        let scale_factor = window.scale_factor().unwrap_or(1.0);
                                        let logical_pos = position.to_logical::<f64>(scale_factor);
                                        if let Err(e) = prefs_manager
                                            .update_window_position(logical_pos.x, logical_pos.y)
                                            .await
                                        {
                                            warn!("Failed to save window position: {}", e);
                                        }
                                    }
                                });
                            }
                            _ => {}
                        }
                    });
                }
            });

            // Setup system tray
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let menu = create_tray_menu(&app_handle).await.unwrap();

                // Try to create tray icon with explicit icon path
                let tray_result = TrayIconBuilder::with_id("main")
                    .menu(&menu)
                    .icon(app_handle.default_window_icon().unwrap().clone())
                    .on_tray_icon_event(move |tray, event| {
                        let app_handle = tray.app_handle().clone();
                        tauri::async_runtime::spawn(async move {
                            handle_tray_event(&app_handle, event).await;
                        });
                    })
                    .build(&app_handle);

                match tray_result {
                    Ok(_tray) => {
                        info!("System tray created successfully");
                    }
                    Err(e) => {
                        error!("Failed to create system tray: {}", e);
                    }
                }
            });

            let app_handle = app.handle().clone();
            tokio::spawn(async move {
                // 使用新的ConfigManagerV2和适配器
                let config_manager_v2 = ambient_light::ConfigManagerV2::global().await;
                let mut config_update_receiver = config_manager_v2.subscribe_config_updates();

                // 创建适配器用于转换配置格式
                let adapter =
                    ambient_light::PublisherAdapter::new(config_manager_v2.get_display_registry());

                loop {
                    if let Err(err) = config_update_receiver.changed().await {
                        error!("config update receiver changed error: {}", err);
                        return;
                    }

                    log::info!("config changed. emit config_changed event.");

                    let v2_config = config_update_receiver.borrow().clone();

                    // 转换为v1格式以保持前端兼容性
                    match adapter.convert_v2_to_v1_config(&v2_config).await {
                        Ok(v1_config) => {
                            app_handle.emit("config_changed", v1_config).unwrap();
                        }
                        Err(e) => {
                            error!("Failed to convert v2 config to v1: {}", e);
                        }
                    }
                }
            });

            let app_handle = app.handle().clone();
            tokio::spawn(async move {
                let publisher = ambient_light::LedColorsPublisher::global().await;
                let mut publisher_update_receiver = publisher.clone_sorted_colors_receiver().await;
                loop {
                    if let Err(err) = publisher_update_receiver.changed().await {
                        error!("publisher update receiver changed error: {}", err);
                        return;
                    }

                    let publisher = publisher_update_receiver.borrow().clone();

                    app_handle
                        .emit("led_sorted_colors_changed", publisher)
                        .unwrap();
                }
            });

            let app_handle = app.handle().clone();
            tokio::spawn(async move {
                let publisher = ambient_light::LedColorsPublisher::global().await;
                let mut publisher_update_receiver = publisher.clone_colors_receiver().await;
                loop {
                    if let Err(err) = publisher_update_receiver.changed().await {
                        error!("publisher update receiver changed error: {}", err);
                        return;
                    }

                    let publisher = publisher_update_receiver.borrow().clone();

                    app_handle.emit("led_colors_changed", publisher).unwrap();
                }
            });

            let app_handle = app.handle().clone();
            tokio::spawn(async move {
                match UdpRpc::global().await {
                    Ok(udp_rpc) => {
                        let mut receiver = udp_rpc.subscribe_boards_change();
                        loop {
                            if let Err(err) = receiver.changed().await {
                                error!("boards change receiver changed error: {}", err);
                                return;
                            }

                            let boards = receiver.borrow().clone();

                            let boards = boards.into_iter().collect::<Vec<_>>();

                            app_handle.emit("boards_changed", boards).unwrap();
                        }
                    }
                    Err(err) => {
                        error!("udp rpc error: {}", err);
                    }
                }
            });

            let app_handle = app.handle().clone();
            tokio::spawn(async move {
                let display_manager = DisplayManager::global().await;
                let mut rx = display_manager.subscribe_displays_changed();

                while rx.changed().await.is_ok() {
                    let displays = rx.borrow().clone();

                    log::info!("displays changed. emit displays_changed event.");

                    app_handle.emit("displays_changed", displays).unwrap();
                }
            });

            // Screenshot manager is already started in main function

            // LED colors publisher is already started in main function

            // Start WebSocket server for screen streaming
            tokio::spawn(async move {
                if let Err(e) = start_websocket_server().await {
                    error!("Failed to start WebSocket server: {}", e);
                }
            });

            // Handle command line arguments for page navigation
            if let Some(page) = target_page {
                let app_handle = app.handle().clone();
                tokio::spawn(async move {
                    // Wait longer for the app to fully initialize and load
                    info!("Waiting for app initialization before navigation...");
                    tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;

                    info!("Starting navigation to page: {}", page);
                    // 简单的导航实现
                    if let Some(window) = app_handle.get_webview_window("main") {
                        let route = match page.as_str() {
                            "led-strips-configuration" => "/led-strips-configuration",
                            "info" => "/info",
                            "settings" => "/settings",
                            _ => "/",
                        };
                        if let Err(e) = window.eval(format!("window.location.hash = '{route}'")) {
                            error!("Failed to navigate to page '{}': {}", page, e);
                        }
                    } else {
                        error!("Main window not found for navigation to page '{}'", page);
                    }
                });
            }

            // Test mode removed - functionality moved to HTTP API

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// WebSocket server for screen streaming
async fn start_websocket_server() -> anyhow::Result<()> {
    use tokio::net::TcpListener;

    let listener = TcpListener::bind("127.0.0.1:8765").await?;
    info!("WebSocket server listening on ws://127.0.0.1:8765");

    while let Ok((stream, addr)) = listener.accept().await {
        info!("New WebSocket connection from: {}", addr);

        tokio::spawn(async move {
            info!("Starting WebSocket handler for connection from: {}", addr);
            match screen_stream::handle_websocket_connection(stream).await {
                Ok(_) => {
                    info!("WebSocket connection from {} completed successfully", addr);
                }
                Err(e) => {
                    warn!("WebSocket connection error from {}: {}", addr, e);
                }
            }
            info!("WebSocket handler task completed for: {}", addr);
        });
    }

    Ok(())
}
