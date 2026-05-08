//! Brightness popup window for quick adjustments.

use crate::core::config::ConfigManager;
use crate::ddc::DDCManager;
use crate::ui::widgets::brightness_slider::BrightnessSlider;
use crate::ui::widgets::vcp_controls::VCPControlsContainer;
use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{Box, Button, ComboBoxText, Label, Orientation, Popover};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Brightness popup window.
pub struct BrightnessPopup {
    /// The popover widget
    popover: Popover,
    /// Brightness slider
    brightness_slider: BrightnessSlider,
    /// Monitor selector combo box
    monitor_selector: ComboBoxText,
    /// Preset buttons
    preset_buttons: Vec<Button>,
    /// VCP controls container
    vcp_controls: VCPControlsContainer,
    /// DDC manager
    ddc_manager: Arc<DDCManager>,
    /// Config manager
    config_manager: Arc<Mutex<ConfigManager>>,
    /// Current monitor ID (None = All Monitors)
    current_monitor_id: Arc<Mutex<Option<String>>>,
    /// Auto-hide timer
    auto_hide_timer: Arc<Mutex<Option<glib::SourceId>>>,
    /// Auto-hide delay in milliseconds
    auto_hide_delay_ms: u32,
}

impl BrightnessPopup {
    /// Create a new brightness popup.
    pub async fn new(
        parent: &impl IsA<gtk4::Widget>,
        ddc_manager: Arc<DDCManager>,
        config_manager: Arc<Mutex<ConfigManager>>,
    ) -> Self {
        let popover = Popover::builder()
            .width_request(350)
            .height_request(400)
            .position(gtk4::PositionType::Top)
            .build();

        popover.set_parent(parent);

        let config = config_manager.lock().await;
        let auto_hide_delay_ms = config.config().ui.auto_hide_delay_ms;
        let preset_values = config.config().ui.preset_values.clone();
        let enable_presets = config.config().ui.enable_presets;
        let show_monitor_selector = config.config().ui.show_monitor_selector;
        drop(config);

        // Main container
        let container = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(10)
            .margin_top(10)
            .margin_bottom(10)
            .margin_start(12)
            .margin_end(12)
            .build();

        // Monitor selector
        let monitor_selector = ComboBoxText::new();
        if show_monitor_selector {
            let selector_row = Box::builder()
                .orientation(Orientation::Horizontal)
                .spacing(8)
                .build();
            let label = Label::builder().label("Monitor:").halign(gtk4::Align::Start).build();
            selector_row.append(&label);
            selector_row.append(&monitor_selector);
            container.append(&selector_row);
        }

        // Brightness slider
        let brightness_slider = BrightnessSlider::new();
        container.append(brightness_slider.widget());

        // Preset buttons
        let mut preset_buttons = Vec::new();
        if enable_presets {
            let preset_box = Box::builder()
                .orientation(Orientation::Horizontal)
                .spacing(6)
                .homogeneous(true)
                .build();

            for &value in &preset_values {
                let button = Button::builder()
                    .label(&value.to_string())
                    .css_classes(["preset-button"])
                    .build();
                preset_box.append(&button);
                preset_buttons.push(button);
            }
            container.append(&preset_box);
        }

        // Separator
        container.append(&gtk4::Separator::new(Orientation::Horizontal));

        // VCP controls
        let mut vcp_controls = VCPControlsContainer::new();
        vcp_controls.add_sections(&[0x12, 0x14, 0x60, 0x62]);
        container.append(vcp_controls.widget());

        popover.set_child(Some(&container));

        let popup = Self {
            popover,
            brightness_slider,
            monitor_selector,
            preset_buttons,
            vcp_controls,
            ddc_manager,
            config_manager,
            current_monitor_id: Arc::new(Mutex::new(None)),
            auto_hide_timer: Arc::new(Mutex::new(None)),
            auto_hide_delay_ms,
        };

        popup.setup_connections().await;
        popup
    }

    /// Setup signal connections.
    async fn setup_connections(&self) {
        let ddc_manager = self.ddc_manager.clone();
        let current_monitor_id = self.current_monitor_id.clone();

        // Slider changes
        let ddc_mgr = ddc_manager.clone();
        let mon_id = current_monitor_id.clone();
        let mut brightness_slider = self.brightness_slider.clone();
        brightness_slider.set_on_change(move |value| {
            let ddc_manager = ddc_mgr.clone();
            let current_monitor_id = mon_id.clone();

            glib::spawn_future_local(async move {
                let monitor_id = current_monitor_id.lock().await.clone();
                match monitor_id {
                    Some(id) => {
                        if let Err(e) = ddc_manager.set_brightness(&id, value).await {
                            tracing::error!("Failed to set brightness: {}", e);
                        }
                    }
                    None => {
                        // All Monitors mode: set brightness on every monitor
                        let monitors = ddc_manager.get_monitors().await;
                        for m in &monitors {
                            if let Err(e) = ddc_manager.set_brightness(&m.unique_id(), value).await {
                                tracing::warn!("Failed to set brightness on {}: {}", m.display_name(), e);
                            }
                        }
                    }
                }
            });
        });

        // Preset buttons
        for (i, button) in self.preset_buttons.iter().enumerate() {
            let config = self.config_manager.lock().await;
            let preset_values = config.config().ui.preset_values.clone();
            drop(config);
            let value = preset_values.get(i).copied().unwrap_or((i as u16 + 1) * 20);
            let ddc_manager = ddc_manager.clone();
            let current_monitor_id = current_monitor_id.clone();

            button.connect_clicked(move |_| {
                let ddc_manager = ddc_manager.clone();
                let current_monitor_id = current_monitor_id.clone();
                let value = value;

                glib::spawn_future_local(async move {
                    let monitor_id = current_monitor_id.lock().await.clone();
                    match monitor_id {
                        Some(id) => {
                            if let Err(e) = ddc_manager.set_brightness(&id, value).await {
                                tracing::error!("Failed to set brightness: {}", e);
                            }
                        }
                        None => {
                            let monitors = ddc_manager.get_monitors().await;
                            for m in &monitors {
                                if let Err(e) = ddc_manager.set_brightness(&m.unique_id(), value).await {
                                    tracing::warn!("Failed to set brightness on {}: {}", m.display_name(), e);
                                }
                            }
                        }
                    }
                });
            });
        }

        // Monitor selector
        let ddc_manager = self.ddc_manager.clone();
        let current_monitor_id = self.current_monitor_id.clone();
        let brightness_slider = self.brightness_slider.clone();

        self.monitor_selector.connect_changed(move |combo| {
            let ddc_manager = ddc_manager.clone();
            let current_monitor_id = current_monitor_id.clone();
            let brightness_slider = brightness_slider.clone();
            let selected_id = combo.active_id();

            glib::spawn_future_local(async move {
                if let Some(id) = selected_id {
                    if id == "all" {
                        *current_monitor_id.lock().await = None;
                    } else {
                        *current_monitor_id.lock().await = Some(id.to_string());

                        if let Ok(brightness) = ddc_manager.get_brightness(&id).await {
                            brightness_slider.set_value(brightness).await;
                        }
                    }
                }
            });
        });

        // Popover closed signal
        let auto_hide_timer = self.auto_hide_timer.clone();
        self.popover.connect_closed(move |_| {
            let timer = auto_hide_timer.clone();
            glib::spawn_future_local(async move {
                let mut t = timer.lock().await;
                if let Some(source_id) = t.take() {
                    source_id.remove();
                }
            });
        });
    }

    /// Refresh the monitor list.
    pub async fn refresh_monitors(&self) {
        let monitors = self.ddc_manager.get_monitors().await;

        self.monitor_selector.remove_all();
        self.monitor_selector.append(Some("all"), "All Monitors");

        for monitor in &monitors {
            self.monitor_selector
                .append(Some(&monitor.unique_id()), &monitor.display_name());
        }

        if !monitors.is_empty() {
            let first = &monitors[0];
            self.monitor_selector.set_active_id(Some(&first.unique_id()));
            *self.current_monitor_id.lock().await = Some(first.unique_id());

            if let Ok(brightness) = self.ddc_manager.get_brightness(&first.unique_id()).await {
                self.brightness_slider.set_value(brightness).await;
            }
        } else {
            self.monitor_selector.set_active_id(Some("all"));
            *self.current_monitor_id.lock().await = None;
        }
    }

    /// Show the popup.
    pub fn popup(&self) {
        // Refresh monitors each time the popup opens
        let ddc_manager = self.ddc_manager.clone();
        let current_monitor_id = self.current_monitor_id.clone();
        let monitor_selector = self.monitor_selector.clone();
        let brightness_slider = self.brightness_slider.clone();

        glib::spawn_future_local(async move {
            let monitors = ddc_manager.get_monitors().await;

            monitor_selector.remove_all();
            monitor_selector.append(Some("all"), "All Monitors");

            for monitor in &monitors {
                monitor_selector.append(Some(&monitor.unique_id()), &monitor.display_name());
            }

            if !monitors.is_empty() {
                let first = &monitors[0];
                monitor_selector.set_active_id(Some(&first.unique_id()));
                *current_monitor_id.lock().await = Some(first.unique_id());

                if let Ok(brightness) = ddc_manager.get_brightness(&first.unique_id()).await {
                    brightness_slider.set_value(brightness).await;
                }
            }
        });

        self.popover.popup();
    }

    /// Hide the popup.
    pub fn popdown(&self) {
        self.popover.popdown();
    }

    /// Get the popover widget.
    pub fn widget(&self) -> &Popover {
        &self.popover
    }
}
