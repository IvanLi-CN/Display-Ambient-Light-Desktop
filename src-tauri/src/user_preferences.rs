use dirs::config_dir;
use paris::{info, warn};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{OnceCell, RwLock};

const CONFIG_FILE_NAME: &str = "cc.ivanli.ambient_light/user_preferences.toml";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    pub window: WindowPreferences,
    pub ui: UIPreferences,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowPreferences {
    pub width: f64,
    pub height: f64,
    pub x: Option<f64>,
    pub y: Option<f64>,
    pub maximized: bool,
    pub minimized_to_tray: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIPreferences {
    pub view_scale: f64,
    pub theme: String,
    pub night_mode_theme_enabled: bool,
    pub night_mode_theme: String,
}

// DisplayPreferences removed - no implemented features

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            window: WindowPreferences::default(),
            ui: UIPreferences::default(),
        }
    }
}

impl Default for WindowPreferences {
    fn default() -> Self {
        Self {
            width: 1400.0,
            height: 1000.0,
            x: None, // Let system decide initial position
            y: None,
            maximized: false,
            minimized_to_tray: false,
        }
    }
}

impl Default for UIPreferences {
    fn default() -> Self {
        Self {
            view_scale: 0.2,
            theme: "dark".to_string(),
            night_mode_theme_enabled: false,
            night_mode_theme: "dark".to_string(),
        }
    }
}

// DisplayPreferences default implementation removed

impl UserPreferences {
    /// Get the configuration file path
    fn get_config_path() -> anyhow::Result<PathBuf> {
        let config_dir =
            config_dir().ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?;
        Ok(config_dir.join(CONFIG_FILE_NAME))
    }

    /// Read configuration from file
    pub async fn read_config() -> anyhow::Result<Self> {
        let config_path = Self::get_config_path()?;

        if !config_path.exists() {
            info!("User preferences config file not found, using defaults");
            return Ok(Self::default());
        }

        let content = fs::read_to_string(&config_path)?;
        let config: Self = toml::from_str(&content)?;
        Ok(config)
    }

    /// Write configuration to file
    pub async fn write_config(&self) -> anyhow::Result<()> {
        let config_path = Self::get_config_path()?;

        // Create parent directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(self)?;
        fs::write(&config_path, content)?;
        Ok(())
    }
}

pub struct UserPreferencesManager {
    preferences: Arc<RwLock<UserPreferences>>,
}

impl UserPreferencesManager {
    pub async fn global() -> &'static Self {
        static USER_PREFERENCES_MANAGER: OnceCell<UserPreferencesManager> = OnceCell::const_new();

        USER_PREFERENCES_MANAGER
            .get_or_init(|| async {
                let preferences = match UserPreferences::read_config().await {
                    Ok(prefs) => prefs,
                    Err(e) => {
                        warn!(
                            "Failed to read user preferences config: {}, using defaults",
                            e
                        );
                        UserPreferences::default()
                    }
                };

                Self {
                    preferences: Arc::new(RwLock::new(preferences)),
                }
            })
            .await
    }

    /// Get current user preferences
    pub async fn get_preferences(&self) -> UserPreferences {
        self.preferences.read().await.clone()
    }

    /// Update user preferences
    pub async fn update_preferences(&self, preferences: UserPreferences) -> anyhow::Result<()> {
        // Write to file first
        preferences.write_config().await?;

        // Update in-memory state
        let mut current_prefs = self.preferences.write().await;
        *current_prefs = preferences;

        Ok(())
    }

    /// Update window preferences
    pub async fn update_window_preferences(
        &self,
        window_prefs: WindowPreferences,
    ) -> anyhow::Result<()> {
        let mut preferences = self.get_preferences().await;
        preferences.window = window_prefs;
        self.update_preferences(preferences).await
    }

    /// Update UI preferences
    pub async fn update_ui_preferences(&self, ui_prefs: UIPreferences) -> anyhow::Result<()> {
        let mut preferences = self.get_preferences().await;
        preferences.ui = ui_prefs;
        self.update_preferences(preferences).await
    }

    /// Update specific window property
    pub async fn update_window_size(&self, width: f64, height: f64) -> anyhow::Result<()> {
        let mut preferences = self.get_preferences().await;
        preferences.window.width = width;
        preferences.window.height = height;
        self.update_preferences(preferences).await
    }

    /// Update window position
    pub async fn update_window_position(&self, x: f64, y: f64) -> anyhow::Result<()> {
        let mut preferences = self.get_preferences().await;
        preferences.window.x = Some(x);
        preferences.window.y = Some(y);
        self.update_preferences(preferences).await
    }

    /// Update window maximized state
    pub async fn update_window_maximized(&self, maximized: bool) -> anyhow::Result<()> {
        let mut preferences = self.get_preferences().await;
        preferences.window.maximized = maximized;
        self.update_preferences(preferences).await
    }

    /// Update view scale
    pub async fn update_view_scale(&self, scale: f64) -> anyhow::Result<()> {
        let mut preferences = self.get_preferences().await;
        preferences.ui.view_scale = scale;
        self.update_preferences(preferences).await
    }

    /// Update theme
    pub async fn update_theme(&self, theme: String) -> anyhow::Result<()> {
        let mut preferences = self.get_preferences().await;
        preferences.ui.theme = theme;
        self.update_preferences(preferences).await
    }

    /// Update night mode theme enabled status
    pub async fn update_night_mode_theme_enabled(&self, enabled: bool) -> anyhow::Result<()> {
        let mut preferences = self.get_preferences().await;
        preferences.ui.night_mode_theme_enabled = enabled;
        self.update_preferences(preferences).await
    }

    /// Update night mode theme
    pub async fn update_night_mode_theme(&self, theme: String) -> anyhow::Result<()> {
        let mut preferences = self.get_preferences().await;
        preferences.ui.night_mode_theme = theme;
        self.update_preferences(preferences).await
    }

    /// Get night mode theme enabled status
    pub async fn get_night_mode_theme_enabled(&self) -> bool {
        let preferences = self.get_preferences().await;
        preferences.ui.night_mode_theme_enabled
    }

    /// Get night mode theme
    pub async fn get_night_mode_theme(&self) -> String {
        let preferences = self.get_preferences().await;
        preferences.ui.night_mode_theme.clone()
    }

    // Removed update_last_visited_page - feature not implemented
}
