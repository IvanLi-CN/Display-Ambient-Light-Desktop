use dirs::config_dir;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{OnceCell, RwLock};

const CONFIG_FILE_NAME: &str = "cc.ivanli.ambient_light/language.toml";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageConfig {
    pub language: String,
}

impl Default for LanguageConfig {
    fn default() -> Self {
        Self {
            language: "zh-CN".to_string(), // Default to Chinese
        }
    }
}

impl LanguageConfig {
    /// Get the config file path
    fn get_config_path() -> anyhow::Result<PathBuf> {
        let config_dir =
            config_dir().ok_or_else(|| anyhow::anyhow!("Failed to get config directory"))?;
        Ok(config_dir.join(CONFIG_FILE_NAME))
    }

    /// Read configuration from file
    pub async fn read_config() -> anyhow::Result<Self> {
        let config_path = Self::get_config_path()?;

        if !config_path.exists() {
            // If config file doesn't exist, create default config
            let default_config = Self::default();
            default_config.write_config().await?;
            return Ok(default_config);
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

pub struct LanguageManager {
    config: Arc<RwLock<LanguageConfig>>,
}

impl LanguageManager {
    pub async fn global() -> &'static Self {
        static LANGUAGE_MANAGER: OnceCell<LanguageManager> = OnceCell::const_new();

        LANGUAGE_MANAGER
            .get_or_init(|| async {
                let config: LanguageConfig =
                    (LanguageConfig::read_config().await).unwrap_or_default();

                log::info!(
                    "🌐 LanguageManager initialized with language: {}",
                    config.language
                );

                Self {
                    config: Arc::new(RwLock::new(config)),
                }
            })
            .await
    }

    /// Get current language
    pub async fn get_language(&self) -> String {
        let language = self.config.read().await.language.clone();
        log::info!("🌐 LanguageManager::get_language() returning: {}", language);
        language
    }

    /// Set language
    pub async fn set_language(&self, language: String) -> anyhow::Result<()> {
        {
            let mut config = self.config.write().await;
            config.language = language;
        }

        // Save to file
        let current_config = self.config.read().await.clone();
        current_config.write_config().await?;
        Ok(())
    }
}

// Translation helper
pub struct TrayTranslations;

impl TrayTranslations {
    pub fn get_text(language: &str, key: &str) -> &'static str {
        match (language, key) {
            // Chinese translations
            ("zh-CN", "ambient_light") => "氛围灯",
            ("zh-CN", "info") => "系统信息",
            ("zh-CN", "led_configuration") => "灯条配置",
            ("zh-CN", "white_balance") => "颜色校准",
            ("zh-CN", "led_test") => "灯带测试",
            ("zh-CN", "settings") => "设置",
            ("zh-CN", "auto_start") => "开机自启",
            ("zh-CN", "about") => "关于",
            ("zh-CN", "show_window") => "显示窗口",
            ("zh-CN", "quit") => "退出",

            // English translations
            ("en-US", "ambient_light") => "Ambient Light",
            ("en-US", "info") => "System Info",
            ("en-US", "led_configuration") => "LED Configuration",
            ("en-US", "white_balance") => "White Balance",
            ("en-US", "led_test") => "LED Test",
            ("en-US", "settings") => "Settings",
            ("en-US", "auto_start") => "Auto Start",
            ("en-US", "about") => "About",
            ("en-US", "show_window") => "Show Window",
            ("en-US", "quit") => "Quit",

            // Default to English
            _ => match key {
                "ambient_light" => "Ambient Light",
                "info" => "System Info",
                "led_configuration" => "LED Configuration",
                "white_balance" => "Color Calibration",
                "led_test" => "LED Test",
                "settings" => "Settings",
                "auto_start" => "Auto Start",
                "about" => "About",
                "show_window" => "Show Window",
                "quit" => "Quit",
                _ => "Unknown",
            },
        }
    }
}
