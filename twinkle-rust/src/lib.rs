//! Twinkle Linux - GUI application for controlling external monitor brightness via DDC/CI on Linux.
//!
//! This library provides the core functionality for Twinkle Linux, including:
//! - DDC/CI backend for monitor communication
//! - GTK4-based UI components
//! - Configuration management
//! - System tray integration

pub mod core;
pub mod ddc;
pub mod ui;
pub mod utils;

// Re-exports for convenience
pub use core::{AppConfig, ConfigManager};
pub use ddc::{DDCError, DDCManager, Monitor};
pub use ui::{BrightnessPopup, TrayIcon};

/// Library version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Library name.
pub const NAME: &str = env!("CARGO_PKG_NAME");
