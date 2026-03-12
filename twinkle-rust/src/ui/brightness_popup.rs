//! Brightness popup window for quick adjustments.

use crate::core::config::ConfigManager;
use crate::ddc::DDCManager;
use crate::ui::widgets::brightness_slider::BrightnessSlider;
use crate::ui::widgets::vcp_controls::VCPControlsContainer;
use gtk4::prelude::*;
use gtk4::{Box, Button, ComboBoxText, Label, Orientation, Popover};
use gtk4::glib;
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
    /// Current monitor ID
    current_monitor_id: Arc<Mutex<Option<String>>>,
    /// Auto-hide timer
    auto_hide_timer: Arc<Mutex<Option<glib::SourceId>>>,
    /// Auto-hide delay in milliseconds
    auto_hide_delay_ms: u32,
}

impl BrightnessPopup {
    /// Create a new brightness popup.
    pub async fn new(
        _parent: &impl IsA<gtk4::Widget>,
        ddc_manager: Arc<DDCManager>,
        config_manager: Arc<Mutex<ConfigManager>>,
    ) -> Self {
        let popover = Popover::builder()
            .width_request(350)
            .height_request(400)
            .build();

        // Get config for auto-hide delay
        let config = config_manager.lock().await;
        let auto_hide_delay_ms = config.config().ui.auto_hide_delay_ms;
        drop(config);

        // Create main container
        let container = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(12)
            .margin_top(12)
            .margin_bottom(12)
            .margin_start(12)
            .margin_end(12)
            .build();

        // Create monitor selector
        let monitor_selector_label = Label::builder()
            .label("Monitor:")
            .halign(gtk4::Align::Start)
            .build();

        let monitor_selector = ComboBoxText::new();
        monitor_selector.append(Some("all"), "All Monitors");

        let monitor_selector_box = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .build();
        monitor_selector_box.append(&monitor_selector_label);
        monitor_selector_box.append(&monitor_selector);

        // Create brightness slider
        let brightness_slider = BrightnessSlider::new();

        // Create preset buttons
        let preset_box = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .build();

        let preset_values = vec![20, 40, 60, 80, 100];
        let mut preset_buttons = Vec::new();

        for &value in &preset_values {
            let button = Button::builder()
                .label(&value.to_string())
                .build();
            preset_box.append(&button);
            preset_buttons.push(button);
        }

        // Create VCP controls
        let mut vcp_controls = VCPControlsContainer::new();
        vcp_controls.add_sections(&[0x12, 0x14, 0x60, 0x62]); // Contrast, Color Temp, Input Source, Volume

        // Add all widgets to container
        container.append(&monitor_selector_box);
        container.append(brightness_slider.widget());
        container.append(&preset_box);
        container.append(&gtk4::Separator::new(Orientation::Horizontal));
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
        // Connect brightness slider changes
        let ddc_manager = self.ddc_manager.clone();
        let current_monitor_id = self.current_monitor_id.clone();
        let auto_hide_timer = self.auto_hide_timer.clone();
        let auto_hide_delay_ms = self.auto_hide_delay_ms;

        // Clone the slider to get mutable access for set_on_change
        let mut brightness_slider = self.brightness_slider.clone();
        brightness_slider.set_on_change(move |value| {
            let ddc_manager = ddc_manager.clone();
            let current_monitor_id = current_monitor_id.clone();
            let auto_hide_timer = auto_hide_timer.clone();

            glib::spawn_future_local(async move {
                let monitor_id = {
                    let id = current_monitor_id.lock().await;
                    id.clone()
                };

                if let Some(id) = monitor_id {
                    if let Err(e) = ddc_manager.set_brightness(&id, value).await {
                        tracing::error!("Failed to set brightness: {}", e);
                    }
                }
            });

            // Reset auto-hide timer
            Self::reset_auto_hide_timer(auto_hide_timer.clone(), auto_hide_delay_ms);
        });

        // Connect preset buttons
        let ddc_manager = self.ddc_manager.clone();
        let current_monitor_id = self.current_monitor_id.clone();
        let auto_hide_timer = self.auto_hide_timer.clone();
        let auto_hide_delay_ms = self.auto_hide_delay_ms;

        for (i, button) in self.preset_buttons.iter().enumerate() {
            let value = [20, 40, 60, 80, 100][i];
            let ddc_manager = ddc_manager.clone();
            let current_monitor_id = current_monitor_id.clone();
            let auto_hide_timer = auto_hide_timer.clone();

            button.connect_clicked(move |_| {
                let ddc_manager = ddc_manager.clone();
                let current_monitor_id = current_monitor_id.clone();
                let auto_hide_timer = auto_hide_timer.clone();
                let value = value;

                glib::spawn_future_local(async move {
                    let monitor_id = {
                        let id = current_monitor_id.lock().await;
                        id.clone()
                    };

                    if let Some(id) = monitor_id {
                        if let Err(e) = ddc_manager.set_brightness(&id, value).await {
                            tracing::error!("Failed to set brightness: {}", e);
                        }
                    }
                });

                Self::reset_auto_hide_timer(auto_hide_timer.clone(), auto_hide_delay_ms);
            });
        }

        // Connect monitor selector changes
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

                        // Load current brightness for this monitor
                        if let Ok(brightness) = ddc_manager.get_brightness(&id).await {
                            brightness_slider.set_value(brightness).await;
                        }
                    }
                }
            });
        });

        // Connect popover closed signal to clear timer
        let auto_hide_timer = self.auto_hide_timer.clone();
        self.popover.connect_closed(move |_| {
            let timer_clone = auto_hide_timer.clone();
            glib::spawn_future_local(async move {
                let mut timer = timer_clone.lock().await;
                if let Some(source_id) = timer.take() {
                    source_id.remove();
                }
            });
        });
    }

    /// Reset the auto-hide timer.
    fn reset_auto_hide_timer(
        timer: Arc<Mutex<Option<glib::SourceId>>>,
        delay_ms: u32,
    ) {
        let timer_clone = timer.clone();
        glib::spawn_future_local(async move {
            let mut timer_guard = timer.lock().await;

            // Remove existing timer
            if let Some(source_id) = timer_guard.take() {
                source_id.remove();
            }

            // Don't set new timer if delay is 0 (disabled)
            if delay_ms == 0 {
                return;
            }

            // Set new timer
            let source_id = glib::timeout_add_local_once(
                std::time::Duration::from_millis(delay_ms as u64),
                move || {
                    // TODO: Hide the popup
                    let mut timer = timer_clone.blocking_lock();
                    *timer = None;
                },
            );

            *timer_guard = Some(source_id);
        });
    }

    /// Refresh the monitor list.
    pub async fn refresh_monitors(&self) {
        let monitors = self.ddc_manager.get_monitors().await;

        // Clear all items and re-add "All Monitors"
        self.monitor_selector.remove_all();
        self.monitor_selector.append(Some("all"), "All Monitors");

        // Add monitors
        for monitor in &monitors {
            self.monitor_selector
                .append(Some(&monitor.unique_id()), &monitor.display_name());
        }

        // Select first monitor if available
        if !monitors.is_empty() {
            let first_monitor = &monitors[0];
            self.monitor_selector.set_active_id(Some(&first_monitor.unique_id()));
            *self.current_monitor_id.lock().await = Some(first_monitor.unique_id());

            // Load current brightness
            if let Ok(brightness) = self.ddc_manager.get_brightness(&first_monitor.unique_id()).await {
                self.brightness_slider.set_value(brightness).await;
            }
        }
    }

    /// Show the popup.
    pub fn popup(&self) {
        self.popover.popup();

        // Start auto-hide timer
        let auto_hide_timer = self.auto_hide_timer.clone();
        let auto_hide_delay_ms = self.auto_hide_delay_ms;

        glib::spawn_future_local(async move {
            Self::reset_auto_hide_timer(auto_hide_timer, auto_hide_delay_ms);
        });
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
