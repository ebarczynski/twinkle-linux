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
    /// General tab widgets
    autostart_switch: Switch,
    theme_combo: ComboBoxText,
    /// UI tab widgets
    auto_hide_spin: SpinButton,
    show_monitor_switch: Switch,
    enable_presets_switch: Switch,
    /// Behavior tab widgets
    debounce_spin: SpinButton,
    remember_brightness_switch: Switch,
    restore_brightness_switch: Switch,
    /// Advanced tab widgets
    timeout_spin: SpinButton,
    retries_spin: SpinButton,
    debug_logging_switch: Switch,
}

impl SettingsDialog {
    /// Create a new settings dialog.
    pub async fn new(
        parent: &impl IsA<gtk4::Window>,
        config_manager: Arc<tokio::sync::Mutex<ConfigManager>>,
    ) -> Self {
        let dialog = Dialog::builder()
            .title("Twinkle Linux Settings")
            .modal(true)
            .transient_for(parent)
            .default_width(500)
            .default_height(450)
            .build();

        // Add buttons
        dialog.add_button("_Cancel", gtk4::ResponseType::Cancel);
        dialog.add_button("_Apply", gtk4::ResponseType::Apply);
        dialog.add_button("_OK", gtk4::ResponseType::Ok);

        // Get current config
        let config = config_manager.lock().await;
        let original_config = config.config().clone();
        drop(config);

        // Build tabs and capture widget references
        let (general_tab, autostart_switch, theme_combo) = Self::build_general_tab(&original_config);
        let (ui_tab, auto_hide_spin, show_monitor_switch, enable_presets_switch) = Self::build_ui_tab(&original_config);
        let (behavior_tab, debounce_spin, remember_brightness_switch, restore_brightness_switch) = Self::build_behavior_tab(&original_config);
        let (advanced_tab, timeout_spin, retries_spin, debug_logging_switch) = Self::build_advanced_tab(&original_config);

        // Create notebook
        let notebook = gtk4::Notebook::new();
        notebook.set_margin_top(8);
        notebook.set_margin_bottom(8);
        notebook.set_margin_start(8);
        notebook.set_margin_end(8);
        notebook.append_page(&general_tab, Some(&Label::new(Some("General"))));
        notebook.append_page(&ui_tab, Some(&Label::new(Some("UI"))));
        notebook.append_page(&behavior_tab, Some(&Label::new(Some("Behavior"))));
        notebook.append_page(&advanced_tab, Some(&Label::new(Some("Advanced"))));

        let content_area = dialog.content_area();
        content_area.append(&notebook);

        let settings = Self {
            dialog,
            config_manager,
            autostart_switch,
            theme_combo,
            auto_hide_spin,
            show_monitor_switch,
            enable_presets_switch,
            debounce_spin,
            remember_brightness_switch,
            restore_brightness_switch,
            timeout_spin,
            retries_spin,
            debug_logging_switch,
        };

        settings.connect_signals();
        settings
    }

    fn connect_signals(&self) {
        let config_manager = self.config_manager.clone();
        let autostart_switch = self.autostart_switch.clone();
        let theme_combo = self.theme_combo.clone();
        let auto_hide_spin = self.auto_hide_spin.clone();
        let show_monitor_switch = self.show_monitor_switch.clone();
        let enable_presets_switch = self.enable_presets_switch.clone();
        let debounce_spin = self.debounce_spin.clone();
        let remember_brightness_switch = self.remember_brightness_switch.clone();
        let restore_brightness_switch = self.restore_brightness_switch.clone();
        let timeout_spin = self.timeout_spin.clone();
        let retries_spin = self.retries_spin.clone();
        let debug_logging_switch = self.debug_logging_switch.clone();

        self.dialog.connect_response(move |dialog, response| {
            match response {
                gtk4::ResponseType::Ok | gtk4::ResponseType::Apply => {
                    // Read widget values and save config
                    let mut mgr = config_manager.blocking_lock();
                    let config = mgr.config_mut();
                    config.general.autostart = autostart_switch.is_active();
                    config.general.theme = theme_combo.active_id().unwrap_or_default().to_string();
                    config.ui.auto_hide_delay_ms = auto_hide_spin.value() as u32;
                    config.ui.show_monitor_selector = show_monitor_switch.is_active();
                    config.ui.enable_presets = enable_presets_switch.is_active();
                    config.behavior.debounce_delay_ms = debounce_spin.value() as u32;
                    config.behavior.remember_brightness = remember_brightness_switch.is_active();
                    config.behavior.restore_brightness = restore_brightness_switch.is_active();
                    config.advanced.command_timeout_secs = timeout_spin.value();
                    config.advanced.max_retries = retries_spin.value() as u32;
                    config.advanced.debug_logging = debug_logging_switch.is_active();

                    if let Err(e) = mgr.save() {
                        tracing::error!("Failed to save settings: {}", e);
                    } else {
                        tracing::info!("Settings saved successfully");
                    }
                }
                _ => {}
            }

            if response != gtk4::ResponseType::Apply {
                dialog.close();
            }
        });
    }

    /// Show the dialog.
    pub fn present(&self) {
        self.dialog.present();
    }

    fn make_section_label(text: &str) -> Label {
        Label::builder()
            .label(text)
            .use_markup(true)
            .halign(gtk4::Align::Start)
            .build()
    }

    fn make_row(switch: &Switch, label_text: &str) -> Box {
        let row = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .build();
        row.append(switch);
        let label = Label::builder()
            .label(label_text)
            .halign(gtk4::Align::Start)
            .wrap(true)
            .build();
        row.append(&label);
        row
    }

    fn build_general_tab(config: &AppConfig) -> (Box, Switch, ComboBoxText) {
        let container = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(12)
            .margin_top(12)
            .margin_bottom(12)
            .margin_start(12)
            .margin_end(12)
            .build();

        container.append(&Self::make_section_label("<b>Startup</b>"));

        let autostart_switch = Switch::builder()
            .active(config.general.autostart)
            .build();
        container.append(&Self::make_row(&autostart_switch, "Start automatically on login"));

        container.append(&Self::make_section_label("<b>Appearance</b>"));

        let theme_label = Label::builder().label("Theme:").halign(gtk4::Align::Start).build();
        let theme_combo = ComboBoxText::new();
        theme_combo.append(Some("system"), "System");
        theme_combo.append(Some("light"), "Light");
        theme_combo.append(Some("dark"), "Dark");
        theme_combo.set_active_id(Some(&config.general.theme));

        let theme_row = Box::builder().orientation(Orientation::Horizontal).spacing(8).build();
        theme_row.append(&theme_label);
        theme_row.append(&theme_combo);
        container.append(&theme_row);

        (container, autostart_switch, theme_combo)
    }

    fn build_ui_tab(config: &AppConfig) -> (Box, SpinButton, Switch, Switch) {
        let container = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(12)
            .margin_top(12)
            .margin_bottom(12)
            .margin_start(12)
            .margin_end(12)
            .build();

        container.append(&Self::make_section_label("<b>Brightness Popup</b>"));

        let auto_hide_label = Label::builder().label("Auto-hide delay (ms, 0 = disabled):").halign(gtk4::Align::Start).build();
        let auto_hide_adj = Adjustment::new(config.ui.auto_hide_delay_ms as f64, 0.0, 30000.0, 100.0, 500.0, 0.0);
        let auto_hide_spin = SpinButton::builder().adjustment(&auto_hide_adj).build();
        let auto_hide_row = Box::builder().orientation(Orientation::Horizontal).spacing(8).build();
        auto_hide_row.append(&auto_hide_label);
        auto_hide_row.append(&auto_hide_spin);
        container.append(&auto_hide_row);

        let show_monitor_switch = Switch::builder().active(config.ui.show_monitor_selector).build();
        container.append(&Self::make_row(&show_monitor_switch, "Show monitor selector"));

        let enable_presets_switch = Switch::builder().active(config.ui.enable_presets).build();
        container.append(&Self::make_row(&enable_presets_switch, "Show preset buttons"));

        (container, auto_hide_spin, show_monitor_switch, enable_presets_switch)
    }

    fn build_behavior_tab(config: &AppConfig) -> (Box, SpinButton, Switch, Switch) {
        let container = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(12)
            .margin_top(12)
            .margin_bottom(12)
            .margin_start(12)
            .margin_end(12)
            .build();

        container.append(&Self::make_section_label("<b>Brightness</b>"));

        let debounce_label = Label::builder().label("Debounce delay (ms):").halign(gtk4::Align::Start).build();
        let debounce_adj = Adjustment::new(config.behavior.debounce_delay_ms as f64, 0.0, 1000.0, 10.0, 50.0, 0.0);
        let debounce_spin = SpinButton::builder().adjustment(&debounce_adj).build();
        let debounce_row = Box::builder().orientation(Orientation::Horizontal).spacing(8).build();
        debounce_row.append(&debounce_label);
        debounce_row.append(&debounce_spin);
        container.append(&debounce_row);

        let remember_brightness_switch = Switch::builder().active(config.behavior.remember_brightness).build();
        container.append(&Self::make_row(&remember_brightness_switch, "Remember brightness per monitor"));

        let restore_brightness_switch = Switch::builder().active(config.behavior.restore_brightness).build();
        container.append(&Self::make_row(&restore_brightness_switch, "Restore brightness on startup"));

        (container, debounce_spin, remember_brightness_switch, restore_brightness_switch)
    }

    fn build_advanced_tab(config: &AppConfig) -> (Box, SpinButton, SpinButton, Switch) {
        let container = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(12)
            .margin_top(12)
            .margin_bottom(12)
            .margin_start(12)
            .margin_end(12)
            .build();

        container.append(&Self::make_section_label("<b>DDC/CI</b>"));

        let timeout_label = Label::builder().label("Command timeout (s):").halign(gtk4::Align::Start).build();
        let timeout_adj = Adjustment::new(config.advanced.command_timeout_secs, 1.0, 30.0, 0.5, 1.0, 0.0);
        let timeout_spin = SpinButton::builder().adjustment(&timeout_adj).digits(1).build();
        let timeout_row = Box::builder().orientation(Orientation::Horizontal).spacing(8).build();
        timeout_row.append(&timeout_label);
        timeout_row.append(&timeout_spin);
        container.append(&timeout_row);

        let retries_label = Label::builder().label("Max retries:").halign(gtk4::Align::Start).build();
        let retries_adj = Adjustment::new(config.advanced.max_retries as f64, 0.0, 10.0, 1.0, 1.0, 0.0);
        let retries_spin = SpinButton::builder().adjustment(&retries_adj).build();
        let retries_row = Box::builder().orientation(Orientation::Horizontal).spacing(8).build();
        retries_row.append(&retries_label);
        retries_row.append(&retries_spin);
        container.append(&retries_row);

        container.append(&Self::make_section_label("<b>Debug</b>"));

        let debug_logging_switch = Switch::builder().active(config.advanced.debug_logging).build();
        container.append(&Self::make_row(&debug_logging_switch, "Enable debug logging"));

        (container, timeout_spin, retries_spin, debug_logging_switch)
    }
}
