#pragma once

#include <gtk/gtk.h>
#include <functional>
#include <memory>
#include <string>
#include <vector>

namespace twinkle::ui::widgets {

/// Callback type for settings apply
using SettingsApplyCallback = std::function<void()>;

/// Settings dialog with tabs
class SettingsDialog {
public:
    SettingsDialog();
    ~SettingsDialog();

    // Non-copyable but movable
    SettingsDialog(const SettingsDialog&) = delete;
    SettingsDialog& operator=(const SettingsDialog&) = delete;
    SettingsDialog(SettingsDialog&&) noexcept = default;
    SettingsDialog& operator=(SettingsDialog&&) noexcept = default;

    /// Initialize dialog
    [[nodiscard]] bool initialize();

    /// Show dialog
    void show();

    /// Hide dialog
    void hide();

    /// Set callback for settings apply
    void set_apply_callback(SettingsApplyCallback callback) {
        apply_callback_ = std::move(callback);
    }

private:
    std::unique_ptr<GtkDialog, decltype(&g_object_unref)> dialog_;
    std::unique_ptr<GtkNotebook, decltype(&g_object_unref)> notebook_;
    SettingsApplyCallback apply_callback_;

    // General tab widgets
    std::unique_ptr<GtkCheckButton, decltype(&g_object_unref)> autostart_checkbox_;
    std::unique_ptr<GtkComboBoxText, decltype(&g_object_unref)> theme_combo_;

    // Behavior tab widgets
    std::unique_ptr<GtkSpinButton, decltype(&g_object_unref)> auto_hide_spin_;
    std::unique_ptr<GtkSpinButton, decltype(&g_object_unref)> step_size_spin_;
    std::unique_ptr<GtkCheckButton, decltype(&g_object_unref)> notifications_checkbox_;

    // Advanced tab widgets
    std::unique_ptr<GtkSpinButton, decltype(&g_object_unref)> timeout_spin_;
    std::unique_ptr<GtkSpinButton, decltype(&g_object_unref)> retries_spin_;
    std::unique_ptr<GtkCheckButton, decltype(&g_object_unref)> debug_checkbox_;

    /// Create dialog window
    [[nodiscard]] bool create_dialog();

    /// Create tabs
    [[nodiscard]] bool create_tabs();

    /// Create general tab
    [[nodiscard]] bool create_general_tab();

    /// Create behavior tab
    [[nodiscard]] bool create_behavior_tab();

    /// Create advanced tab
    [[nodiscard]] bool create_advanced_tab();

    /// Handle apply button click
    static void on_apply(GtkButton* button, gpointer user_data);

    /// Handle close button click
    static void on_close(GtkButton* button, gpointer user_data);

    /// Handle dialog response
    static void on_response(GtkDialog* dialog, gint response_id, gpointer user_data);
};

} // namespace twinkle::ui::widgets
