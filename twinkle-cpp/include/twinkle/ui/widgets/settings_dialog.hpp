#pragma once
/// @file settings_dialog.hpp
/// @brief GTK4 settings dialog with 4 tabs: General, UI, Behavior, Advanced.

#include <gtk/gtk.h>
#include <cstdint>
#include <memory>

namespace twinkle::core { class ConfigManager; }

namespace twinkle::ui::widgets {

class SettingsDialog {
public:
    SettingsDialog(GtkWindow* parent, std::shared_ptr<core::ConfigManager> cfg);
    ~SettingsDialog();

    SettingsDialog(const SettingsDialog&) = delete;
    SettingsDialog& operator=(const SettingsDialog&) = delete;

    void show();
    void hide();
    [[nodiscard]] GtkWindow* window() const noexcept { return dialog_; }

private:
    GtkWindow* dialog_{nullptr};
    GtkNotebook* notebook_{nullptr};

    // General tab
    GtkSwitch* autostart_switch_{nullptr};
    GtkDropDown* theme_dropdown_{nullptr};

    // UI tab
    GtkSpinButton* auto_hide_spin_{nullptr};
    GtkSwitch* show_monitor_switch_{nullptr};
    GtkSwitch* enable_presets_switch_{nullptr};

    // Behavior tab
    GtkSpinButton* debounce_spin_{nullptr};
    GtkSwitch* remember_brightness_switch_{nullptr};
    GtkSwitch* restore_brightness_switch_{nullptr};

    // Advanced tab
    GtkSpinButton* timeout_spin_{nullptr};
    GtkSpinButton* retries_spin_{nullptr};
    GtkSwitch* debug_logging_switch_{nullptr};

    std::shared_ptr<core::ConfigManager> config_;

    void build_general_tab();
    void build_ui_tab();
    void build_behavior_tab();
    void build_advanced_tab();
    void apply_settings();
};

} // namespace twinkle::ui::widgets
