//! Core module for Twinkle Linux.

pub mod config;

// Re-exports for convenience
pub use config::{
    AppConfig, AdvancedConfig, BehaviorConfig, ConfigManager, ConfigError, ConfigResult,
    GeneralConfig, MonitorConfig, UIConfig,
};
