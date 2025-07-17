use anyhow::{anyhow, Result};
use paris::{info, warn};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct AutoStartConfig {
    pub enabled: bool,
}

impl Default for AutoStartConfig {
    fn default() -> Self {
        Self { enabled: false }
    }
}

pub struct AutoStartManager;

impl AutoStartManager {
    /// Get the LaunchAgent plist file path for macOS
    fn get_plist_path() -> Result<PathBuf> {
        let home_dir = dirs::home_dir().ok_or_else(|| anyhow!("Could not find home directory"))?;

        let launch_agents_dir = home_dir.join("Library/LaunchAgents");

        // Create the LaunchAgents directory if it doesn't exist
        if !launch_agents_dir.exists() {
            fs::create_dir_all(&launch_agents_dir)?;
        }

        Ok(launch_agents_dir.join("cc.ivanli.ambient-light.desktop.plist"))
    }

    /// Get the current executable path
    fn get_executable_path() -> Result<String> {
        let exe_path = std::env::current_exe()?;
        Ok(exe_path.to_string_lossy().to_string())
    }

    /// Create the LaunchAgent plist content
    fn create_plist_content() -> Result<String> {
        let exe_path = Self::get_executable_path()?;

        let plist_content = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>cc.ivanli.ambient-light.desktop</string>
    <key>ProgramArguments</key>
    <array>
        <string>{}</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <false/>
    <key>LaunchOnlyOnce</key>
    <true/>
    <key>StandardOutPath</key>
    <string>/tmp/ambient-light-control.out</string>
    <key>StandardErrorPath</key>
    <string>/tmp/ambient-light-control.err</string>
</dict>
</plist>"#,
            exe_path
        );

        Ok(plist_content)
    }

    /// Check if auto start is currently enabled
    pub fn is_enabled() -> Result<bool> {
        let plist_path = Self::get_plist_path()?;
        Ok(plist_path.exists())
    }

    /// Enable auto start
    pub fn enable() -> Result<()> {
        info!("Enabling auto start...");

        let plist_path = Self::get_plist_path()?;
        let plist_content = Self::create_plist_content()?;

        // Write the plist file
        fs::write(&plist_path, plist_content)?;

        info!(
            "Auto start enabled successfully. Plist file created at: {:?}",
            plist_path
        );
        Ok(())
    }

    /// Disable auto start
    pub fn disable() -> Result<()> {
        info!("Disabling auto start...");

        let plist_path = Self::get_plist_path()?;

        if plist_path.exists() {
            fs::remove_file(&plist_path)?;
            info!(
                "Auto start disabled successfully. Plist file removed from: {:?}",
                plist_path
            );
        } else {
            warn!(
                "Auto start was already disabled. Plist file not found at: {:?}",
                plist_path
            );
        }

        Ok(())
    }

    /// Toggle auto start setting
    pub fn toggle() -> Result<bool> {
        let current_state = Self::is_enabled()?;

        if current_state {
            Self::disable()?;
            Ok(false)
        } else {
            Self::enable()?;
            Ok(true)
        }
    }

    /// Set auto start state
    pub fn set_enabled(enabled: bool) -> Result<()> {
        if enabled {
            Self::enable()
        } else {
            Self::disable()
        }
    }

    /// Get auto start configuration
    pub fn get_config() -> Result<AutoStartConfig> {
        let enabled = Self::is_enabled()?;
        Ok(AutoStartConfig { enabled })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plist_content_creation() {
        let content = AutoStartManager::create_plist_content();
        assert!(content.is_ok());

        let content = content.unwrap();
        assert!(content.contains("cc.ivanli.ambient-light.desktop"));
        assert!(content.contains("RunAtLoad"));
        assert!(content.contains("<true/>"));
    }

    #[test]
    fn test_plist_path() {
        let path = AutoStartManager::get_plist_path();
        assert!(path.is_ok());

        let path = path.unwrap();
        assert!(path.to_string_lossy().contains("LaunchAgents"));
        assert!(path
            .to_string_lossy()
            .contains("cc.ivanli.ambient-light.desktop.plist"));
    }
}
