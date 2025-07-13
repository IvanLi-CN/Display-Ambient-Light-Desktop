// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod ambient_light;
mod ambient_light_state;
mod auto_start;
mod display;
mod language_manager;
mod led_color;
mod led_test_effects;
mod rpc;
mod screen_stream;
mod screenshot;
mod screenshot_manager;
mod user_preferences;
mod volume;

use ambient_light::{Border, ColorCalibration, LedStripConfig, LedStripConfigGroup, LedType};
use display::{DisplayManager, DisplayState};
use display_info::DisplayInfo;
use led_test_effects::{LedTestEffects, TestEffectConfig};
use paris::{error, info, warn};
use rpc::{BoardInfo, UdpRpc};
use screenshot::Screenshot;
use screenshot_manager::ScreenshotManager;
use tauri::{
    http::{Request, Response},
    menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager, Runtime,
};
use user_preferences::{UIPreferences, UserPreferences, UserPreferencesManager, WindowPreferences};

use serde::{Deserialize, Serialize};
use serde_json::to_string;
use std::sync::Arc;
use tokio::sync::RwLock;
use volume::VolumeManager;

// Global static variables for LED test effect management
static EFFECT_HANDLE: tokio::sync::OnceCell<Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>> =
    tokio::sync::OnceCell::const_new();
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

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {name}! You've been greeted from Rust!")
}

#[derive(Serialize)]
struct AppVersion {
    version: String,
    is_dev: bool,
}

#[tauri::command]
fn get_app_version() -> AppVersion {
    let version = env!("CARGO_PKG_VERSION").to_string();
    let is_dev = cfg!(debug_assertions) && std::env::var("HIDE_DEV_MARKER").is_err();

    AppVersion { version, is_dev }
}

#[tauri::command]
fn list_display_info() -> Result<String, String> {
    let displays = display_info::DisplayInfo::all().map_err(|e| {
        error!("can not list display info: {}", e);
        e.to_string()
    })?;
    let displays: Vec<DisplayInfoWrapper> = displays.iter().map(DisplayInfoWrapper).collect();
    let json_str = to_string(&displays).map_err(|e| {
        error!("can not list display info: {}", e);
        e.to_string()
    })?;
    Ok(json_str)
}

#[tauri::command]
async fn read_led_strip_configs() -> Result<LedStripConfigGroup, String> {
    let config = ambient_light::LedStripConfigGroup::read_config()
        .await
        .map_err(|e| {
            error!("can not read led strip configs: {}", e);
            e.to_string()
        })?;
    Ok(config)
}

#[tauri::command]
async fn write_led_strip_configs(
    configs: Vec<ambient_light::LedStripConfig>,
) -> Result<(), String> {
    let config_manager = ambient_light::ConfigManager::global().await;

    config_manager.set_items(configs).await.map_err(|e| {
        error!("can not write led strip configs: {}", e);
        e.to_string()
    })
}

#[tauri::command]
async fn get_led_strips_sample_points(
    config: LedStripConfig,
) -> Result<Vec<screenshot::LedSamplePoints>, String> {
    let screenshot_manager = ScreenshotManager::global().await;
    let channels = screenshot_manager.channels.read().await;
    if let Some(rx) = channels.get(&config.display_id) {
        let rx = rx.read().await;
        let screenshot = rx.borrow().clone();
        let sample_points = screenshot.get_sample_points(&config);
        Ok(sample_points)
    } else {
        Err(format!("display not found: {}", config.display_id))
    }
}

#[tauri::command]
async fn get_one_edge_colors(
    display_id: u32,
    sample_points: Vec<screenshot::LedSamplePoints>,
) -> Result<Vec<led_color::LedColor>, String> {
    let screenshot_manager = ScreenshotManager::global().await;
    let channels = screenshot_manager.channels.read().await;
    if let Some(rx) = channels.get(&display_id) {
        let rx = rx.read().await;
        let screenshot = rx.borrow().clone();
        let bytes = screenshot.bytes.read().await.to_owned();
        let colors =
            Screenshot::get_one_edge_colors(&sample_points, &bytes, screenshot.bytes_per_row);
        Ok(colors)
    } else {
        Err(format!("display not found: {display_id}"))
    }
}

#[tauri::command]
async fn patch_led_strip_len(display_id: u32, border: Border, delta_len: i8) -> Result<(), String> {
    info!(
        "patch_led_strip_len: {} {:?} {}",
        display_id, border, delta_len
    );
    let config_manager = ambient_light::ConfigManager::global().await;
    config_manager
        .patch_led_strip_len(display_id, border, delta_len)
        .await
        .map_err(|e| {
            error!("can not patch led strip len: {}", e);
            e.to_string()
        })?;

    info!("patch_led_strip_len: ok");
    Ok(())
}

#[tauri::command]
async fn patch_led_strip_type(
    display_id: u32,
    border: Border,
    led_type: LedType,
) -> Result<(), String> {
    let config_manager = ambient_light::ConfigManager::global().await;
    config_manager
        .patch_led_strip_type(display_id, border, led_type)
        .await
        .map_err(|e| {
            error!("can not patch led strip type: {}", e);
            e.to_string()
        })?;

    Ok(())
}

#[tauri::command]
async fn send_colors(offset: u16, buffer: Vec<u8>) -> Result<(), String> {
    ambient_light::LedColorsPublisher::send_colors(offset, buffer)
        .await
        .map_err(|e| {
            error!("can not send colors: {}", e);
            e.to_string()
        })
}

#[tauri::command]
async fn send_test_colors_to_board(
    board_address: String,
    offset: u16,
    buffer: Vec<u8>,
) -> Result<(), String> {
    use tokio::net::UdpSocket;

    let socket = UdpSocket::bind("0.0.0.0:0").await.map_err(|e| {
        error!("Failed to bind UDP socket: {}", e);
        e.to_string()
    })?;

    let mut packet = vec![0x02]; // Header
    packet.push((offset >> 8) as u8); // Byte offset high
    packet.push((offset & 0xff) as u8); // Byte offset low
    packet.extend_from_slice(&buffer); // Color data

    socket.send_to(&packet, &board_address).await.map_err(|e| {
        error!(
            "Failed to send test colors to board {}: {}",
            board_address, e
        );
        e.to_string()
    })?;

    info!(
        "Sent test colors to board {} with offset {} and {} bytes",
        board_address,
        offset,
        buffer.len()
    );
    Ok(())
}

#[tauri::command]
async fn enable_test_mode() -> Result<(), String> {
    let publisher = ambient_light::LedColorsPublisher::global().await;
    publisher.enable_test_mode().await;
    Ok(())
}

#[tauri::command]
async fn disable_test_mode() -> Result<(), String> {
    info!("🔄 disable_test_mode command called from frontend");
    let publisher = ambient_light::LedColorsPublisher::global().await;
    publisher.disable_test_mode().await;
    info!("✅ disable_test_mode command completed");
    Ok(())
}

#[tauri::command]
async fn is_test_mode_active() -> Result<bool, String> {
    let publisher = ambient_light::LedColorsPublisher::global().await;
    Ok(publisher.is_test_mode_active().await)
}

#[tauri::command]
async fn start_led_test_effect(
    board_address: String,
    effect_config: TestEffectConfig,
    update_interval_ms: u64,
) -> Result<(), String> {
    use tokio::time::{interval, Duration};

    // Enable test mode first
    let publisher = ambient_light::LedColorsPublisher::global().await;
    publisher.enable_test_mode().await;

    let handle_storage = EFFECT_HANDLE
        .get_or_init(|| async { Arc::new(RwLock::new(None)) })
        .await;

    let cancel_storage = CANCEL_TOKEN
        .get_or_init(|| async { Arc::new(RwLock::new(None)) })
        .await;

    // Stop any existing effect
    {
        let mut cancel_guard = cancel_storage.write().await;
        if let Some(token) = cancel_guard.take() {
            token.cancel();
        }

        let mut handle_guard = handle_storage.write().await;
        if let Some(handle) = handle_guard.take() {
            let _ = handle.await; // Wait for graceful shutdown
        }
    }

    // Start new effect
    let effect_config = Arc::new(effect_config);
    let board_address = Arc::new(board_address);
    let start_time = std::time::Instant::now();

    // Create new cancellation token
    let cancel_token = tokio_util::sync::CancellationToken::new();
    let cancel_token_clone = cancel_token.clone();

    let handle = tokio::spawn(async move {
        let mut interval = interval(Duration::from_millis(update_interval_ms));

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    let elapsed_ms = start_time.elapsed().as_millis() as u64;
                    let colors = LedTestEffects::generate_colors(&effect_config, elapsed_ms);

                    // Calculate byte offset for 0x02 packet
                    let byte_offset = LedTestEffects::calculate_byte_offset(&effect_config);

                    // Send to board with calculated offset
                    if let Err(e) = send_test_colors_to_board_internal(&board_address, byte_offset, colors).await {
                        error!("Failed to send test effect colors: {}", e);
                        break;
                    }
                }
                _ = cancel_token_clone.cancelled() => {
                    info!("LED test effect cancelled gracefully");
                    break;
                }
            }
        }
        info!("LED test effect task ended");
    });

    // Store the handle and cancel token
    {
        let mut handle_guard = handle_storage.write().await;
        *handle_guard = Some(handle);

        let mut cancel_guard = cancel_storage.write().await;
        *cancel_guard = Some(cancel_token);
    }

    Ok(())
}

#[tauri::command]
async fn stop_led_test_effect(
    board_address: String,
    led_count: u32,
    led_type: led_test_effects::LedType,
) -> Result<(), String> {
    // Stop the effect task first

    info!("🛑 Stopping LED test effect - board: {}", board_address);

    // Cancel the task gracefully first
    if let Some(cancel_storage) = CANCEL_TOKEN.get() {
        let mut cancel_guard = cancel_storage.write().await;
        if let Some(token) = cancel_guard.take() {
            info!("🔄 Cancelling test effect task gracefully");
            token.cancel();
        }
    }

    // Wait for the task to finish
    if let Some(handle_storage) = EFFECT_HANDLE.get() {
        let mut handle_guard = handle_storage.write().await;
        if let Some(handle) = handle_guard.take() {
            info!("⏳ Waiting for test effect task to finish");
            match handle.await {
                Ok(_) => info!("✅ Test effect task finished successfully"),
                Err(e) => warn!("⚠️ Test effect task finished with error: {}", e),
            }
        }
    }

    // Turn off all LEDs
    let bytes_per_led = match led_type {
        led_test_effects::LedType::WS2812B => 3,
        led_test_effects::LedType::SK6812 => 4,
    };
    let buffer = vec![0u8; (led_count * bytes_per_led) as usize];

    send_test_colors_to_board_internal(&board_address, 0, buffer)
        .await
        .map_err(|e| e.to_string())?;

    info!("💡 Sent LED off command");

    // Disable test mode to resume normal publishing
    let publisher = ambient_light::LedColorsPublisher::global().await;
    publisher.disable_test_mode().await;

    info!("🔄 Test mode disabled, normal publishing resumed");
    info!("✅ LED test effect stopped completely");

    Ok(())
}

// Internal helper function
async fn send_test_colors_to_board_internal(
    board_address: &str,
    offset: u16,
    buffer: Vec<u8>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use tokio::net::UdpSocket;

    let socket = UdpSocket::bind("0.0.0.0:0").await?;

    let mut packet = vec![0x02]; // Header
    packet.push((offset >> 8) as u8); // Byte offset high
    packet.push((offset & 0xff) as u8); // Byte offset low
    packet.extend_from_slice(&buffer); // Color data

    socket.send_to(&packet, board_address).await?;
    Ok(())
}

#[tauri::command]
async fn move_strip_part(
    display_id: u32,
    border: Border,
    target_start: usize,
) -> Result<(), String> {
    let config_manager = ambient_light::ConfigManager::global().await;
    config_manager
        .move_strip_part(display_id, border, target_start)
        .await
        .map_err(|e| {
            error!("can not move strip part: {}", e);
            e.to_string()
        })
}

#[tauri::command]
async fn reverse_led_strip_part(display_id: u32, border: Border) -> Result<(), String> {
    let config_manager = ambient_light::ConfigManager::global().await;
    config_manager
        .reverse_led_strip_part(display_id, border)
        .await
        .map_err(|e| {
            error!("can not reverse led strip part: {}", e);
            e.to_string()
        })
}

#[tauri::command]
async fn set_color_calibration(calibration: ColorCalibration) -> Result<(), String> {
    let config_manager = ambient_light::ConfigManager::global().await;
    config_manager
        .set_color_calibration(calibration)
        .await
        .map_err(|e| {
            error!("can not set color calibration: {}", e);
            e.to_string()
        })
}

#[tauri::command]
async fn read_config() -> ambient_light::LedStripConfigGroup {
    let config_manager = ambient_light::ConfigManager::global().await;
    config_manager.configs().await
}

#[tauri::command]
async fn get_boards() -> Result<Vec<BoardInfo>, String> {
    let udp_rpc = UdpRpc::global().await;

    if let Err(e) = udp_rpc {
        return Err(format!("can not ping: {e}"));
    }

    let udp_rpc = udp_rpc.as_ref().unwrap();

    let boards = udp_rpc.get_boards().await;
    let boards = boards.into_iter().collect::<Vec<_>>();
    Ok(boards)
}

#[tauri::command]
async fn get_displays() -> Vec<DisplayState> {
    let display_manager = DisplayManager::global().await;

    display_manager.get_displays().await
}

#[tauri::command]
fn is_auto_start_enabled() -> Result<bool, String> {
    auto_start::AutoStartManager::is_enabled().map_err(|e| {
        error!("Failed to check auto start status: {}", e);
        e.to_string()
    })
}

#[tauri::command]
fn set_auto_start_enabled(enabled: bool) -> Result<(), String> {
    auto_start::AutoStartManager::set_enabled(enabled).map_err(|e| {
        error!("Failed to set auto start: {}", e);
        e.to_string()
    })
}

#[tauri::command]
fn get_auto_start_config() -> Result<auto_start::AutoStartConfig, String> {
    auto_start::AutoStartManager::get_config().map_err(|e| {
        error!("Failed to get auto start config: {}", e);
        e.to_string()
    })
}

#[tauri::command]
async fn is_ambient_light_enabled() -> Result<bool, String> {
    let state_manager = ambient_light_state::AmbientLightStateManager::global().await;
    Ok(state_manager.is_enabled().await)
}

#[tauri::command]
async fn set_ambient_light_enabled(enabled: bool) -> Result<(), String> {
    let state_manager = ambient_light_state::AmbientLightStateManager::global().await;
    state_manager.set_enabled(enabled).await.map_err(|e| {
        error!("Failed to set ambient light state: {}", e);
        e.to_string()
    })
}

#[tauri::command]
async fn toggle_ambient_light() -> Result<bool, String> {
    let state_manager = ambient_light_state::AmbientLightStateManager::global().await;
    state_manager.toggle().await.map_err(|e| {
        error!("Failed to toggle ambient light state: {}", e);
        e.to_string()
    })
}

#[tauri::command]
async fn get_ambient_light_state() -> Result<ambient_light_state::AmbientLightState, String> {
    let state_manager = ambient_light_state::AmbientLightStateManager::global().await;
    Ok(state_manager.get_state().await)
}

#[tauri::command]
async fn get_current_language() -> Result<String, String> {
    let language_manager = language_manager::LanguageManager::global().await;
    Ok(language_manager.get_language().await)
}

#[tauri::command]
async fn set_current_language(language: String) -> Result<(), String> {
    let language_manager = language_manager::LanguageManager::global().await;
    language_manager.set_language(language).await.map_err(|e| {
        error!("Failed to set language: {}", e);
        e.to_string()
    })
}

#[tauri::command]
async fn get_user_preferences() -> Result<UserPreferences, String> {
    let preferences_manager = UserPreferencesManager::global().await;
    Ok(preferences_manager.get_preferences().await)
}

#[tauri::command]
async fn update_user_preferences(preferences: UserPreferences) -> Result<(), String> {
    let preferences_manager = UserPreferencesManager::global().await;
    preferences_manager
        .update_preferences(preferences)
        .await
        .map_err(|e| {
            error!("Failed to update user preferences: {}", e);
            e.to_string()
        })
}

#[tauri::command]
async fn update_window_preferences(window_prefs: WindowPreferences) -> Result<(), String> {
    let preferences_manager = UserPreferencesManager::global().await;
    preferences_manager
        .update_window_preferences(window_prefs)
        .await
        .map_err(|e| {
            error!("Failed to update window preferences: {}", e);
            e.to_string()
        })
}

#[tauri::command]
async fn update_ui_preferences(ui_prefs: UIPreferences) -> Result<(), String> {
    let preferences_manager = UserPreferencesManager::global().await;
    preferences_manager
        .update_ui_preferences(ui_prefs)
        .await
        .map_err(|e| {
            error!("Failed to update UI preferences: {}", e);
            e.to_string()
        })
}

// Removed update_display_preferences - feature not implemented

#[tauri::command]
async fn update_view_scale(scale: f64) -> Result<(), String> {
    let preferences_manager = UserPreferencesManager::global().await;
    preferences_manager
        .update_view_scale(scale)
        .await
        .map_err(|e| {
            error!("Failed to update view scale: {}", e);
            e.to_string()
        })
}

#[tauri::command]
async fn update_theme(theme: String) -> Result<(), String> {
    let preferences_manager = UserPreferencesManager::global().await;
    preferences_manager.update_theme(theme).await.map_err(|e| {
        error!("Failed to update theme: {}", e);
        e.to_string()
    })
}

#[tauri::command]
async fn get_theme() -> Result<String, String> {
    let preferences_manager = UserPreferencesManager::global().await;
    let preferences = preferences_manager.get_preferences().await;
    Ok(preferences.ui.theme)
}

#[tauri::command]
async fn update_night_mode_theme_enabled(enabled: bool) -> Result<(), String> {
    let preferences_manager = UserPreferencesManager::global().await;
    preferences_manager
        .update_night_mode_theme_enabled(enabled)
        .await
        .map_err(|e| {
            error!("Failed to update night mode theme enabled: {}", e);
            e.to_string()
        })
}

#[tauri::command]
async fn update_night_mode_theme(theme: String) -> Result<(), String> {
    let preferences_manager = UserPreferencesManager::global().await;
    preferences_manager
        .update_night_mode_theme(theme)
        .await
        .map_err(|e| {
            error!("Failed to update night mode theme: {}", e);
            e.to_string()
        })
}

#[tauri::command]
async fn get_night_mode_theme_enabled() -> Result<bool, String> {
    let preferences_manager = UserPreferencesManager::global().await;
    Ok(preferences_manager.get_night_mode_theme_enabled().await)
}

#[tauri::command]
async fn get_night_mode_theme() -> Result<String, String> {
    let preferences_manager = UserPreferencesManager::global().await;
    Ok(preferences_manager.get_night_mode_theme().await)
}

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

#[tauri::command]
async fn update_tray_menu(app_handle: tauri::AppHandle) -> Result<(), String> {
    update_tray_menu_internal(&app_handle).await;
    Ok(())
}

#[tauri::command]
async fn test_tray_visibility(app_handle: tauri::AppHandle) -> Result<String, String> {
    if let Some(_tray) = app_handle.tray_by_id("main") {
        Ok("Tray icon exists and is accessible".to_string())
    } else {
        Err("Tray icon not found".to_string())
    }
}

#[tauri::command]
async fn get_app_version_string() -> Result<String, String> {
    Ok(env!("CARGO_PKG_VERSION").to_string())
}

#[tauri::command]
async fn open_external_url(url: String, app_handle: tauri::AppHandle) -> Result<(), String> {
    use tauri_plugin_opener::OpenerExt;

    // Validate URL to prevent security issues
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err("Invalid URL scheme".to_string());
    }

    // Use opener plugin to open URL
    app_handle
        .opener()
        .open_url(url, None::<String>)
        .map_err(|e| format!("Failed to open URL: {}", e))
}

#[tauri::command]
async fn show_about_window<R: tauri::Runtime>(
    app_handle: tauri::AppHandle<R>,
) -> Result<(), String> {
    use tauri::WebviewWindowBuilder;

    // Check if about window already exists
    if let Some(window) = app_handle.get_webview_window("about") {
        let _ = window.show();
        let _ = window.set_focus();
        return Ok(());
    }

    // Create new about window
    let about_window = WebviewWindowBuilder::new(
        &app_handle,
        "about",
        tauri::WebviewUrl::App("about.html".into()),
    )
    .title("关于环境光控制")
    .inner_size(420.0, 450.0)
    .min_inner_size(400.0, 430.0)
    .max_inner_size(450.0, 480.0)
    .resizable(false)
    .center()
    .build()
    .map_err(|e| format!("Failed to create about window: {e}"))?;

    let _ = about_window.show();
    let _ = about_window.set_focus();

    Ok(())
}

async fn create_tray_menu<R: Runtime>(app: &tauri::AppHandle<R>) -> tauri::Result<Menu<R>> {
    let state_manager = ambient_light_state::AmbientLightStateManager::global().await;
    let ambient_light_enabled = state_manager.is_enabled().await;
    let auto_start_enabled = auto_start::AutoStartManager::is_enabled().unwrap_or(false);

    info!(
        "Creating tray menu - Ambient light: {}, Auto start: {}",
        ambient_light_enabled, auto_start_enabled
    );

    // Get current language
    let language_manager = language_manager::LanguageManager::global().await;
    let current_language = language_manager.get_language().await;
    let t = |key: &str| language_manager::TrayTranslations::get_text(&current_language, key);

    // Create menu items
    let ambient_light_item = CheckMenuItem::with_id(
        app,
        "toggle_ambient_light",
        t("ambient_light").to_string(),
        true,
        ambient_light_enabled,
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
        t("auto_start").to_string(),
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
            if let Ok(new_state) = toggle_ambient_light().await {
                info!("Ambient light toggled to: {}", new_state);

                // Emit event to notify frontend of state change
                let state_manager = ambient_light_state::AmbientLightStateManager::global().await;
                let current_state = state_manager.get_state().await;
                app.emit("ambient_light_state_changed", current_state)
                    .unwrap();

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
                let _ = window.eval("window.location.hash = '#/led-strips-configuration'");
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
            if let Err(e) = show_about_window(app.clone()).await {
                error!("Failed to show about window: {}", e);
            }
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
    match event {
        TrayIconEvent::Click {
            button: MouseButton::Left,
            button_state: MouseButtonState::Up,
            ..
        } => {
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
        _ => {}
    }
}

// Protocol handler for ambient-light://
fn handle_ambient_light_protocol<R: Runtime>(
    _ctx: tauri::UriSchemeContext<R>,
    request: Request<Vec<u8>>,
) -> Response<Vec<u8>> {
    let url = request.uri();
    // info!("Handling ambient-light protocol request: {}", url);

    // Parse the URL to extract parameters
    let url_str = url.to_string();
    let re =
        regex::Regex::new(r"ambient-light://displays/(\d+)\?width=(\d+)&height=(\d+)").unwrap();

    if let Some(captures) = re.captures(&url_str) {
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
                    let bytes = screenshot.bytes.read().await.to_owned();

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

    // Initialize display info (removed debug output)

    tokio::spawn(async move {
        let screenshot_manager = ScreenshotManager::global().await;
        screenshot_manager.start().await.unwrap_or_else(|e| {
            error!("can not start screenshot manager: {}", e);
        })
    });

    tokio::spawn(async move {
        let led_color_publisher = ambient_light::LedColorsPublisher::global().await;
        led_color_publisher.start().await;
    });

    // Start WebSocket server for screen streaming
    tokio::spawn(async move {
        if let Err(e) = start_websocket_server().await {
            error!("Failed to start WebSocket server: {}", e);
        }
    });

    let _volume = VolumeManager::global().await;

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            get_app_version,
            list_display_info,
            read_led_strip_configs,
            write_led_strip_configs,
            get_led_strips_sample_points,
            get_one_edge_colors,
            patch_led_strip_len,
            patch_led_strip_type,
            send_colors,
            send_test_colors_to_board,
            enable_test_mode,
            disable_test_mode,
            is_test_mode_active,
            start_led_test_effect,
            stop_led_test_effect,
            move_strip_part,
            reverse_led_strip_part,
            set_color_calibration,
            read_config,
            get_boards,
            get_displays,
            is_auto_start_enabled,
            set_auto_start_enabled,
            get_auto_start_config,
            is_ambient_light_enabled,
            set_ambient_light_enabled,
            toggle_ambient_light,
            get_ambient_light_state,
            get_current_language,
            set_current_language,
            get_user_preferences,
            update_user_preferences,
            update_window_preferences,
            update_ui_preferences,
            update_view_scale,
            update_theme,
            get_theme,
            update_night_mode_theme_enabled,
            update_night_mode_theme,
            get_night_mode_theme_enabled,
            get_night_mode_theme,
            update_tray_menu,
            test_tray_visibility,
            get_app_version_string,
            open_external_url,
            show_about_window
        ])
        .register_uri_scheme_protocol("ambient-light", handle_ambient_light_protocol)
        .on_menu_event(|app, event| {
            let app_handle = app.clone();
            tauri::async_runtime::spawn(async move {
                handle_menu_event(&app_handle, event).await;
            });
        })
        .setup(move |app| {
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
                let config_manager = ambient_light::ConfigManager::global().await;
                let mut config_update_receiver = config_manager.clone_config_update_receiver();
                loop {
                    if let Err(err) = config_update_receiver.changed().await {
                        error!("config update receiver changed error: {}", err);
                        return;
                    }

                    log::info!("config changed. emit config_changed event.");

                    let config = config_update_receiver.borrow().clone();

                    app_handle.emit("config_changed", config).unwrap();
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

            // Start screenshot manager
            tokio::spawn(async move {
                let screenshot_manager = ScreenshotManager::global().await;
                screenshot_manager.start().await.unwrap();
            });

            // Start LED colors publisher
            tokio::spawn(async move {
                let publisher = ambient_light::LedColorsPublisher::global().await;
                publisher.start().await;
            });

            // Start WebSocket server for screen streaming
            tokio::spawn(async move {
                if let Err(e) = start_websocket_server().await {
                    error!("Failed to start WebSocket server: {}", e);
                }
            });

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
