#pragma once
/// @file settings_dialog.hpp
/// @brief GTK4 settings dialog with 4 tabs: General, UI, Behavior, Advanced.

#include <gtk/gtk.h>
#include <cstdint>
#include <memory>

namespace twinkle::core { class ConfigManager; }

namespace twinkle::ui::widgets {

/// Settings dialog.
class SettingsDialog {
public:
    SettingsDialog(GtkWindow* parent, std::shared_ptr<core::ConfigManager> cfg);
    ~SettingsDialog();

    SettingsDialog(const SettingsDialog&) = delete;
    SettingsDialog& operator=(const SettingsDialog&) = delete;

    void show();
    void hide();

private:
    GtkWindow* dialog_{nullptr};
    GtkNotebook* notebook_{nullptr};
    std::shared_ptr<core::ConfigManager> config_;

    // General tab widgets
    GtkSwitch* autostart_switch_{nullptr};
    GtkComboBoxText* theme_combo_{nullptr};

    // UI tab widgets
    GtkSpinButton* auto_hide_spin_{nullptr};
    GtkSwitch* show_monitor_switch_{nullptr};
    GtkSwitch* enable_presets_switch_{nullptr};

    // Behavior tab widgets
    GtkSpinButton* debounce_spin_{nullptr};
    GtkSwitch* remember_brightness_switch_{nullptr};
    GtkSwitch* restore_brightness_switch_{nullptr};

    // Advanced tab widgets
    GtkSpinButton* timeout_spin_{nullptr};
    GtkSpinButton* retries_spin_{nullptr};
    GtkSwitch* debug_logging_switch_{nullptr};

    void build_general_tab();
    void build_ui_tab();
    void build_behavior_tab();
    void build_advanced_tab();
    void apply_settings();

    static void on_response(GtkDialog* dialog, int response_id, gpointer user_data);
};

} // namespace twinkle::ui::widgets
