//! Configuration management for Twinkle Linux.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

/// Result type alias for configuration operations.
pub type ConfigResult<T> = Result<T, ConfigError>;

/// Errors that can occur during configuration operations.
#[derive(Error, Debug)]
pub enum ConfigError {
    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization error.
    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    /// Config file not found.
    #[error("Config file not found: {0}")]
    NotFound(PathBuf),

    /// Invalid config value.
    #[error("Invalid config value: {0}")]
    InvalidValue(String),
}

/// Application configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// General settings
    pub general: GeneralConfig,
    /// UI settings
    pub ui: UIConfig,
    /// Behavior settings
    pub behavior: BehaviorConfig,
    /// Monitor-specific settings
    pub monitors: Vec<MonitorConfig>,
    /// Advanced settings
    pub advanced: AdvancedConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            ui: UIConfig::default(),
            behavior: BehaviorConfig::default(),
            monitors: Vec::new(),
            advanced: AdvancedConfig::default(),
        }
    }
}

/// General settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    /// Start on system login
    pub autostart: bool,
    /// Theme (system, light, dark)
    pub theme: String,
    /// Language code
    pub language: String,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            autostart: false,
            theme: "system".to_string(),
            language: "en_US".to_string(),
        }
    }
}

/// UI settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIConfig {
    /// Auto-hide popup delay in milliseconds (0 to disable)
    pub auto_hide_delay_ms: u32,
    /// Show monitor selector in popup
    pub show_monitor_selector: bool,
    /// Enable quick preset buttons
    pub enable_presets: bool,
    /// Custom preset values
    pub preset_values: Vec<u16>,
}

impl Default for UIConfig {
    fn default() -> Self {
        Self {
            auto_hide_delay_ms: 3000,
            show_monitor_selector: false,
            enable_presets: true,
            preset_values: vec![20, 40, 60, 80, 100],
        }
    }
}

/// Behavior settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorConfig {
    /// Debounce delay for brightness changes in milliseconds
    pub debounce_delay_ms: u32,
    /// Remember last brightness per monitor
    pub remember_brightness: bool,
    /// Restore brightness on startup
    pub restore_brightness: bool,
    /// Enable notifications
    pub enable_notifications: bool,
}

impl Default for BehaviorConfig {
    fn default() -> Self {
        Self {
            debounce_delay_ms: 100,
            remember_brightness: true,
            restore_brightness: true,
            enable_notifications: true,
        }
    }
}

/// Monitor-specific configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorConfig {
    /// Unique ID of the monitor
    pub unique_id: String,
    /// Display name
    pub display_name: String,
    /// Last brightness value
    pub last_brightness: u16,
    /// Brightness preset values
    pub brightness_presets: Vec<u16>,
    /// Enabled VCP codes for this monitor
    pub enabled_vcp_codes: Vec<u8>,
    /// Custom VCP values
    pub custom_vcp_values: std::collections::HashMap<u8, u16>,
}

impl MonitorConfig {
    /// Create a new monitor config.
    pub fn new(unique_id: String, display_name: String) -> Self {
        Self {
            unique_id,
            display_name,
            last_brightness: 100,
            brightness_presets: vec![20, 40, 60, 80, 100],
            enabled_vcp_codes: vec![0x10, 0x12, 0x14, 0x60, 0x62],
            custom_vcp_values: std::collections::HashMap::new(),
        }
    }
}

/// Advanced settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedConfig {
    /// Command timeout in seconds
    pub command_timeout_secs: f64,
    /// Maximum retry attempts
    pub max_retries: u32,
    /// Enable debug logging
    pub debug_logging: bool,
    /// Log file path (empty for default)
    pub log_file_path: String,
}

impl Default for AdvancedConfig {
    fn default() -> Self {
        Self {
            command_timeout_secs: 5.0,
            max_retries: 3,
            debug_logging: false,
            log_file_path: String::new(),
        }
    }
}

/// Configuration manager.
pub struct ConfigManager {
    /// Configuration data
    config: AppConfig,
    /// Path to the config file
    config_path: PathBuf,
    /// Whether the config has been modified
    modified: bool,
}

impl ConfigManager {
    /// Create a new ConfigManager with default settings.
    pub fn new() -> ConfigResult<Self> {
        let config_dir = Self::get_config_dir()?;
        let config_path = config_dir.join("twinkle-linux").join("config.json");

        // Create config directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        Ok(Self {
            config: AppConfig::default(),
            config_path,
            modified: false,
        })
    }

    /// Load configuration from file.
    pub fn load(&mut self) -> ConfigResult<()> {
        if !self.config_path.exists() {
            // Use default config if file doesn't exist
            self.config = AppConfig::default();
            return Ok(());
        }

        let content = std::fs::read_to_string(&self.config_path)?;
        self.config = serde_json::from_str(&content)?;
        self.modified = false;

        tracing::info!("Loaded configuration from {:?}", self.config_path);
        Ok(())
    }

    /// Save configuration to file.
    pub fn save(&mut self) -> ConfigResult<()> {
        let content = serde_json::to_string_pretty(&self.config)?;
        std::fs::write(&self.config_path, content)?;
        self.modified = false;

        tracing::info!("Saved configuration to {:?}", self.config_path);
        Ok(())
    }

    /// Get the configuration.
    pub fn config(&self) -> &AppConfig {
        &self.config
    }

    /// Get mutable reference to the configuration.
    pub fn config_mut(&mut self) -> &mut AppConfig {
        self.modified = true;
        &mut self.config
    }

    /// Check if the configuration has been modified.
    pub fn is_modified(&self) -> bool {
        self.modified
    }

    /// Get the config file path.
    pub fn config_path(&self) -> &PathBuf {
        &self.config_path
    }

    /// Get the XDG config directory.
    fn get_config_dir() -> ConfigResult<PathBuf> {
        let base = directories::BaseDirs::new()
            .ok_or_else(|| ConfigError::InvalidValue("Failed to get base directories".to_string()))?;

        Ok(base.config_dir().to_path_buf())
    }

    /// Get or create monitor config for a monitor.
    pub fn get_or_create_monitor_config(&mut self, unique_id: &str, display_name: &str) -> &mut MonitorConfig {
        if let Some(pos) = self.config.monitors.iter().position(|m| m.unique_id == unique_id) {
            &mut self.config.monitors[pos]
        } else {
            self.modified = true;
            self.config.monitors.push(MonitorConfig::new(
                unique_id.to_string(),
                display_name.to_string(),
            ));
            self.config.monitors.last_mut().unwrap()
        }
    }

    /// Remove monitor config for a disconnected monitor.
    pub fn remove_monitor_config(&mut self, unique_id: &str) {
        if let Some(pos) = self.config.monitors.iter().position(|m| m.unique_id == unique_id) {
            self.config.monitors.remove(pos);
            self.modified = true;
        }
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new().unwrap_or_else(|e| {
            tracing::warn!("Failed to create ConfigManager: {}, using defaults", e);
            Self {
                config: AppConfig::default(),
                config_path: PathBuf::from("config.json"),
                modified: false,
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_config_default() {
        let config = AppConfig::default();
        assert_eq!(config.general.theme, "system");
        assert_eq!(config.ui.auto_hide_delay_ms, 3000);
    }

    #[test]
    fn test_monitor_config_new() {
        let config = MonitorConfig::new("test_id".to_string(), "Test Monitor".to_string());
        assert_eq!(config.unique_id, "test_id");
        assert_eq!(config.display_name, "Test Monitor");
        assert_eq!(config.last_brightness, 100);
    }

    #[test]
    fn test_config_serialization() {
        let config = AppConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: AppConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config.general.theme, deserialized.general.theme);
    }
}
