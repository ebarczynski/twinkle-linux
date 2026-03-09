#pragma once

#include <gtk/gtk.h>
#include <cstdint>
#include <functional>
#include <memory>
#include <string>
#include <vector>

namespace twinkle::ui {

/// Callback types for brightness popup events
using BrightnessCallback = std::function<void(uint8_t)>;
using MonitorCallback = std::function<void(const std::string&)>;

/// Brightness popup window
class BrightnessPopup {
public:
    BrightnessPopup();
    ~BrightnessPopup();

    // Non-copyable but movable
    BrightnessPopup(const BrightnessPopup&) = delete;
    BrightnessPopup& operator=(const BrightnessPopup&) = delete;
    BrightnessPopup(BrightnessPopup&&) noexcept = default;
    BrightnessPopup& operator=(BrightnessPopup&&) noexcept = default;

    /// Initialize popup window
    [[nodiscard]] bool initialize();

    /// Show popup at cursor position
    void show();

    /// Hide popup
    void hide();

    /// Set current brightness value
    void set_brightness(uint8_t brightness);

    /// Get current brightness value
    [[nodiscard]] uint8_t get_brightness() const noexcept { return current_brightness_; }

    /// Set available monitors
    void set_monitors(const std::vector<std::string>& monitors);

    /// Set selected monitor
    void set_selected_monitor(const std::string& monitor_id);

    /// Set callback for brightness changes
    void set_brightness_callback(BrightnessCallback callback) {
        brightness_callback_ = std::move(callback);
    }

    /// Set callback for monitor selection
    void set_monitor_callback(MonitorCallback callback) {
        monitor_callback_ = std::move(callback);
    }

    /// Enable/disable monitor selector
    void set_monitor_selector_visible(bool visible);

private:
    std::unique_ptr<GtkWindow, decltype(&g_object_unref)> window_;
    std::unique_ptr<GtkScale, decltype(&g_object_unref)> brightness_slider_;
    std::unique_ptr<GtkComboBoxText, decltype(&g_object_unref)> monitor_selector_;
    std::unique_ptr<GtkLabel, decltype(&g_object_unref)> status_label_;
    std::vector<std::unique_ptr<GtkWidget, decltype(&g_object_unref)>> preset_buttons_;

    BrightnessCallback brightness_callback_;
    MonitorCallback monitor_callback_;
    uint8_t current_brightness_{50};
    bool monitor_selector_visible_{false};

    /// Create popup window
    [[nodiscard]] bool create_window();

    /// Create brightness slider
    [[nodiscard]] bool create_brightness_slider();

    /// Create monitor selector
    [[nodiscard]] bool create_monitor_selector();

    /// Create preset buttons
    [[nodiscard]] bool create_preset_buttons();

    /// Create status label
    [[nodiscard]] bool create_status_label();

    /// Handle brightness slider change
    static void on_brightness_changed(GtkRange* range, gpointer user_data);

    /// Handle monitor selection change
    static void on_monitor_changed(GtkComboBox* combo, gpointer user_data);

    /// Handle preset button click
    static void on_preset_click(GtkButton* button, gpointer user_data);

    /// Handle window hide
    static gboolean on_window_hide(GtkWidget* widget, GdkEvent* event, gpointer user_data);
};

} // namespace twinkle::ui
