//! UI widgets for Twinkle Linux.

pub mod brightness_slider;
pub mod settings_dialog;
pub mod vcp_controls;

// Re-exports for convenience
pub use brightness_slider::BrightnessSlider;
pub use settings_dialog::SettingsDialog;
pub use vcp_controls::{VCPControlSection, VCPControlsContainer};
