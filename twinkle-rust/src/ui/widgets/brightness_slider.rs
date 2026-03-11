//! Brightness slider widget.

use gtk4::prelude::*;
use gtk4::{Adjustment, Box, Label, Orientation, Scale, SpinButton};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Brightness slider widget.
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
        }
    }

    /// Get the container widget.
    pub fn widget(&self) -> &Box {
        &self.container
    }

    /// Set the brightness value.
    pub async fn set_value(&self, value: u16) {
        let value = value.clamp(0, 100);
        self.adjustment.set_value(value as f64);
        *self.current_value.lock().await = value;
    }

    /// Get the current brightness value.
    pub async fn get_value(&self) -> u16 {
        *self.current_value.lock().await
    }

    /// Set the callback for value changes.
    pub fn set_on_change<F>(&mut self, callback: F)
    where
        F: Fn(u16) + Clone + Send + Sync + 'static,
    {
        let callback_clone = callback.clone();
        let current_value = self.current_value.clone();

        self.adjustment.connect_value_changed(move |adj| {
            let value = adj.value() as u16;
            let mut current = current_value.blocking_lock();
            *current = value;
            drop(current);

            callback_clone(value);
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

    #[test]
    fn test_brightness_slider_new() {
        let slider = BrightnessSlider::new();
        assert_eq!(slider.container.orientation(), Orientation::Vertical);
    }
}
