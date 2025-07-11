use dirs::config_dir;
use paris::{info, warn};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{OnceCell, RwLock};

const CONFIG_FILE_NAME: &str = "cc.ivanli.ambient_light/ambient_light_state.toml";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmbientLightState {
    pub enabled: bool,
}

impl Default for AmbientLightState {
    fn default() -> Self {
        Self {
            enabled: true, // Default to enabled
        }
    }
}

impl AmbientLightState {
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

pub struct AmbientLightStateManager {
    state: Arc<RwLock<AmbientLightState>>,
}

impl AmbientLightStateManager {
    pub async fn global() -> &'static Self {
        static AMBIENT_LIGHT_STATE_MANAGER: OnceCell<AmbientLightStateManager> =
            OnceCell::const_new();

        AMBIENT_LIGHT_STATE_MANAGER
            .get_or_init(|| async {
                let state = match AmbientLightState::read_config().await {
                    Ok(state) => state,
                    Err(e) => {
                        warn!(
                            "Failed to read ambient light state config: {}, using default",
                            e
                        );
                        AmbientLightState::default()
                    }
                };

                Self {
                    state: Arc::new(RwLock::new(state)),
                }
            })
            .await
    }

    /// Get current ambient light state
    pub async fn get_state(&self) -> AmbientLightState {
        self.state.read().await.clone()
    }

    /// Check if ambient light is enabled
    pub async fn is_enabled(&self) -> bool {
        self.state.read().await.enabled
    }

    /// Set ambient light enabled state
    pub async fn set_enabled(&self, enabled: bool) -> anyhow::Result<()> {
        {
            let mut state = self.state.write().await;
            state.enabled = enabled;
        }

        // Save to file
        let current_state = self.get_state().await;
        current_state.write_config().await?;

        info!(
            "Ambient light state changed to: {}",
            if enabled { "enabled" } else { "disabled" }
        );
        Ok(())
    }

    /// Toggle ambient light state
    pub async fn toggle(&self) -> anyhow::Result<bool> {
        let current_enabled = self.is_enabled().await;
        let new_enabled = !current_enabled;
        self.set_enabled(new_enabled).await?;
        Ok(new_enabled)
    }
}
