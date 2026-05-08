//! Brightness slider widget with debounced change notifications.

use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{Adjustment, Box, Label, Orientation, Scale, SpinButton};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::Mutex;

/// Brightness slider widget.
#[derive(Clone)]
pub struct BrightnessSlider {
    /// Container widget
    container: Box,
    /// The scale slider
    scale: Scale,
    /// The spin button for precise input
    spin_button: SpinButton,
    /// The adjustment for both widgets
    adjustment: Adjustment,
    /// Current brightness value
    current_value: Arc<Mutex<u16>>,
    /// Whether the next value_changed is programmatic (should not trigger callback)
    suppress_callback: Arc<AtomicBool>,
}

impl BrightnessSlider {
    /// Create a new brightness slider.
    pub fn new() -> Self {
        let adjustment = Adjustment::new(50.0, 0.0, 100.0, 1.0, 5.0, 0.0);

        // Create scale
        let scale = Scale::builder()
            .orientation(Orientation::Horizontal)
            .adjustment(&adjustment)
            .hexpand(true)
            .draw_value(true)
            .has_origin(true)
            .value_pos(gtk4::PositionType::Bottom)
            .build();

        scale.add_mark(0.0, gtk4::PositionType::Bottom, Some("0"));
        scale.add_mark(25.0, gtk4::PositionType::Bottom, Some("25"));
        scale.add_mark(50.0, gtk4::PositionType::Bottom, Some("50"));
        scale.add_mark(75.0, gtk4::PositionType::Bottom, Some("75"));
        scale.add_mark(100.0, gtk4::PositionType::Bottom, Some("100"));

        // Create spin button
        let spin_button = SpinButton::builder()
            .adjustment(&adjustment)
            .climb_rate(1.0)
            .digits(0)
            .numeric(true)
            .build();

        // Create container
        let container = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(8)
            .margin_top(8)
            .margin_bottom(8)
            .margin_start(8)
            .margin_end(8)
            .build();

        // Add label
        let label = Label::builder()
            .label("Brightness")
            .halign(gtk4::Align::Start)
            .build();

        container.append(&label);
        container.append(&scale);
        container.append(&spin_button);

        Self {
            container,
            scale,
            spin_button,
            adjustment,
            current_value: Arc::new(Mutex::new(50)),
            suppress_callback: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Get the container widget.
    pub fn widget(&self) -> &Box {
        &self.container
    }

    /// Set the brightness value programmatically.
    /// Does NOT trigger the on_change callback.
    pub async fn set_value(&self, value: u16) {
        let value = value.clamp(0, 100);
        self.suppress_callback.store(true, Ordering::SeqCst);
        self.adjustment.set_value(value as f64);
        *self.current_value.lock().await = value;
    }

    /// Get the current brightness value.
    pub async fn get_value(&self) -> u16 {
        *self.current_value.lock().await
    }

    /// Set the callback for value changes, debounced by 300ms.
    ///
    /// The callback is only called once the slider has stopped moving
    /// for 300ms. This avoids sending dozens of DDC commands while
    /// the user drags the slider. Programmatic set_value() calls
    /// do NOT trigger the callback.
    pub fn set_on_change<F>(&mut self, callback: F)
    where
        F: Fn(u16) + Clone + Send + Sync + 'static,
    {
        let callback_clone = callback.clone();
        let current_value = self.current_value.clone();
        let suppress = self.suppress_callback.clone();

        // Track the pending timer source ID as raw u32.
        // We use raw g_source_remove() which does not panic on already-removed sources.
        let pending_source_id: Arc<std::sync::Mutex<Option<u32>>> =
            Arc::new(std::sync::Mutex::new(None));

        self.adjustment.connect_value_changed(move |adj| {
            // Skip if this was a programmatic set_value() call
            if suppress.swap(false, Ordering::SeqCst) {
                return;
            }

            let value = adj.value() as u16;

            // Update tracked value immediately
            {
                let mut current = current_value.blocking_lock();
                *current = value;
            }

            // Cancel any previous pending timer.
            // g_source_remove() returns FALSE if source already fired — that's fine, ignore it.
            if let Some(old_id) = pending_source_id.lock().ok().and_then(|mut id| id.take()) {
                unsafe {
                    gtk4::glib::ffi::g_source_remove(old_id);
                }
            }

            // Set a new 300ms debounce timer
            let cb = callback_clone.clone();
            let new_id = glib::timeout_add_local(std::time::Duration::from_millis(300), move || {
                cb(value);
                glib::ControlFlow::Break
            });

            // Store the raw source ID (u32)
            // SourceId doesn't expose as_raw(), but we can get it via IntoGlib
            let raw_id: u32 = unsafe { std::mem::transmute(new_id) };
            if let Ok(mut id) = pending_source_id.lock() {
                *id = Some(raw_id);
            }
        });
    }

    /// Set the sensitivity of the slider.
    pub fn set_sensitive(&self, sensitive: bool) {
        self.scale.set_sensitive(sensitive);
        self.spin_button.set_sensitive(sensitive);
    }

    /// Set the tooltip text.
    pub fn set_tooltip_text(&self, text: &str) {
        self.scale.set_tooltip_text(Some(text));
        self.spin_button.set_tooltip_text(Some(text));
    }
}

impl Default for BrightnessSlider {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Returns true if GTK initialized successfully (display available).
    fn init_gtk() -> bool {
        gtk4::init().is_ok()
    }

    #[test]
    fn test_brightness_slider_new() {
        if !init_gtk() {
            eprintln!("skipping: no display available");
            return;
        }
        let slider = BrightnessSlider::new();
        assert_eq!(slider.container.orientation(), Orientation::Vertical);
    }
}
