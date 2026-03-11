//! System tray icon implementation.

use crate::core::config::ConfigManager;
use crate::ddc::DDCManager;
use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Image, MenuItem, PopoverMenu, Widget};
use std::sync::Arc;

/// System tray icon.
pub struct TrayIcon {
    /// GTK application
    app: Application,
    /// Status icon (using a workaround since GTK4 doesn't have StatusIcon)
    status_button: gtk4::MenuButton,
    /// Popover menu for the tray icon
    popover: PopoverMenu,
    /// DDC manager
    ddc_manager: Arc<DDCManager>,
    /// Config manager
    config_manager: Arc<tokio::sync::Mutex<ConfigManager>>,
}

impl TrayIcon {
    /// Create a new tray icon.
    pub async fn new(
        app: Application,
        ddc_manager: Arc<DDCManager>,
        config_manager: Arc<tokio::sync::Mutex<ConfigManager>>,
    ) -> Self {
        // Create a menu button to simulate tray icon
        let status_button = gtk4::MenuButton::builder()
            .icon_name("display-brightness-symbolic")
            .tooltip_text("Twinkle Linux - Monitor Brightness Control")
            .build();

        // Create popover menu
        let popover = PopoverMenu::new();

        // Build the menu
        Self::build_menu(&popover);

        status_button.set_popover(Some(&popover));

        Self {
            app,
            status_button,
            popover,
            ddc_manager,
            config_manager,
        }
    }

    /// Build the tray icon menu.
    fn build_menu(popover: &PopoverMenu) {
        let menu = gio::Menu::new();

        // Brightness section
        let brightness_section = gio::Menu::new();
        brightness_section.append(Some("Brightness Control"), Some("app.show-brightness"));
        menu.append_section(None, &brightness_section);

        // Settings section
        let settings_section = gio::Menu::new();
        settings_section.append(Some("Settings"), Some("app.show-settings"));
        settings_section.append(Some("About"), Some("app.show-about"));
        menu.append_section(None, &settings_section);

        // Quit section
        let quit_section = gio::Menu::new();
        quit_section.append(Some("Quit"), Some("app.quit"));
        menu.append_section(None, &quit_section);

        popover.set_menu_model(Some(&menu));
    }

    /// Get the status button widget.
    pub fn widget(&self) -> &gtk4::MenuButton {
        &self.status_button
    }

    /// Update the tray icon state.
    pub async fn update_state(&self) {
        // Update icon based on current state
        let monitors = self.ddc_manager.get_monitors().await;
        if monitors.is_empty() {
            self.status_button.set_icon_name("dialog-warning-symbolic");
            self.status_button.set_tooltip_text(Some("No monitors detected"));
        } else {
            self.status_button.set_icon_name("display-brightness-symbolic");
            let tooltip = format!(
                "Twinkle Linux - {} monitor(s) detected",
                monitors.len()
            );
            self.status_button.set_tooltip_text(Some(&tooltip));
        }
    }

    /// Show a notification.
    pub fn show_notification(&self, title: &str, message: &str) {
        // GTK4 doesn't have built-in notifications
        // We'll use the application to show a dialog or log
        tracing::info!("Notification: {} - {}", title, message);
    }
}

/// Create the application actions for the tray icon.
pub fn setup_tray_actions(app: &Application) {
    // Show brightness action
    let show_brightness = gio::SimpleAction::new("show-brightness", None);
    show_brightness.connect_activate(|_, _| {
        tracing::info!("Show brightness popup");
        // TODO: Show brightness popup
    });
    app.add_action(&show_brightness);

    // Show settings action
    let show_settings = gio::SimpleAction::new("show-settings", None);
    show_settings.connect_activate(|_, _| {
        tracing::info!("Show settings dialog");
        // TODO: Show settings dialog
    });
    app.add_action(&show_settings);

    // Show about action
    let show_about = gio::SimpleAction::new("show-about", None);
    show_about.connect_activate(|_, _| {
        tracing::info!("Show about dialog");
        // TODO: Show about dialog
    });
    app.add_action(&show_about);

    // Quit action
    let quit = gio::SimpleAction::new("quit", None);
    quit.connect_activate(|_, app| {
        app.quit();
    });
    app.add_action(&quit);
}
