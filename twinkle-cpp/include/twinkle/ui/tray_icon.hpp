#pragma once

#include <gtk/gtk.h>
#include <functional>
#include <memory>
#include <string>

namespace twinkle::ui {

/// Callback types for tray icon events
using TrayCallback = std::function<void()>;

/// System tray icon for Twinkle Linux
class TrayIcon {
public:
    TrayIcon();
    ~TrayIcon();

    // Non-copyable but movable
    TrayIcon(const TrayIcon&) = delete;
    TrayIcon& operator=(const TrayIcon&) = delete;
    TrayIcon(TrayIcon&&) noexcept = default;
    TrayIcon& operator=(TrayIcon&&) noexcept = default;

    /// Initialize tray icon
    [[nodiscard]] bool initialize();

    /// Show tray icon
    void show();

    /// Hide tray icon
    void hide();

    /// Set callback for brightness control click
    void set_brightness_callback(TrayCallback callback) {
        brightness_callback_ = std::move(callback);
    }

    /// Set callback for settings click
    void set_settings_callback(TrayCallback callback) {
        settings_callback_ = std::move(callback);
    }

    /// Set callback for quit
    void set_quit_callback(TrayCallback callback) {
        quit_callback_ = std::move(callback);
    }

    /// Update icon based on monitor state
    void update_icon(bool monitors_available);

private:
    std::unique_ptr<GtkStatusIcon, decltype(&g_object_unref)> status_icon_;
    TrayCallback brightness_callback_;
    TrayCallback settings_callback_;
    TrayCallback quit_callback_;

    /// Create menu
    [[nodiscard]] GtkWidget* create_menu();

    /// Handle menu item clicks
    static void on_menu_item_click(GtkWidget* widget, gpointer data);

    /// Handle tray icon click
    static void on_tray_click(GtkStatusIcon* status_icon, gpointer user_data);
};

} // namespace twinkle::ui
