//! Quick brightness slider — minimal borderless panel shown directly on
//! tray click.
//!
//! One "All Monitors" slider with live percentage and snap-to-preset
//! behavior. No title bar, no decorations: the shell menu cannot host a
//! real slider (dbusmenu items are text-only), so this panel is the closest
//! native equivalent — it appears instantly and closes on focus loss,
//! Escape, or after 10 s of inactivity. The full per-monitor panel remains
//! available via the tray menu ("Brightness details").

use crate::core::config::ConfigManager;
use crate::core::schedule::ScheduleEvent;
use crate::ddc::DDCManager;
use crate::utils::{normalized_presets, snap_to_preset};
use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{Adjustment, Box, GestureClick, Label, Orientation, Scale, Window};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::sync::Mutex as StdMutex;
use tokio::sync::Mutex;

/// How long the quick panel stays open without slider interaction.
const AUTO_HIDE_SECS: u64 = 10;

pub struct QuickPopup {
    window: Window,
    adjustment: Adjustment,
    scale: Scale,
    value_label: Label,
    suppress: Arc<AtomicBool>,
    ddc_manager: Arc<DDCManager>,
    config_manager: Arc<Mutex<ConfigManager>>,
    snap_presets: Arc<StdMutex<Vec<u16>>>,
    manual_tx: Option<tokio::sync::mpsc::UnboundedSender<ScheduleEvent>>,
    hide_timer: Arc<StdMutex<Option<u32>>>,
    _css_provider: gtk4::CssProvider,
}

impl QuickPopup {
    pub async fn new(
        ddc_manager: Arc<DDCManager>,
        config_manager: Arc<Mutex<ConfigManager>>,
        manual_tx: Option<tokio::sync::mpsc::UnboundedSender<ScheduleEvent>>,
    ) -> Self {
        let css_provider = gtk4::CssProvider::new();
        css_provider.load_from_data(include_str!("style.css"));
        gtk4::style_context_add_provider_for_display(
            &gtk4::gdk::Display::default().unwrap_or_else(|| panic!("No display")),
            &css_provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        let main_box = Box::builder()
            .orientation(Orientation::Vertical)
            .css_classes(["all-monitors-row"])
            .build();

        // Header: label + live percentage
        let header = Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .build();
        let icon = Label::new(Some("\u{2600}"));
        icon.set_css_classes(&["sun-icon"]);
        let name = Label::builder()
            .label("All Monitors")
            .css_classes(["monitor-name"])
            .hexpand(true)
            .xalign(0.0)
            .build();
        let value_label = Label::builder()
            .label("—")
            .css_classes(["brightness-value"])
            .build();
        header.append(&icon);
        header.append(&name);
        header.append(&value_label);
        main_box.append(&header);

        // Slider with snap marks
        let adjustment = Adjustment::new(50.0, 0.0, 100.0, 1.0, 10.0, 0.0);
        let scale = Scale::builder()
            .adjustment(&adjustment)
            .hexpand(true)
            .draw_value(false)
            .build();
        main_box.append(&scale);

        // Undecorated, non-resizable toplevel: click tray → slider, nothing else.
        let window = Window::builder()
            .title("Brightness")
            .decorated(false)
            .resizable(false)
            .default_width(300)
            .focusable(true)
            .build();
        window.set_child(Some(&main_box));

        let suppress = Arc::new(AtomicBool::new(false));
        let initial_presets = {
            let cfg = config_manager.blocking_lock();
            normalized_presets(
                &cfg.config().ui.preset_values,
                cfg.config().behavior.min_brightness,
            )
        };
        apply_marks(&scale, &initial_presets);
        let snap_presets = Arc::new(StdMutex::new(initial_presets));
        attach_snap(&scale, &adjustment, snap_presets.clone());

        let quick = Self {
            window: window.clone(),
            adjustment: adjustment.clone(),
            scale: scale.clone(),
            value_label: value_label.clone(),
            suppress,
            ddc_manager,
            config_manager,
            snap_presets,
            manual_tx,
            hide_timer: Arc::new(StdMutex::new(None)),
            _css_provider: css_provider,
        };

        quick.connect_slider(window);
        quick.connect_close_gestures();
        quick
    }

    /// Wire the slider: live percentage + debounced send to all monitors.
    fn connect_slider(&self, window: Window) {
        let ddc = self.ddc_manager.clone();
        let config = self.config_manager.clone();
        let manual_tx = self.manual_tx.clone();
        let suppress = self.suppress.clone();
        let value_label = self.value_label.clone();
        let hide_timer = self.hide_timer.clone();

        self.adjustment.connect_value_changed(move |adj| {
            if suppress.swap(false, Ordering::SeqCst) {
                return; // programmatic update while syncing to real state
            }
            let value = adj.value() as u16;
            value_label.set_label(&format!("{}%", value));

            // Any interaction postpones auto-hide.
            reschedule_auto_hide(&hide_timer, &window);

            let ddc = ddc.clone();
            let config = config.clone();
            let manual_tx = manual_tx.clone();
            glib::spawn_future_local(async move {
                let min = {
                    let cfg = config.blocking_lock();
                    cfg.config().behavior.min_brightness
                };
                let clamped = value.max(min);
                let monitors = ddc.get_monitors().await;
                for m in &monitors {
                    if let Err(e) = ddc.set_brightness(&m.unique_id(), clamped).await {
                        tracing::warn!("Quick slider: failed on {}: {}", m.display_name(), e);
                    }
                }
                // Manual change suspends the day/night schedule for this phase.
                if let Some(tx) = &manual_tx {
                    let _ = tx.send(ScheduleEvent::ManualChange);
                }
            });
        });
    }
    /// Close on Escape or when focus moves elsewhere (click outside).
    fn connect_close_gestures(&self) {
        let win = self.window.clone();
        let key = gtk4::EventControllerKey::new();
        key.connect_key_pressed(move |_, key, _code, _state| {
            if key == gtk4::gdk::Key::Escape {
                win.hide();
                return glib::Propagation::Proceed;
            }
            glib::Propagation::Proceed
        });
        self.window.add_controller(key);

        let win = self.window.clone();
        let focus = gtk4::EventControllerFocus::new();
        focus.connect_leave(move |_| {
            win.hide();
        });
        self.window.add_controller(focus);

        // Keep focus while the user interacts (drag start can briefly drop
        // focus depending on the WM); re-presenting is harmless if focused.
        let win = self.window.clone();
        let gesture = GestureClick::new();
        gesture.connect_pressed(move |_, _, _, _| {
            win.present();
        });
        self.window.add_controller(gesture);
    }

    /// Show the quick slider, synced to the average current brightness.
    pub fn popup(&self) {
        let ddc = self.ddc_manager.clone();
        let config = self.config_manager.clone();
        let adjustment = self.adjustment.clone();
        let suppress = self.suppress.clone();
        let value_label = self.value_label.clone();
        let scale = self.scale.clone();
        let snap_presets = self.snap_presets.clone();
        let window = self.window.clone();
        let hide_timer = self.hide_timer.clone();

        glib::spawn_future_local(async move {
            // Refresh snap marks from config (propagates live config edits).
            let (presets, min_brightness) = {
                let cfg = config.blocking_lock();
                let behavior = &cfg.config().behavior;
                (
                    normalized_presets(&cfg.config().ui.preset_values, behavior.min_brightness),
                    behavior.min_brightness,
                )
            };
            *snap_presets.lock().unwrap() = presets.clone();
            apply_marks(&scale, &presets);

            // Sync slider to the average current brightness of all monitors.
            let monitors = ddc.get_monitors().await;
            let mut sum: u32 = 0;
            let mut count: u32 = 0;
            for m in &monitors {
                if let Ok(b) = ddc.get_brightness(&m.unique_id()).await {
                    sum += u32::from(b.max(min_brightness));
                    count += 1;
                }
            }
            if count > 0 {
                let avg = (sum / count) as u16;
                suppress.store(true, Ordering::SeqCst);
                adjustment.set_value(f64::from(avg));
                value_label.set_label(&format!("{}%", avg));
            }

            window.present();
        });

        reschedule_auto_hide(&hide_timer, &self.window.clone());
    }

    pub fn popdown(&self) {
        self.window.hide();
    }

    pub fn widget(&self) -> &Window {
        &self.window
    }
}

/// Tick marks for each configured preset value.
fn apply_marks(scale: &Scale, presets: &[u16]) {
    scale.clear_marks();
    for value in presets {
        scale.add_mark(
            f64::from(*value),
            gtk4::PositionType::Bottom,
            Some(value.to_string().as_str()),
        );
    }
}

/// Snap to the nearest configured preset on pointer release.
fn attach_snap(scale: &Scale, adjustment: &Adjustment, snap_presets: Arc<StdMutex<Vec<u16>>>) {
    let gesture = GestureClick::new();
    let adjustment = adjustment.clone();
    gesture.connect_released(move |_g, _n, _x, _y| {
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
            tracing::debug!("Quick slider snap: {}% → {}%", raw, snapped);
            adjustment.set_value(f64::from(snapped));
        }
    });
    scale.add_controller(gesture);
}

/// (Re)arm the auto-hide timer; cancels any pending one first.
fn reschedule_auto_hide(hide_timer: &StdMutex<Option<u32>>, window: &Window) {
    let window = window.clone();
    if let Ok(mut slot) = hide_timer.lock() {
        if let Some(old) = slot.take() {
            unsafe {
                glib::ffi::g_source_remove(old);
            }
        }
        let new_id =
            glib::timeout_add_local(std::time::Duration::from_secs(AUTO_HIDE_SECS), move || {
                window.hide();
                glib::ControlFlow::Break
            });
        *slot = Some(new_id.as_raw());
    }
}
