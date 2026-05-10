#pragma once
/// @file brightness_slider.hpp
/// @brief Debounced brightness slider widget (GTK4).

#include <gtk/gtk.h>
#include <cstdint>
#include <functional>

namespace twinkle::ui::widgets {

/// Brightness slider with 300ms debounce.
class BrightnessSlider {
public:
    BrightnessSlider();
    ~BrightnessSlider();

    BrightnessSlider(const BrightnessSlider&) = delete;
    BrightnessSlider& operator=(const BrightnessSlider&) = delete;

    /// Get the GTK widget.
    [[nodiscard]] GtkWidget* widget() const noexcept { return container_; }

    /// Set value programmatically (does NOT trigger callback).
    void set_value(uint8_t value);

    /// Get current value.
    [[nodiscard]] uint8_t get_value() const noexcept { return current_value_; }

    /// Set the change callback (debounced 300ms).
    void set_on_change(std::function<void(uint16_t)> callback);

    /// Enable/disable the slider.
    void set_enabled(bool enabled);

private:
    GtkWidget* container_{nullptr};
    GtkWidget* scale_{nullptr};
    GtkAdjustment* adjustment_{nullptr};
    uint8_t current_value_{50};
    bool suppress_{false};
    guint debounce_source_id_{0};
    std::function<void(uint16_t)> callback_;

    static void on_value_changed(GtkAdjustment* adj, gpointer user_data);
};

} // namespace twinkle::ui::widgets
