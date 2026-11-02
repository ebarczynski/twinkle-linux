//! UI module for Twinkle Linux.

pub mod brightness_popup;
pub mod tray_icon;
pub mod widgets;

use gtk4::prelude::*;

/// Apply the configured theme to GTK's native widgets.
///
/// The popup/dialog CSS forces our dark palette regardless, but native GTK
/// controls (dialog buttons, spinbuttons, comboboxes, switches) follow the
/// system theme — on a light-theme system with `theme: "system"` they render
/// as bright buttons with unreadable white text (see style.css).
/// This maps the config value onto `gtk-application-prefer-dark-theme`:
/// - "dark"   → always prefer dark
/// - "light"  → never
/// - "system" → follow the GNOME `color-scheme` setting (Ubuntu 24.04)
pub fn apply_theme_preference(theme: &str) {
    let prefer_dark = match theme {
        "dark" => true,
        "light" => false,
        _ => gnome_prefers_dark(),
    };
    if let Some(settings) = gtk4::Settings::default() {
        settings.set_gtk_application_prefer_dark_theme(prefer_dark);
    }
    tracing::info!(
        "Theme preference applied: {} (prefer_dark={})",
        theme,
        prefer_dark
    );
}

/// Read the GNOME color-scheme preference, tolerating non-GNOME desktops.
fn gnome_prefers_dark() -> bool {
    let Some(source) = gio::SettingsSchemaSource::default() else {
        return false;
    };
    if source
        .lookup("org.gnome.desktop.interface", false)
        .is_none()
    {
        return false;
    }
    let settings = gio::Settings::new("org.gnome.desktop.interface");
    settings.string("color-scheme").as_str() == "prefer-dark"
}
