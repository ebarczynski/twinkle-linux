#pragma once

#include <gtk/gtk.h>
#include <cstdint>
#include <functional>
#include <memory>

namespace twinkle::ui::widgets {

/// Callback type for brightness value changes
using BrightnessChangedCallback = std::function<void(uint8_t)>;

/// Brightness slider widget with debouncing
class BrightnessSlider {
public:
    BrightnessSlider();
    ~BrightnessSlider();

    // Non-copyable but movable
    BrightnessSlider(const BrightnessSlider&) = delete;
    BrightnessSlider& operator=(const BrightnessSlider&) = delete;
    BrightnessSlider(BrightnessSlider&&) noexcept = default;
    BrightnessSlider& operator=(BrightnessSlider&&) noexcept = default;

    /// Initialize slider
    [[nodiscard]] bool initialize();

    /// Get GTK widget
    [[nodiscard]] GtkWidget* widget() const noexcept { return slider_.get(); }

    /// Set current brightness value
    void set_value(uint8_t value);

    /// Get current brightness value
    [[nodiscard]] uint8_t get_value() const noexcept { return current_value_; }

    /// Set callback for value changes
    void set_callback(BrightnessChangedCallback callback) {
        callback_ = std::move(callback);
    }

    /// Set debounce time in milliseconds
    void set_debounce_ms(uint32_t ms) noexcept { debounce_ms_ = ms; }

    /// Enable/disable widget
    void set_enabled(bool enabled);

private:
    std::unique_ptr<GtkScale, decltype(&g_object_unref)> slider_;
    BrightnessChangedCallback callback_;
    uint8_t current_value_{50};
    uint32_t debounce_ms_{200};
    guint debounce_source_id_{0};

    /// Handle slider value change
    static void on_value_changed(GtkRange* range, gpointer user_data);

    /// Debounced callback
    gboolean on_debounced_callback();
};

} // namespace twinkle::ui::widgets
