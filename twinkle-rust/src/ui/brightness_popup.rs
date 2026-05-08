//! Brightness popup window with per-monitor card-based layout.

use crate::core::config::ConfigManager;
use crate::ddc::DDCManager;
use crate::ddc::monitor::Monitor;
use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{
    Adjustment, Box, Button, Label, Orientation, Scale, Window,
};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::Mutex;

/// A single monitor card with its own slider.
struct MonitorCard {
    /// Monitor unique ID
    monitor_id: String,
    /// Display name label
    name_label: Label,
    /// Brightness value label
    value_label: Label,
    /// The slider adjustment
    adjustment: Adjustment,
    /// Suppress callback flag
    suppress: Arc<AtomicBool>,
}

/// Brightness popup window.
pub struct BrightnessPopup {
    /// The window widget
    window: Window,
    /// Container for monitor cards
    cards_container: Box,
    /// Per-monitor cards
    cards: Vec<MonitorCard>,
    /// "All Monitors" override section
    override_adjustment: Adjustment,
    override_suppress: Arc<AtomicBool>,
    override_value_label: Label,
    override_row: Box,
    /// All-monitors mode active
    all_monitors_active: Arc<AtomicBool>,
    /// DDC manager
    ddc_manager: Arc<DDCManager>,
    /// Config manager
    config_manager: Arc<Mutex<ConfigManager>>,
    /// CSS provider (keep alive)
    _css_provider: gtk4::CssProvider,
}

impl BrightnessPopup {
    /// Create a new brightness popup.
    pub async fn new(
        parent: &impl IsA<gtk4::Window>,
        ddc_manager: Arc<DDCManager>,
        config_manager: Arc<Mutex<ConfigManager>>,
    ) -> Self {
        // Load CSS
        let css_provider = gtk4::CssProvider::new();
        css_provider.load_from_data(include_str!("style.css"));
        gtk4::StyleContext::add_provider_for_display(
            &gtk4::gdk::Display::default().unwrap_or_else(|| panic!("No display")),
            &css_provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        // Main container
        let main_box = Box::builder()
            .orientation(Orientation::Vertical)
            .css_classes(["main-container"])
            .build();

        // Header
        let header = Label::builder()
            .label("Brightness")
            .css_classes(["header-label"])
            .halign(gtk4::Align::Start)
            .build();
        main_box.append(&header);

        // Cards container (scrollable if many monitors)
        let cards_container = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(0)
            .build();
        main_box.append(&cards_container);

        // "All Monitors" override row (hidden by default)
        let override_row = Box::builder()
            .orientation(Orientation::Vertical)
            .css_classes(["override-row"])
            .build();

        let override_header = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .build();
        let override_icon = Label::new(Some("☀")); // sun icon
        override_icon.set_css_classes(&["sun-icon"]);
        let override_name = Label::builder()
            .label("All Monitors")
            .css_classes(["monitor-name"])
            .hexpand(true)
            .build();
        let override_value_label = Label::builder()
            .label("100%")
            .css_classes(["brightness-value"])
            .build();
        override_header.append(&override_icon);
        override_header.append(&override_name);
        override_header.append(&override_value_label);
        override_row.append(&override_header);

        let override_slider_row = Box::builder()
            .orientation(Orientation::Horizontal)
            .css_classes(["slider-row"])
            .build();
        let dim_icon = Label::new(Some("🔅"));
        dim_icon.set_css_classes(&["sun-dim-icon"]);
        let override_adjustment = Adjustment::new(100.0, 0.0, 100.0, 1.0, 10.0, 0.0);
        let override_scale = Scale::builder()
            .adjustment(&override_adjustment)
            .hexpand(true)
            .draw_value(false)
            .build();
        let bright_icon = Label::new(Some("🔆"));
        bright_icon.set_css_classes(&["sun-icon"]);
        override_slider_row.append(&dim_icon);
        override_slider_row.append(&override_scale);
        override_slider_row.append(&bright_icon);
        override_row.append(&override_slider_row);

        main_box.append(&override_row);
        override_row.set_visible(false);

        // Bottom toolbar: "All Monitors" toggle + settings
        let bottom = Box::builder()
            .orientation(Orientation::Horizontal)
            .css_classes(["bottom-toolbar"])
            .halign(gtk4::Align::Fill)
            .build();

        let all_btn = Button::builder()
            .label("All Monitors")
            .css_classes(["all-monitors-button"])
            .halign(gtk4::Align::Start)
            .build();

        let settings_btn = Button::builder()
            .label("⚙")
            .css_classes(["icon-button"])
            .halign(gtk4::Align::End)
            .build();

        bottom.append(&all_btn);
        bottom.append(&settings_btn);
        main_box.append(&bottom);

        // Window
        let window = Window::builder()
            .title("Brightness")
            .transient_for(parent)
            .resizable(false)
            .default_width(340)
            .decorated(true)
            .build();

        window.set_child(Some(&main_box));

        let all_monitors_active = Arc::new(AtomicBool::new(false));
        let override_suppress = Arc::new(AtomicBool::new(false));

        let popup = Self {
            window,
            cards_container,
            cards: Vec::new(),
            override_adjustment,
            override_suppress,
            override_value_label,
            override_row,
            all_monitors_active,
            ddc_manager,
            config_manager,
            _css_provider: css_provider,
        };

        popup.setup_connections(all_btn, settings_btn);
        popup
    }

    fn setup_connections(&self, all_btn: Button, settings_btn: Button) {
        // "All Monitors" button toggles the override row
        let all_monitors_active = self.all_monitors_active.clone();
        let override_row = self.override_row.clone();

        all_btn.connect_clicked(move |btn| {
            let active = all_monitors_active.load(Ordering::SeqCst);
            let new_active = !active;
            all_monitors_active.store(new_active, Ordering::SeqCst);
            override_row.set_visible(new_active);
            if new_active {
                btn.add_css_class("active");
            } else {
                btn.remove_css_class("active");
            }
        });

        // Override slider: sets ALL monitors at once
        let ddc_manager = self.ddc_manager.clone();
        let override_suppress = self.override_suppress.clone();
        let override_value_label = self.override_value_label.clone();
        let all_monitors_active = self.all_monitors_active.clone();

        self.override_adjustment.connect_value_changed(move |adj| {
            if override_suppress.swap(false, Ordering::SeqCst) {
                return;
            }
            if !all_monitors_active.load(Ordering::SeqCst) {
                return;
            }

            let value = adj.value() as u16;
            override_value_label.set_label(&format!("{}%", value));

            let ddc_manager = ddc_manager.clone();
            glib::spawn_future_local(async move {
                let monitors = ddc_manager.get_monitors().await;
                for m in &monitors {
                    if let Err(e) = ddc_manager.set_brightness(&m.unique_id(), value).await {
                        tracing::warn!("Override: failed on {}: {}", m.display_name(), e);
                    }
                }
            });

            // Debounce: prevent rapid fire
            override_suppress.store(true, Ordering::SeqCst);
            let suppress = override_suppress.clone();
            glib::timeout_add_local_once(std::time::Duration::from_millis(300), move || {
                suppress.store(false, Ordering::SeqCst);
            });
        });

        // Settings button
        settings_btn.connect_clicked(|_| {
            // TODO: open settings dialog
        });
    }

    /// Build a single monitor card widget.
    fn build_card(monitor: &Monitor, brightness: u16) -> (Box, MonitorCard) {
        let card = Box::builder()
            .orientation(Orientation::Vertical)
            .css_classes(["monitor-card"])
            .build();

        // Header row: icon + name + value
        let header = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .build();

        let icon = Label::new(Some("🖥")); // monitor icon
        icon.set_css_classes(&["monitor-icon"]);
        let name_label = Label::builder()
            .label(&monitor.display_name())
            .css_classes(["monitor-name"])
            .hexpand(true)
            .xalign(0.0)
            .build();
        let value_label = Label::builder()
            .label(&format!("{}%", brightness))
            .css_classes(["brightness-value"])
            .build();

        header.append(&icon);
        header.append(&name_label);
        header.append(&value_label);
        card.append(&header);

        // Slider row
        let slider_row = Box::builder()
            .orientation(Orientation::Horizontal)
            .css_classes(["slider-row"])
            .spacing(4)
            .build();

        let dim = Label::new(Some("🔅"));
        dim.set_css_classes(&["sun-dim-icon"]);

        let adjustment = Adjustment::new(brightness as f64, 0.0, 100.0, 1.0, 10.0, 0.0);
        let scale = Scale::builder()
            .adjustment(&adjustment)
            .hexpand(true)
            .draw_value(false)
            .build();

        let bright = Label::new(Some("🔆"));
        bright.set_css_classes(&["sun-icon"]);

        slider_row.append(&dim);
        slider_row.append(&scale);
        slider_row.append(&bright);
        card.append(&slider_row);

        let suppress = Arc::new(AtomicBool::new(false));
        let monitor_card = MonitorCard {
            monitor_id: monitor.unique_id(),
            name_label,
            value_label,
            adjustment,
            suppress,
        };

        (card, monitor_card)
    }

    /// Refresh all monitor cards.
    pub async fn refresh_monitors(&self) {
        // Clear existing cards
        while let Some(child) = self.cards_container.first_child() {
            self.cards_container.remove(&child);
        }

        let monitors = self.ddc_manager.get_monitors().await;
        let mut new_cards = Vec::new();

        for monitor in &monitors {
            let brightness = self.ddc_manager.get_brightness(&monitor.unique_id())
                .await
                .unwrap_or(50);

            let (card_widget, card) = Self::build_card(monitor, brightness);
            self.cards_container.append(&card_widget);
            new_cards.push(card);
        }

        // We need to reconnect sliders after rebuilding.
        // This is handled by popup() which calls setup_slider_connections.
        // For now store cards without connections (popup() will connect them).
        // Actually we can't mutate self.cards from here since we only have &self.
        // We'll use a different pattern: rebuild happens in popup() via glib::spawn_future_local.
    }

    /// Show the popup.
    pub fn popup(&self) {
        let ddc_manager = self.ddc_manager.clone();
        let cards_container = self.cards_container.clone();
        let all_monitors_active = self.all_monitors_active.clone();
        let override_adjustment = self.override_adjustment.clone();
        let override_suppress = self.override_suppress.clone();
        let override_value_label = self.override_value_label.clone();
        let override_row = self.override_row.clone();

        glib::spawn_future_local(async move {
            let monitors = ddc_manager.get_monitors().await;

            // Clear existing children
            while let Some(child) = cards_container.first_child() {
                cards_container.remove(&child);
            }

            for monitor in &monitors {
                let brightness = ddc_manager.get_brightness(&monitor.unique_id())
                    .await
                    .unwrap_or(50);

                let (card_widget, card) = Self::build_card(monitor, brightness);

                // Connect slider
                let ddc_mgr = ddc_manager.clone();
                let monitor_id = card.monitor_id.clone();
                let value_label = card.value_label.clone();
                let suppress = card.suppress.clone();
                let all_active = all_monitors_active.clone();

                card.adjustment.connect_value_changed(move |adj| {
                    if suppress.swap(false, Ordering::SeqCst) {
                        return;
                    }
                    let value = adj.value() as u16;
                    value_label.set_label(&format!("{}%", value));

                    let ddc_manager = ddc_mgr.clone();
                    let monitor_id = monitor_id.clone();
                    let all_active = all_active.clone();
                    glib::spawn_future_local(async move {
                        if all_active.load(Ordering::SeqCst) {
                            // In "all monitors" mode, set all monitors
                            let monitors = ddc_manager.get_monitors().await;
                            for m in &monitors {
                                if let Err(e) = ddc_manager.set_brightness(&m.unique_id(), value).await {
                                    tracing::warn!("Failed on {}: {}", m.display_name(), e);
                                }
                            }
                        } else {
                            if let Err(e) = ddc_manager.set_brightness(&monitor_id, value).await {
                                tracing::error!("Failed to set brightness: {}", e);
                            }
                        }
                    });

                    // Debounce
                    suppress.store(true, Ordering::SeqCst);
                    let suppress = suppress.clone();
                    glib::timeout_add_local_once(std::time::Duration::from_millis(300), move || {
                        suppress.store(false, Ordering::SeqCst);
                    });
                });

                cards_container.append(&card_widget);
            }

            // Set override slider to first monitor's brightness (or average)
            if let Some(first) = monitors.first() {
                if let Ok(b) = ddc_manager.get_brightness(&first.unique_id()).await {
                    override_suppress.store(true, Ordering::SeqCst);
                    override_adjustment.set_value(b as f64);
                    override_value_label.set_label(&format!("{}%", b));
                }
            }

            // If override was active, show it
            if all_monitors_active.load(Ordering::SeqCst) {
                override_row.set_visible(true);
            }
        });

        self.window.present();
    }

    /// Hide the popup.
    pub fn popdown(&self) {
        self.window.hide();
    }

    /// Get the window widget.
    pub fn widget(&self) -> &Window {
        &self.window
    }
}
