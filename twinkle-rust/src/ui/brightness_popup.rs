//! Brightness popup window with per-monitor card-based layout.
//!
//! Sliders show the live percentage while dragging and snap to the preset
//! values configured in `ui.preset_values` (rendered as tick marks on the
//! scale). Manual changes are persisted per monitor (when
//! `behavior.remember_brightness` is on) and suspend the day/night schedule
//! engine for the current phase.

use crate::core::config::ConfigManager;
use crate::core::schedule::ScheduleEvent;
use crate::ddc::monitor::Monitor;
use crate::ddc::software_filter::SoftwareFilter;
use crate::ddc::DDCManager;
use crate::utils::{normalized_presets, snap_to_preset};
use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{Adjustment, Box, Button, GestureClick, Label, Orientation, Scale, Window};
use std::sync::atomic::{AtomicBool, AtomicU16, Ordering};
use std::sync::Arc;
use std::sync::Mutex as StdMutex;
use tokio::sync::Mutex;

/// Per-monitor state needed to sync card sliders with the master slider.
#[allow(dead_code)]
struct CardEntry {
    monitor_id: String,
    adjustment: Adjustment,
    value_label: Label,
    suppress: Arc<AtomicBool>,
}

/// Render tick marks for each configured preset value on a scale.
fn apply_slider_marks(scale: &Scale, presets: &[u16]) {
    scale.clear_marks();
    for value in presets {
        scale.add_mark(
            f64::from(*value),
            gtk4::PositionType::Bottom,
            Some(value.to_string().as_str()),
        );
    }
}

/// Snap the adjustment to the nearest configured preset on pointer release.
///
/// GTK scales do not snap natively; the released gesture rounds the final
/// value to the closest preset. The resulting `set_value` re-triggers the
/// debounced send path, so the snapped value is what reaches the monitors.
fn attach_snap_on_release(
    scale: &Scale,
    adjustment: &Adjustment,
    snap_presets: Arc<StdMutex<Vec<u16>>>,
) {
    let gesture = GestureClick::new();
    let adjustment = adjustment.clone();
    gesture.connect_released(move |_gesture, _n_press, _x, _y| {
        let presets = match snap_presets.lock() {
            Ok(p) => p.clone(),
            Err(_) => return,
        };
        if presets.is_empty() {
            return;
        }
        let raw = adjustment.value() as u16;
        let snapped = snap_to_preset(raw, &presets);
        if snapped != raw {
            tracing::debug!("Slider snap: {}% → {}%", raw, snapped);
            adjustment.set_value(f64::from(snapped));
        }
    });
    scale.add_controller(gesture);
}

/// Connect a scale with simple debounce: send value only after 300ms of inactivity.
/// Note: send_fn runs on the GLib main thread via timeout_add_local, so Send+Sync
/// is NOT required. This allows closures capturing GTK widgets.
fn connect_slider_debounced(
    adjustment: &Adjustment,
    value_label: Label,
    suppress: Arc<AtomicBool>,
    send_fn: impl Fn(u16) + Clone + 'static,
) {
    let pending_value = Arc::new(AtomicU16::new(0));
    let debounce_id: Arc<StdMutex<Option<u32>>> = Arc::new(StdMutex::new(None));
    let send = send_fn.clone();
    let deb = debounce_id.clone();
    let pv = pending_value.clone();

    adjustment.connect_value_changed(move |adj| {
        // Skip if programmatically set
        if suppress.swap(false, Ordering::SeqCst) {
            return;
        }
        let value = adj.value() as u16;
        value_label.set_label(&format!("{}%", value));
        pv.store(value, Ordering::SeqCst);

        // Cancel old debounce timer
        if let Some(id) = deb.lock().ok().and_then(|mut id| id.take()) {
            unsafe {
                gtk4::glib::ffi::g_source_remove(id);
            }
        }

        // Set new debounce timer (300ms)
        let send = send.clone();
        let deb2 = deb.clone();
        let pv2 = pv.clone();
        let new_id = glib::timeout_add_local(std::time::Duration::from_millis(300), move || {
            let val = pv2.load(Ordering::SeqCst);
            send(val);
            if let Ok(mut id) = deb2.lock() {
                *id = None;
            }
            glib::ControlFlow::Break
        });
        if let Ok(mut id) = deb.lock() {
            *id = Some(new_id.as_raw());
        }
    });
}

/// Brightness popup window.
#[allow(dead_code)]
pub struct BrightnessPopup {
    window: Window,
    cards_container: Box,
    /// Per-monitor slider state for syncing with the master slider.
    card_entries: Arc<StdMutex<Vec<CardEntry>>>,
    override_adjustment: Adjustment,
    override_scale: Scale,
    override_suppress: Arc<AtomicBool>,
    override_value_label: Label,
    override_row: Box,
    ddc_manager: Arc<DDCManager>,
    config_manager: Arc<Mutex<ConfigManager>>,
    /// Software filter for sub-minimum dimming via gamma
    software_filter: Arc<StdMutex<SoftwareFilter>>,
    /// Software filter slider adjustment
    filter_adjustment: Adjustment,
    filter_value_label: Label,
    /// Current snap values (sorted, deduped config presets), shared with
    /// all slider snap handlers and refreshed on every popup rebuild.
    snap_presets: Arc<StdMutex<Vec<u16>>>,
    /// Notifies the schedule engine about manual brightness changes.
    manual_tx: Option<tokio::sync::mpsc::UnboundedSender<ScheduleEvent>>,
    _css_provider: gtk4::CssProvider,
}

impl BrightnessPopup {
    pub async fn new(
        parent: &impl IsA<gtk4::Window>,
        ddc_manager: Arc<DDCManager>,
        config_manager: Arc<Mutex<ConfigManager>>,
        manual_tx: Option<tokio::sync::mpsc::UnboundedSender<ScheduleEvent>>,
    ) -> Self {
        // Load CSS
        let css_provider = gtk4::CssProvider::new();
        css_provider.load_from_data(include_str!("style.css"));
        gtk4::style_context_add_provider_for_display(
            &gtk4::gdk::Display::default().unwrap_or_else(|| panic!("No display")),
            &css_provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

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

        // Cards container
        let cards_container = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(0)
            .build();
        main_box.append(&cards_container);

        // "All Monitors" slider — always visible
        let override_row = Box::builder()
            .orientation(Orientation::Vertical)
            .css_classes(["all-monitors-row"])
            .build();

        let override_header = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .build();
        let override_icon = Label::new(Some("\u{2600}"));
        override_icon.set_css_classes(&["sun-icon"]);
        let override_name = Label::builder()
            .label("All Monitors")
            .css_classes(["monitor-name"])
            .hexpand(true)
            .xalign(0.0)
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
        let dim_icon = Label::new(Some("\u{1F505}"));
        dim_icon.set_css_classes(&["sun-dim-icon"]);
        let override_adjustment = Adjustment::new(100.0, 0.0, 100.0, 1.0, 10.0, 0.0);
        let override_scale = Scale::builder()
            .adjustment(&override_adjustment)
            .hexpand(true)
            .draw_value(false)
            .build();
        let bright_icon = Label::new(Some("\u{1F506}"));
        bright_icon.set_css_classes(&["sun-icon"]);
        override_slider_row.append(&dim_icon);
        override_slider_row.append(&override_scale);
        override_slider_row.append(&bright_icon);
        override_row.append(&override_slider_row);

        main_box.append(&override_row);

        // ── Software Filter (Night Dimmer) ──────────────────────
        // Gamma-based dimming below hardware minimum
        let filter_section = Box::builder()
            .orientation(Orientation::Vertical)
            .css_classes(["filter-section"])
            .build();

        let filter_header = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .build();
        let filter_icon = Label::new(Some("\u{1F319}")); // 🌙
        filter_icon.set_css_classes(&["moon-icon"]);
        let filter_title = Label::builder()
            .label("Night Filter")
            .css_classes(["filter-title"])
            .hexpand(true)
            .xalign(0.0)
            .build();
        let filter_value_label = Label::builder()
            .label("Off")
            .css_classes(["filter-value"])
            .build();
        filter_header.append(&filter_icon);
        filter_header.append(&filter_title);
        filter_header.append(&filter_value_label);
        filter_section.append(&filter_header);

        let filter_slider_row = Box::builder()
            .orientation(Orientation::Horizontal)
            .css_classes(["slider-row"])
            .build();
        let filter_dim = Label::new(Some("\u{2600}")); // ☀
        filter_dim.set_css_classes(&["sun-dim-icon"]);
        let filter_adjustment = Adjustment::new(0.0, 0.0, 100.0, 1.0, 10.0, 0.0);
        let filter_scale = Scale::builder()
            .adjustment(&filter_adjustment)
            .hexpand(true)
            .draw_value(false)
            .build();
        let filter_bright = Label::new(Some("\u{1F319}")); // 🌙
        filter_bright.set_css_classes(&["moon-icon"]);
        filter_slider_row.append(&filter_dim);
        filter_slider_row.append(&filter_scale);
        filter_slider_row.append(&filter_bright);
        filter_section.append(&filter_slider_row);

        main_box.append(&filter_section);

        // Separator
        let sep = Box::builder().css_classes(["separator"]).build();
        main_box.append(&sep);

        // Bottom toolbar — settings only
        let bottom = Box::builder()
            .orientation(Orientation::Horizontal)
            .css_classes(["bottom-toolbar"])
            .halign(gtk4::Align::End)
            .build();

        let settings_btn = Button::builder()
            .label("\u{2699}")
            .css_classes(["icon-button"])
            .build();

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

        let override_suppress = Arc::new(AtomicBool::new(false));

        // Initialize software filter
        let mut software_filter = SoftwareFilter::new();
        software_filter.detect_outputs();
        let software_filter = Arc::new(StdMutex::new(software_filter));

        // Restore saved filter level from config
        let saved_filter_level = {
            let cfg = config_manager.blocking_lock();
            cfg.config().ui.software_filter_level
        };
        if saved_filter_level > 0 {
            if let Ok(mut sf) = software_filter.lock() {
                if let Err(e) = sf.set_filter_level(saved_filter_level as f64 / 100.0) {
                    tracing::warn!("Failed to restore software filter: {}", e);
                }
            }
            filter_adjustment.set_value(saved_filter_level as f64);
            filter_value_label.set_label(&format!("{}%", saved_filter_level));
        }

        // Initial snap values from config
        let initial_presets = {
            let cfg = config_manager.blocking_lock();
            normalized_presets(
                &cfg.config().ui.preset_values,
                cfg.config().behavior.min_brightness,
            )
        };
        apply_slider_marks(&override_scale, &initial_presets);
        let snap_presets = Arc::new(StdMutex::new(initial_presets));
        attach_snap_on_release(&override_scale, &override_adjustment, snap_presets.clone());

        let popup = Self {
            window,
            cards_container,
            card_entries: Arc::new(StdMutex::new(Vec::new())),
            override_adjustment,
            override_scale,
            override_suppress,
            override_value_label,
            override_row,
            ddc_manager,
            config_manager,
            software_filter,
            filter_adjustment,
            filter_value_label,
            snap_presets,
            manual_tx,
            _css_provider: css_provider,
        };

        popup.setup_connections(settings_btn);
        popup
    }

    fn setup_connections(&self, settings_btn: Button) {
        // Override slider: sends to all monitors after 300ms debounce
        let ddc_manager = self.ddc_manager.clone();
        let override_suppress = self.override_suppress.clone();
        let override_value_label = self.override_value_label.clone();
        let config_manager = self.config_manager.clone();
        let card_entries = self.card_entries.clone();
        let manual_tx = self.manual_tx.clone();

        connect_slider_debounced(
            &self.override_adjustment,
            override_value_label,
            override_suppress,
            move |value| {
                let ddc_manager = ddc_manager.clone();
                let config_manager = config_manager.clone();
                let card_entries = card_entries.clone();
                let manual_tx = manual_tx.clone();
                glib::spawn_future_local(async move {
                    // Get min_brightness from config
                    let min = {
                        let cfg = config_manager.blocking_lock();
                        cfg.config().behavior.min_brightness
                    };
                    let clamped = value.max(min);
                    let monitors = ddc_manager.get_monitors().await;
                    for m in &monitors {
                        if let Err(e) = ddc_manager.set_brightness(&m.unique_id(), clamped).await {
                            tracing::warn!("Override: failed on {}: {}", m.display_name(), e);
                        }
                    }

                    // Notify the schedule engine: manual change wins for this phase
                    if let Some(tx) = &manual_tx {
                        let _ = tx.send(ScheduleEvent::ManualChange);
                    }

                    // Sync individual card sliders to the override value
                    if let Ok(cards) = card_entries.lock() {
                        for entry in cards.iter() {
                            entry.suppress.store(true, Ordering::SeqCst);
                            entry.adjustment.set_value(clamped as f64);
                            entry.value_label.set_label(&format!("{}%", clamped));
                            // Clear suppress after a short delay so individual changes work again
                            let supp = entry.suppress.clone();
                            glib::timeout_add_local(
                                std::time::Duration::from_millis(350),
                                move || {
                                    supp.store(false, Ordering::SeqCst);
                                    glib::ControlFlow::Break
                                },
                            );
                        }
                    }
                });
            },
        );

        // Software filter slider: adjusts gamma via xrandr
        let software_filter = self.software_filter.clone();
        let filter_value_label = self.filter_value_label.clone();
        let config_manager = self.config_manager.clone();

        self.filter_adjustment.connect_value_changed(move |adj| {
            let percent = adj.value() as u8;
            if percent == 0 {
                filter_value_label.set_label("Off");
            } else {
                filter_value_label.set_label(&format!("{}%", percent));
            }

            let level = percent as f64 / 100.0;
            if let Ok(mut sf) = software_filter.lock() {
                if let Err(e) = sf.set_filter_level(level) {
                    tracing::warn!("Software filter error: {}", e);
                }
            }

            // Save filter level to config
            let config_manager = config_manager.clone();
            glib::spawn_future_local(async move {
                let mut cfg = config_manager.lock().await;
                cfg.config_mut().ui.software_filter_level = percent;
                if let Err(e) = cfg.save() {
                    tracing::warn!("Failed to save filter level: {}", e);
                }
            });
        });

        // Settings button — opens the settings dialog.
        let config = self.config_manager.clone();
        let win = self.window.clone();
        settings_btn.connect_clicked(move |_| {
            let config = config.clone();
            let win = win.clone();
            glib::spawn_future_local(async move {
                let dialog =
                    crate::ui::widgets::settings_dialog::SettingsDialog::new(&win, config).await;
                dialog.present();
            });
        });
    }

    /// Build a single monitor card widget with preset tick marks and
    /// snap-on-release behavior.
    fn build_card(
        monitor: &Monitor,
        brightness: u16,
        presets: &[u16],
        snap_presets: &Arc<StdMutex<Vec<u16>>>,
    ) -> (Box, CardEntry, Adjustment) {
        let card = Box::builder()
            .orientation(Orientation::Vertical)
            .css_classes(["monitor-card"])
            .build();

        // Header row: icon + name + value
        let header = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .build();

        let icon = Label::new(Some(
            if monitor.monitor_type == crate::ddc::monitor::MonitorType::Internal {
                "\u{1F30D}"
            } else {
                "\u{1F5A5}"
            },
        ));
        icon.set_css_classes(&["monitor-icon"]);
        let name_label = Label::builder()
            .label(monitor.display_name())
            .css_classes(["monitor-name"])
            .hexpand(true)
            .xalign(0.0)
            .build();
        let value_label = Label::builder()
            .label(format!("{}%", brightness))
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

        let dim = Label::new(Some("\u{1F505}"));
        dim.set_css_classes(&["sun-dim-icon"]);

        let adjustment = Adjustment::new(brightness as f64, 0.0, 100.0, 1.0, 10.0, 0.0);
        let scale = Scale::builder()
            .adjustment(&adjustment)
            .hexpand(true)
            .draw_value(false)
            .build();
        apply_slider_marks(&scale, presets);
        attach_snap_on_release(&scale, &adjustment, snap_presets.clone());

        let bright = Label::new(Some("\u{1F506}"));
        bright.set_css_classes(&["sun-icon"]);

        slider_row.append(&dim);
        slider_row.append(&scale);
        slider_row.append(&bright);
        card.append(&slider_row);

        let suppress = Arc::new(AtomicBool::new(false));
        let adjustment_clone = adjustment.clone();
        let card_entry = CardEntry {
            monitor_id: monitor.unique_id(),
            adjustment,
            value_label,
            suppress,
        };

        (card, card_entry, adjustment_clone)
    }

    /// Show the popup. Rebuilds the monitor cards each time so freshly
    /// connected displays and edited config presets are picked up.
    pub fn popup(&self) {
        let ddc_manager = self.ddc_manager.clone();
        let cards_container = self.cards_container.clone();
        let override_adjustment = self.override_adjustment.clone();
        let override_suppress = self.override_suppress.clone();
        let override_value_label = self.override_value_label.clone();
        let config_manager = self.config_manager.clone();
        let card_entries = self.card_entries.clone();
        let snap_presets = self.snap_presets.clone();
        let override_scale = self.override_scale.clone();
        let manual_tx = self.manual_tx.clone();

        glib::spawn_future_local(async move {
            let monitors = ddc_manager.get_monitors().await;
            // Clear existing children
            while let Some(child) = cards_container.first_child() {
                cards_container.remove(&child);
            }

            // Clear card entries for fresh rebuild
            card_entries.lock().unwrap().clear();

            // Config-driven slider values: sorted, deduped, above min
            let (presets, min_brightness, remember) = {
                let cfg = config_manager.blocking_lock();
                let behavior = &cfg.config().behavior;
                (
                    normalized_presets(&cfg.config().ui.preset_values, behavior.min_brightness),
                    behavior.min_brightness,
                    behavior.remember_brightness,
                )
            };
            *snap_presets.lock().unwrap() = presets.clone();
            apply_slider_marks(&override_scale, &presets);

            for monitor in &monitors {
                let brightness = ddc_manager
                    .get_brightness(&monitor.unique_id())
                    .await
                    .unwrap_or(50)
                    .max(min_brightness);

                let (card_widget, entry, adjustment) =
                    Self::build_card(monitor, brightness, &presets, &snap_presets);

                // Connect slider: debounce 300ms, single monitor
                let ddc_mgr = ddc_manager.clone();
                let monitor_id = monitor.unique_id();
                let value_label = entry.value_label.clone();
                let suppress = entry.suppress.clone();

                // Store card entry for sync with override slider
                card_entries.lock().unwrap().push(CardEntry {
                    monitor_id: monitor_id.clone(),
                    adjustment: entry.adjustment.clone(),
                    value_label: entry.value_label.clone(),
                    suppress: entry.suppress.clone(),
                });

                let min = min_brightness;
                let config_mgr = config_manager.clone();
                let manual = manual_tx.clone();
                connect_slider_debounced(&adjustment, value_label, suppress, move |value| {
                    let ddc_manager = ddc_mgr.clone();
                    let monitor_id = monitor_id.clone();
                    let config_mgr = config_mgr.clone();
                    let manual = manual.clone();
                    glib::spawn_future_local(async move {
                        let clamped = value.max(min);
                        if let Err(e) = ddc_manager.set_brightness(&monitor_id, clamped).await {
                            tracing::error!("Failed to set brightness: {}", e);
                            return;
                        }

                        // Manual change: suspend the day/night schedule for
                        // this phase so the user's choice sticks.
                        if let Some(tx) = &manual {
                            let _ = tx.send(ScheduleEvent::ManualChange);
                        }

                        // Remember last brightness per monitor
                        if remember {
                            let mut cfg = config_mgr.lock().await;
                            {
                                let mcfg = cfg.get_or_create_monitor_config(&monitor_id, "");
                                mcfg.last_brightness = clamped;
                            }
                            if let Err(e) = cfg.save() {
                                tracing::warn!("Failed to persist brightness: {}", e);
                            }
                        }
                    });
                });

                cards_container.append(&card_widget);
            }

            // Set override slider to average brightness of all monitors
            if !monitors.is_empty() {
                let mut sum: u16 = 0;
                let mut count: u16 = 0;
                for m in &monitors {
                    if let Ok(b) = ddc_manager.get_brightness(&m.unique_id()).await {
                        sum += b.max(min_brightness);
                        count += 1;
                    }
                }
                if count > 0 {
                    let avg = sum / count;
                    override_suppress.store(true, Ordering::SeqCst);
                    override_adjustment.set_value(avg as f64);
                    override_value_label.set_label(&format!("{}%", avg));
                }
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

impl Drop for BrightnessPopup {
    fn drop(&mut self) {
        // Reset gamma when popup is destroyed (app exit)
        if let Ok(sf) = self.software_filter.lock() {
            if let Err(e) = sf.reset_gamma() {
                tracing::warn!("Failed to reset gamma on drop: {}", e);
            }
        }
    }
}
