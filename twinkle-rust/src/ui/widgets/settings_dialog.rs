//! Settings dialog widget.

use crate::core::config::{AppConfig, ConfigManager};
use gtk4::prelude::*;
use gtk4::{Adjustment, Box, ComboBoxText, Dialog, Label, Orientation, SpinButton, Switch};
use std::sync::Arc;

/// Settings dialog.
pub struct SettingsDialog {
    /// The dialog widget
    dialog: Dialog,
    /// Config manager
    config_manager: Arc<tokio::sync::Mutex<ConfigManager>>,
    /// Original config for comparison
    original_config: AppConfig,
}

impl SettingsDialog {
    /// Create a new settings dialog.
    pub async fn new(
        _parent: &impl IsA<gtk4::Window>,
        config_manager: Arc<tokio::sync::Mutex<ConfigManager>>,
    ) -> Self {
        let dialog = Dialog::builder()
            .title("Twinkle Linux Settings")
            .modal(true)
            .default_width(600)
            .default_height(500)
            .build();

        // Add buttons
        dialog.add_button("Cancel", gtk4::ResponseType::Cancel);
        dialog.add_button("Apply", gtk4::ResponseType::Apply);
        dialog.add_button("OK", gtk4::ResponseType::Ok);

        // Get current config
        let config = config_manager.lock().await;
        let original_config = config.config().clone();
        drop(config);

        let settings = Self {
            dialog,
            config_manager,
            original_config,
        };

        settings.build_ui().await;

        settings
    }

    /// Build the settings UI.
    async fn build_ui(&self) {
        let content_area = self.dialog.content_area();
        content_area.set_margin_top(12);
        content_area.set_margin_bottom(12);
        content_area.set_margin_start(12);
        content_area.set_margin_end(12);

        // Create notebook for tabs
        let notebook = gtk4::Notebook::new();

        // General tab
        let general_tab = self.build_general_tab();
        notebook.append_page(&general_tab, Some(&Label::new(Some("General"))));

        // UI tab
        let ui_tab = self.build_ui_tab();
        notebook.append_page(&ui_tab, Some(&Label::new(Some("UI"))));

        // Behavior tab
        let behavior_tab = self.build_behavior_tab();
        notebook.append_page(&behavior_tab, Some(&Label::new(Some("Behavior"))));

        // Advanced tab
        let advanced_tab = self.build_advanced_tab();
        notebook.append_page(&advanced_tab, Some(&Label::new(Some("Advanced"))));

        content_area.append(&notebook);
    }

    /// Build the general settings tab.
    fn build_general_tab(&self) -> Box {
        let container = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(16)
            .margin_top(12)
            .margin_bottom(12)
            .margin_start(12)
            .margin_end(12)
            .build();

        // Auto-start group
        let autostart_label = Label::builder()
            .label("<b>Startup</b>")
            .use_markup(true)
            .halign(gtk4::Align::Start)
            .build();

        let autostart_switch = Switch::builder()
            .active(self.original_config.general.autostart)
            .build();

        let autostart_desc = Label::builder()
            .label("Start Twinkle Linux automatically when you log in")
            .wrap(true)
            .halign(gtk4::Align::Start)
            .build();

        container.append(&autostart_label);
        container.append(&autostart_switch);
        container.append(&autostart_desc);

        // Appearance group
        let appearance_label = Label::builder()
            .label("<b>Appearance</b>")
            .use_markup(true)
            .halign(gtk4::Align::Start)
            .margin_top(16)
            .build();

        let theme_label = Label::builder()
            .label("Theme:")
            .halign(gtk4::Align::Start)
            .build();

        let theme_combo = ComboBoxText::new();
        theme_combo.append(Some("system"), "Auto (follow system)");
        theme_combo.append(Some("light"), "Light");
        theme_combo.append(Some("dark"), "Dark");
        theme_combo.set_active_id(Some(&self.original_config.general.theme));

        let theme_box = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .build();
        theme_box.append(&theme_label);
        theme_box.append(&theme_combo);

        let language_label = Label::builder()
            .label("Language:")
            .halign(gtk4::Align::Start)
            .build();

        let language_combo = ComboBoxText::new();
        language_combo.append(Some("en_US"), "English (US)");
        language_combo.append(Some("pl_PL"), "Polish");
        language_combo.set_active_id(Some(&self.original_config.general.language));
        language_combo.set_sensitive(false); // Placeholder for future i18n

        let language_box = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .build();
        language_box.append(&language_label);
        language_box.append(&language_combo);

        container.append(&appearance_label);
        container.append(&theme_box);
        container.append(&language_box);

        container
    }

    /// Build the UI settings tab.
    fn build_ui_tab(&self) -> Box {
        let container = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(16)
            .margin_top(12)
            .margin_bottom(12)
            .margin_start(12)
            .margin_end(12)
            .build();

        // Popup settings
        let popup_label = Label::builder()
            .label("<b>Brightness Popup</b>")
            .use_markup(true)
            .halign(gtk4::Align::Start)
            .build();

        let auto_hide_label = Label::builder()
            .label("Auto-hide delay (ms, 0 to disable):")
            .halign(gtk4::Align::Start)
            .build();

        let auto_hide_adj = Adjustment::new(
            self.original_config.ui.auto_hide_delay_ms as f64,
            0.0,
            10000.0,
            100.0,
            500.0,
            0.0,
        );

        let auto_hide_spin = SpinButton::builder()
            .adjustment(&auto_hide_adj)
            .build();

        let auto_hide_box = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .build();
        auto_hide_box.append(&auto_hide_label);
        auto_hide_box.append(&auto_hide_spin);

        let show_monitor_switch = Switch::builder()
            .active(self.original_config.ui.show_monitor_selector)
            .build();

        let show_monitor_label = Label::builder()
            .label("Show monitor selector in popup")
            .halign(gtk4::Align::Start)
            .build();

        let show_monitor_box = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .build();
        show_monitor_box.append(&show_monitor_switch);
        show_monitor_box.append(&show_monitor_label);

        let enable_presets_switch = Switch::builder()
            .active(self.original_config.ui.enable_presets)
            .build();

        let enable_presets_label = Label::builder()
            .label("Enable quick preset buttons")
            .halign(gtk4::Align::Start)
            .build();

        let enable_presets_box = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .build();
        enable_presets_box.append(&enable_presets_switch);
        enable_presets_box.append(&enable_presets_label);

        container.append(&popup_label);
        container.append(&auto_hide_box);
        container.append(&show_monitor_box);
        container.append(&enable_presets_box);

        container
    }

    /// Build the behavior settings tab.
    fn build_behavior_tab(&self) -> Box {
        let container = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(16)
            .margin_top(12)
            .margin_bottom(12)
            .margin_start(12)
            .margin_end(12)
            .build();

        // Brightness behavior
        let brightness_label = Label::builder()
            .label("<b>Brightness Behavior</b>")
            .use_markup(true)
            .halign(gtk4::Align::Start)
            .build();

        let debounce_label = Label::builder()
            .label("Debounce delay (ms):")
            .halign(gtk4::Align::Start)
            .build();

        let debounce_adj = Adjustment::new(
            self.original_config.behavior.debounce_delay_ms as f64,
            0.0,
            1000.0,
            10.0,
            50.0,
            0.0,
        );

        let debounce_spin = SpinButton::builder()
            .adjustment(&debounce_adj)
            .build();

        let debounce_box = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .build();
        debounce_box.append(&debounce_label);
        debounce_box.append(&debounce_spin);

        let remember_brightness_switch = Switch::builder()
            .active(self.original_config.behavior.remember_brightness)
            .build();

        let remember_brightness_label = Label::builder()
            .label("Remember last brightness per monitor")
            .halign(gtk4::Align::Start)
            .build();

        let remember_brightness_box = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .build();
        remember_brightness_box.append(&remember_brightness_switch);
        remember_brightness_box.append(&remember_brightness_label);

        let restore_brightness_switch = Switch::builder()
            .active(self.original_config.behavior.restore_brightness)
            .build();

        let restore_brightness_label = Label::builder()
            .label("Restore brightness on startup")
            .halign(gtk4::Align::Start)
            .build();

        let restore_brightness_box = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .build();
        restore_brightness_box.append(&restore_brightness_switch);
        restore_brightness_box.append(&restore_brightness_label);

        container.append(&brightness_label);
        container.append(&debounce_box);
        container.append(&remember_brightness_box);
        container.append(&restore_brightness_box);

        container
    }

    /// Build the advanced settings tab.
    fn build_advanced_tab(&self) -> Box {
        let container = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(16)
            .margin_top(12)
            .margin_bottom(12)
            .margin_start(12)
            .margin_end(12)
            .build();

        // DDC settings
        let ddc_label = Label::builder()
            .label("<b>DDC/CI Settings</b>")
            .use_markup(true)
            .halign(gtk4::Align::Start)
            .build();

        let timeout_label = Label::builder()
            .label("Command timeout (seconds):")
            .halign(gtk4::Align::Start)
            .build();

        let timeout_adj = Adjustment::new(
            self.original_config.advanced.command_timeout_secs,
            1.0,
            30.0,
            0.5,
            1.0,
            0.0,
        );

        let timeout_spin = SpinButton::builder()
            .adjustment(&timeout_adj)
            .digits(1)
            .build();

        let timeout_box = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .build();
        timeout_box.append(&timeout_label);
        timeout_box.append(&timeout_spin);

        let retries_label = Label::builder()
            .label("Maximum retry attempts:")
            .halign(gtk4::Align::Start)
            .build();

        let retries_adj = Adjustment::new(
            self.original_config.advanced.max_retries as f64,
            0.0,
            10.0,
            1.0,
            1.0,
            0.0,
        );

        let retries_spin = SpinButton::builder()
            .adjustment(&retries_adj)
            .build();

        let retries_box = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .build();
        retries_box.append(&retries_label);
        retries_box.append(&retries_spin);

        // Debug settings
        let debug_label = Label::builder()
            .label("<b>Debug</b>")
            .use_markup(true)
            .halign(gtk4::Align::Start)
            .margin_top(16)
            .build();

        let debug_logging_switch = Switch::builder()
            .active(self.original_config.advanced.debug_logging)
            .build();

        let debug_logging_label = Label::builder()
            .label("Enable debug logging")
            .halign(gtk4::Align::Start)
            .build();

        let debug_logging_box = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .build();
        debug_logging_box.append(&debug_logging_switch);
        debug_logging_box.append(&debug_logging_label);

        container.append(&ddc_label);
        container.append(&timeout_box);
        container.append(&retries_box);
        container.append(&debug_label);
        container.append(&debug_logging_box);

        container
    }

    /// Run the dialog and return the response.
    pub fn run(&self) {
        self.dialog.present();
    }

    /// Get the dialog widget.
    pub fn widget(&self) -> &Dialog {
        &self.dialog
    }
}
