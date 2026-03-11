//! UI module for Twinkle Linux.

pub mod brightness_popup;
pub mod tray_icon;
pub mod widgets;

// Re-exports for convenience
pub use brightness_popup::BrightnessPopup;
pub use tray_icon::{setup_tray_actions, TrayIcon};
