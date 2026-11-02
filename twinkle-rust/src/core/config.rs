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
///
/// `#[serde(default)]` makes every missing field fall back to its default when
/// loading older or hand-edited config files, so adding new sections (e.g.
/// `schedule`) never breaks existing setups.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
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
    /// Day/night brightness schedule (sunrise/sunset)
    pub schedule: ScheduleConfig,
}

/// Default geographic coordinates: Wrocław, Poland.
fn default_latitude() -> f64 {
    51.107_9
}

fn default_longitude() -> f64 {
    17.038_5
}

/// Day/night brightness schedule configuration.
///
/// Brightness levels are cycled automatically based on computed sunrise/sunset
/// times for the configured location:
/// - `day`     — applied at sunrise
/// - `evening` — applied at sunset
/// - `night`   — applied `evening_duration_min` minutes after sunset
///
/// `ramp_minutes` smoothens each transition instead of jumping.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct ScheduleConfig {
    /// Enable automatic day/night cycling
    pub enabled: bool,
    /// Location latitude in decimal degrees (north positive)
    pub latitude: f64,
    /// Location longitude in decimal degrees (east positive)
    pub longitude: f64,
    /// Human-readable location name (shown in the tray menu)
    pub location_name: String,
    /// Brightness applied during daytime (sunrise → sunset)
    pub day_brightness: u16,
    /// Brightness applied in the evening (sunset → sunset + evening_duration_min)
    pub evening_brightness: u16,
    /// Brightness applied at night (after evening_duration_min past sunset, until sunrise)
    pub night_brightness: u16,
    /// Minutes after sunset when the evening phase ends and night begins
    pub evening_duration_min: u32,
    /// Minutes of smooth ramping at each phase transition (0 = instant jump)
    pub ramp_minutes: u32,
}

impl Default for ScheduleConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            latitude: default_latitude(),
            longitude: default_longitude(),
            location_name: "Wrocław, Poland".to_string(),
            day_brightness: 90,
            evening_brightness: 45,
            night_brightness: 15,
            evening_duration_min: 60,
            ramp_minutes: 30,
        }
    }
}

/// General settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GeneralConfig {
    /// Start on system login
    pub autostart: bool,
    /// Theme (system, light, dark)
    pub theme: String,
    /// Language code
    pub language: String,
}

/// Default minimum brightness to prevent backlight shutoff.
fn default_min_brightness() -> u16 {
    1
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
#[serde(default)]
pub struct UIConfig {
    /// Auto-hide popup delay in milliseconds (0 to disable)
    pub auto_hide_delay_ms: u32,
    /// Show monitor selector in popup
    pub show_monitor_selector: bool,
    /// Enable quick preset buttons
    pub enable_presets: bool,
    /// Custom preset values (brightness percentages)
    pub preset_values: Vec<u16>,
    /// Preset labels (corresponds to preset_values)
    pub preset_labels: Vec<String>,
    /// Software filter level (0 = off, 1-100 = dimming percentage via gamma)
    #[serde(default)]
    pub software_filter_level: u8,
}

impl Default for UIConfig {
    fn default() -> Self {
        Self {
            auto_hide_delay_ms: 3000,
            show_monitor_selector: false,
            enable_presets: true,
            preset_values: vec![10, 20, 40, 60, 80],
            preset_labels: vec![
                "Night".into(),
                "Dusk".into(),
                "Cloudy".into(),
                "Sunny".into(),
                "Full Sun".into(),
            ],
            software_filter_level: 0,
        }
    }
}

/// Behavior settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct BehaviorConfig {
    /// Debounce delay for brightness changes in milliseconds
    pub debounce_delay_ms: u32,
    /// Remember last brightness per monitor
    pub remember_brightness: bool,
    /// Restore brightness on startup
    pub restore_brightness: bool,
    /// Enable notifications
    pub enable_notifications: bool,
    /// Minimum brightness percentage to prevent backlight shutoff (0 = disabled, typical: 1)
    #[serde(default = "default_min_brightness")]
    pub min_brightness: u16,
}

impl Default for BehaviorConfig {
    fn default() -> Self {
        Self {
            debounce_delay_ms: 100,
            remember_brightness: true,
            restore_brightness: true,
            enable_notifications: true,
            min_brightness: 1, // Prevent 0% which kills backlight
        }
    }
}

/// Monitor-specific configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
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

impl Default for MonitorConfig {
    fn default() -> Self {
        Self::new(String::new(), String::new())
    }
}

impl MonitorConfig {
    /// Create a new monitor config.
    pub fn new(unique_id: String, display_name: String) -> Self {
        Self {
            unique_id,
            display_name,
            last_brightness: 100,
            brightness_presets: vec![10, 20, 40, 60, 80],
            enabled_vcp_codes: vec![0x10, 0x12, 0x14, 0x60, 0x62],
            custom_vcp_values: std::collections::HashMap::new(),
        }
    }
}

/// Advanced settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
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
        let base = directories::BaseDirs::new().ok_or_else(|| {
            ConfigError::InvalidValue("Failed to get base directories".to_string())
        })?;

        Ok(base.config_dir().to_path_buf())
    }

    /// Get or create monitor config for a monitor.
    pub fn get_or_create_monitor_config(
        &mut self,
        unique_id: &str,
        display_name: &str,
    ) -> &mut MonitorConfig {
        if let Some(pos) = self
            .config
            .monitors
            .iter()
            .position(|m| m.unique_id == unique_id)
        {
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
        if let Some(pos) = self
            .config
            .monitors
            .iter()
            .position(|m| m.unique_id == unique_id)
        {
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

    #[test]
    fn test_schedule_config_defaults() {
        let cfg = ScheduleConfig::default();
        assert!(cfg.enabled);
        // Wrocław, Poland
        assert!((cfg.latitude - 51.1079).abs() < 0.001);
        assert!((cfg.longitude - 17.0385).abs() < 0.001);
        assert_eq!(cfg.day_brightness, 90);
        assert_eq!(cfg.evening_brightness, 45);
        assert_eq!(cfg.night_brightness, 15);
    }

    /// Old config files (pre-schedule) must still load: missing sections
    /// fall back to defaults instead of failing deserialization.
    #[test]
    fn test_config_backward_compatible_load() {
        let legacy = r#"{
            "general": { "autostart": true, "theme": "dark", "language": "en_US" },
            "ui": {
                "auto_hide_delay_ms": 3000,
                "show_monitor_selector": false,
                "enable_presets": true,
                "preset_values": [10, 20, 40, 60, 80],
                "preset_labels": ["Night", "Dusk", "Cloudy", "Sunny", "Full Sun"]
            },
            "behavior": {
                "debounce_delay_ms": 100,
                "remember_brightness": true,
                "restore_brightness": true,
                "enable_notifications": true
            },
            "monitors": [],
            "advanced": {
                "command_timeout_secs": 5.0,
                "max_retries": 3,
                "debug_logging": false,
                "log_file_path": ""
            }
        }"#;
        let config: AppConfig = serde_json::from_str(legacy).unwrap();
        assert!(config.general.autostart);
        assert_eq!(config.general.theme, "dark");
        // Schedule section was absent — defaults must kick in
        assert!(config.schedule.enabled);
        assert_eq!(config.schedule.location_name, "Wrocław, Poland");
        // min_brightness was absent too
        assert_eq!(config.behavior.min_brightness, 1);
    }

    /// Unknown fields (from a future version) must not break loading either.
    #[test]
    fn test_config_unknown_fields_ignored() {
        let futuristic = r#"{ "some_future_field": 42, "schedule": {} }"#;
        let config: AppConfig = serde_json::from_str(futuristic).unwrap();
        assert_eq!(config.schedule, ScheduleConfig::default());
    }
}
