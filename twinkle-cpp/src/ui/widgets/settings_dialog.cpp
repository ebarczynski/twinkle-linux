/// @file settings_dialog.cpp
/// @brief GTK4 settings dialog with 4 tabs.

#include "twinkle/ui/widgets/settings_dialog.hpp"
#include "twinkle/core/config.hpp"
#include "twinkle/core/logger.hpp"

namespace twinkle::ui::widgets {

SettingsDialog::SettingsDialog(GtkWindow* parent,
                               std::shared_ptr<core::ConfigManager> cfg)
    : config_(std::move(cfg)) {

    dialog_ = GTK_WINDOW(gtk_dialog_new_with_buttons(
        "Twinkle Linux Settings",
        parent,
        GTK_DIALOG_MODAL | GTK_DIALOG_DESTROY_WITH_PARENT,
        "_Cancel", GTK_RESPONSE_CANCEL,
        "_Apply", GTK_RESPONSE_APPLY,
        "_OK", GTK_RESPONSE_OK,
        nullptr));

    gtk_window_set_default_size(dialog_, 500, 450);

    notebook_ = GTK_NOTEBOOK(gtk_notebook_new());
    gtk_widget_set_margin_top(GTK_WIDGET(notebook_), 8);
    gtk_widget_set_margin_bottom(GTK_WIDGET(notebook_), 8);
    gtk_widget_set_margin_start(GTK_WIDGET(notebook_), 8);
    gtk_widget_set_margin_end(GTK_WIDGET(notebook_), 8);

    build_general_tab();
    build_ui_tab();
    build_behavior_tab();
    build_advanced_tab();

    auto* content = gtk_dialog_get_content_area(GTK_DIALOG(dialog_));
    gtk_box_append(GTK_BOX(content), GTK_WIDGET(notebook_));

    g_signal_connect(dialog_, "response",
        G_CALLBACK(SettingsDialog::on_response), this);
}

SettingsDialog::~SettingsDialog() {
    if (dialog_) gtk_window_destroy(dialog_);
}

void SettingsDialog::show() { gtk_window_present(dialog_); }
void SettingsDialog::hide() { gtk_widget_set_visible(GTK_WIDGET(dialog_), FALSE); }

void SettingsDialog::build_general_tab() {
    auto* vbox = gtk_box_new(GTK_ORIENTATION_VERTICAL, 10);
    gtk_widget_set_margin_top(vbox, 10);
    gtk_widget_set_margin_bottom(vbox, 10);
    gtk_widget_set_margin_start(vbox, 10);
    gtk_widget_set_margin_end(vbox, 10);

    // Autostart
    autostart_switch_ = GTK_SWITCH(gtk_switch_new());
    auto* autostart_row = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 8);
    auto* autostart_lbl = gtk_label_new("Start on system login");
    gtk_widget_set_hexpand(autostart_lbl, TRUE);
    gtk_widget_set_halign(autostart_lbl, GTK_ALIGN_START);
    gtk_box_append(GTK_BOX(autostart_row), autostart_lbl);
    gtk_box_append(GTK_BOX(autostart_row), GTK_WIDGET(autostart_switch_));
    gtk_box_append(GTK_BOX(vbox), autostart_row);

    // Theme
    auto* theme_row = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 8);
    auto* theme_lbl = gtk_label_new("Theme:");
    theme_combo_ = GTK_COMBO_BOX_TEXT(gtk_combo_box_text_new());
    gtk_combo_box_text_append(theme_combo_, "system", "System");
    gtk_combo_box_text_append(theme_combo_, "light", "Light");
    gtk_combo_box_text_append(theme_combo_, "dark", "Dark");
    gtk_combo_box_set_active_id(GTK_COMBO_BOX(theme_combo_), "system");
    gtk_box_append(GTK_BOX(theme_row), theme_lbl);
    gtk_box_append(GTK_BOX(theme_row), GTK_WIDGET(theme_combo_));
    gtk_box_append(GTK_BOX(vbox), theme_row);

    gtk_notebook_append_page(notebook_, vbox, gtk_label_new("General"));
}

void SettingsDialog::build_ui_tab() {
    auto* vbox = gtk_box_new(GTK_ORIENTATION_VERTICAL, 10);
    gtk_widget_set_margin_top(vbox, 10);
    gtk_widget_set_margin_bottom(vbox, 10);
    gtk_widget_set_margin_start(vbox, 10);
    gtk_widget_set_margin_end(vbox, 10);

    // Auto-hide delay
    auto* hide_row = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 8);
    auto* hide_lbl = gtk_label_new("Auto-hide delay (ms):");
    auto_hide_spin_ = GTK_SPIN_BUTTON(gtk_spin_button_new_with_range(0, 30000, 100));
    gtk_spin_button_set_value(auto_hide_spin_, 3000);
    gtk_box_append(GTK_BOX(hide_row), hide_lbl);
    gtk_box_append(GTK_BOX(hide_row), GTK_WIDGET(auto_hide_spin_));
    gtk_box_append(GTK_BOX(vbox), hide_row);

    // Show monitor selector
    show_monitor_switch_ = GTK_SWITCH(gtk_switch_new());
    auto* mon_row = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 8);
    auto* mon_lbl = gtk_label_new("Show monitor selector");
    gtk_widget_set_hexpand(mon_lbl, TRUE);
    gtk_widget_set_halign(mon_lbl, GTK_ALIGN_START);
    gtk_box_append(GTK_BOX(mon_row), mon_lbl);
    gtk_box_append(GTK_BOX(mon_row), GTK_WIDGET(show_monitor_switch_));
    gtk_box_append(GTK_BOX(vbox), mon_row);

    // Enable presets
    enable_presets_switch_ = GTK_SWITCH(gtk_switch_new());
    gtk_switch_set_active(enable_presets_switch_, TRUE);
    auto* preset_row = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 8);
    auto* preset_lbl = gtk_label_new("Enable quick presets");
    gtk_widget_set_hexpand(preset_lbl, TRUE);
    gtk_widget_set_halign(preset_lbl, GTK_ALIGN_START);
    gtk_box_append(GTK_BOX(preset_row), preset_lbl);
    gtk_box_append(GTK_BOX(preset_row), GTK_WIDGET(enable_presets_switch_));
    gtk_box_append(GTK_BOX(vbox), preset_row);

    gtk_notebook_append_page(notebook_, vbox, gtk_label_new("UI"));
}

void SettingsDialog::build_behavior_tab() {
    auto* vbox = gtk_box_new(GTK_ORIENTATION_VERTICAL, 10);
    gtk_widget_set_margin_top(vbox, 10);
    gtk_widget_set_margin_bottom(vbox, 10);
    gtk_widget_set_margin_start(vbox, 10);
    gtk_widget_set_margin_end(vbox, 10);

    // Debounce delay
    auto* deb_row = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 8);
    auto* deb_lbl = gtk_label_new("Debounce delay (ms):");
    debounce_spin_ = GTK_SPIN_BUTTON(gtk_spin_button_new_with_range(50, 1000, 50));
    gtk_spin_button_set_value(debounce_spin_, 300);
    gtk_box_append(GTK_BOX(deb_row), deb_lbl);
    gtk_box_append(GTK_BOX(deb_row), GTK_WIDGET(debounce_spin_));
    gtk_box_append(GTK_BOX(vbox), deb_row);

    // Remember brightness
    remember_brightness_switch_ = GTK_SWITCH(gtk_switch_new());
    gtk_switch_set_active(remember_brightness_switch_, TRUE);
    auto* rem_row = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 8);
    auto* rem_lbl = gtk_label_new("Remember brightness per monitor");
    gtk_widget_set_hexpand(rem_lbl, TRUE);
    gtk_widget_set_halign(rem_lbl, GTK_ALIGN_START);
    gtk_box_append(GTK_BOX(rem_row), rem_lbl);
    gtk_box_append(GTK_BOX(rem_row), GTK_WIDGET(remember_brightness_switch_));
    gtk_box_append(GTK_BOX(vbox), rem_row);

    // Restore on startup
    restore_brightness_switch_ = GTK_SWITCH(gtk_switch_new());
    gtk_switch_set_active(restore_brightness_switch_, TRUE);
    auto* rest_row = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 8);
    auto* rest_lbl = gtk_label_new("Restore brightness on startup");
    gtk_widget_set_hexpand(rest_lbl, TRUE);
    gtk_widget_set_halign(rest_lbl, GTK_ALIGN_START);
    gtk_box_append(GTK_BOX(rest_row), rest_lbl);
    gtk_box_append(GTK_BOX(rest_row), GTK_WIDGET(restore_brightness_switch_));
    gtk_box_append(GTK_BOX(vbox), rest_row);

    gtk_notebook_append_page(notebook_, vbox, gtk_label_new("Behavior"));
}

void SettingsDialog::build_advanced_tab() {
    auto* vbox = gtk_box_new(GTK_ORIENTATION_VERTICAL, 10);
    gtk_widget_set_margin_top(vbox, 10);
    gtk_widget_set_margin_bottom(vbox, 10);
    gtk_widget_set_margin_start(vbox, 10);
    gtk_widget_set_margin_end(vbox, 10);

    // Command timeout
    auto* to_row = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 8);
    auto* to_lbl = gtk_label_new("Command timeout (ms):");
    timeout_spin_ = GTK_SPIN_BUTTON(gtk_spin_button_new_with_range(500, 30000, 100));
    gtk_spin_button_set_value(timeout_spin_, 3000);
    gtk_box_append(GTK_BOX(to_row), to_lbl);
    gtk_box_append(GTK_BOX(to_row), GTK_WIDGET(timeout_spin_));
    gtk_box_append(GTK_BOX(vbox), to_row);

    // Max retries
    auto* ret_row = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 8);
    auto* ret_lbl = gtk_label_new("Max retries:");
    retries_spin_ = GTK_SPIN_BUTTON(gtk_spin_button_new_with_range(0, 10, 1));
    gtk_spin_button_set_value(retries_spin_, 1);
    gtk_box_append(GTK_BOX(ret_row), ret_lbl);
    gtk_box_append(GTK_BOX(ret_row), GTK_WIDGET(retries_spin_));
    gtk_box_append(GTK_BOX(vbox), ret_row);

    // Debug logging
    debug_logging_switch_ = GTK_SWITCH(gtk_switch_new());
    auto* dbg_row = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 8);
    auto* dbg_lbl = gtk_label_new("Debug logging");
    gtk_widget_set_hexpand(dbg_lbl, TRUE);
    gtk_widget_set_halign(dbg_lbl, GTK_ALIGN_START);
    gtk_box_append(GTK_BOX(dbg_row), dbg_lbl);
    gtk_box_append(GTK_BOX(dbg_row), GTK_WIDGET(debug_logging_switch_));
    gtk_box_append(GTK_BOX(vbox), dbg_row);

    gtk_notebook_append_page(notebook_, vbox, gtk_label_new("Advanced"));
}

void SettingsDialog::apply_settings() {
    auto& config = config_->config_mut();
    config.general.autostart = gtk_switch_get_active(autostart_switch_);
    config.ui.auto_hide_delay_ms = static_cast<uint32_t>(gtk_spin_button_get_value_as_int(auto_hide_spin_));
    config.ui.show_monitor_selector = gtk_switch_get_active(show_monitor_switch_);
    config.ui.enable_presets = gtk_switch_get_active(enable_presets_switch_);
    config.behavior.debounce_delay_ms = static_cast<uint32_t>(gtk_spin_button_get_value_as_int(debounce_spin_));
    config.behavior.remember_brightness = gtk_switch_get_active(remember_brightness_switch_);
    config.behavior.restore_brightness = gtk_switch_get_active(restore_brightness_switch_);
    config.advanced.command_timeout_ms = static_cast<uint32_t>(gtk_spin_button_get_value_as_int(timeout_spin_));
    config.advanced.max_retries = static_cast<uint32_t>(gtk_spin_button_get_value_as_int(retries_spin_));
    config.advanced.debug_logging = gtk_switch_get_active(debug_logging_switch_);

    if (auto result = config_->save(); !result) {
        LOG_ERROR("Failed to save config");
    } else {
        LOG_INFO("Settings saved");
    }
}

void SettingsDialog::on_response(GtkDialog* dialog, int response_id, gpointer user_data) {
    auto* self = static_cast<SettingsDialog*>(user_data);

    switch (response_id) {
        case GTK_RESPONSE_OK:
            self->apply_settings();
            self->hide();
            break;
        case GTK_RESPONSE_APPLY:
            self->apply_settings();
            break;
        case GTK_RESPONSE_CANCEL:
        default:
            self->hide();
            break;
    }
}

} // namespace twinkle::ui::widgets
