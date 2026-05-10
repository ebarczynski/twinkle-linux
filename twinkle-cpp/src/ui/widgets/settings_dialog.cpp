/// @file settings_dialog.cpp
/// @brief GTK4 settings dialog with 4 tabs.
///
/// Uses GtkWindow directly (GtkDialog is deprecated in GTK4 4.14).
/// Uses GtkDropDown instead of GtkComboBoxText.

#include "twinkle/ui/widgets/settings_dialog.hpp"
#include "twinkle/core/config.hpp"
#include "twinkle/core/logger.hpp"

#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Wdeprecated-declarations"

namespace twinkle::ui::widgets {

SettingsDialog::SettingsDialog(GtkWindow* parent,
                               std::shared_ptr<core::ConfigManager> cfg)
    : config_(std::move(cfg)) {

    dialog_ = GTK_WINDOW(gtk_window_new());
    gtk_window_set_title(dialog_, "Twinkle Linux Settings");
    gtk_window_set_transient_for(dialog_, parent);
    gtk_window_set_modal(dialog_, TRUE);
    gtk_window_set_default_size(dialog_, 500, 450);

    // Main vertical box: content area + button bar
    auto* vbox = gtk_box_new(GTK_ORIENTATION_VERTICAL, 0);

    notebook_ = GTK_NOTEBOOK(gtk_notebook_new());
    gtk_widget_set_margin_top(GTK_WIDGET(notebook_), 8);
    gtk_widget_set_margin_bottom(GTK_WIDGET(notebook_), 8);
    gtk_widget_set_margin_start(GTK_WIDGET(notebook_), 8);
    gtk_widget_set_margin_end(GTK_WIDGET(notebook_), 8);
    gtk_widget_set_vexpand(GTK_WIDGET(notebook_), TRUE);
    gtk_box_append(GTK_BOX(vbox), GTK_WIDGET(notebook_));

    // Button bar
    auto* btn_bar = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 8);
    gtk_widget_set_margin_top(btn_bar, 8);
    gtk_widget_set_margin_bottom(btn_bar, 8);
    gtk_widget_set_margin_start(btn_bar, 8);
    gtk_widget_set_margin_end(btn_bar, 8);
    gtk_widget_set_halign(btn_bar, GTK_ALIGN_END);

    auto* cancel_btn = gtk_button_new_with_label("Cancel");
    auto* apply_btn = gtk_button_new_with_label("Apply");
    gtk_widget_add_css_class(apply_btn, "suggested-action");
    auto* ok_btn = gtk_button_new_with_label("OK");
    gtk_widget_add_css_class(ok_btn, "suggested-action");

    gtk_box_append(GTK_BOX(btn_bar), cancel_btn);
    gtk_box_append(GTK_BOX(btn_bar), apply_btn);
    gtk_box_append(GTK_BOX(btn_bar), ok_btn);
    gtk_box_append(GTK_BOX(vbox), btn_bar);

    gtk_window_set_child(dialog_, vbox);

    build_general_tab();
    build_ui_tab();
    build_behavior_tab();
    build_advanced_tab();

    // Button signals
    g_signal_connect(cancel_btn, "clicked",
        G_CALLBACK(+[](GtkWidget*, gpointer ud) {
            auto* self = static_cast<SettingsDialog*>(ud);
            self->hide();
        }), this);

    g_signal_connect(apply_btn, "clicked",
        G_CALLBACK(+[](GtkWidget*, gpointer ud) {
            auto* self = static_cast<SettingsDialog*>(ud);
            self->apply_settings();
        }), this);

    g_signal_connect(ok_btn, "clicked",
        G_CALLBACK(+[](GtkWidget*, gpointer ud) {
            auto* self = static_cast<SettingsDialog*>(ud);
            self->apply_settings();
            self->hide();
        }), this);

    // Close window via Escape / window manager
    g_signal_connect(dialog_, "close-request",
        G_CALLBACK(+[](GtkWidget*, gpointer ud) -> gboolean {
            auto* self = static_cast<SettingsDialog*>(ud);
            self->hide();
            return TRUE; // prevent default destroy
        }), this);
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

    // Theme — use GtkDropDown with string list
    auto* theme_row = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 8);
    auto* theme_lbl = gtk_label_new("Theme:");
    auto* theme_model = gtk_string_list_new(new const char*[3]{"System", "Light", "Dark"});
    theme_dropdown_ = GTK_DROP_DOWN(gtk_drop_down_new(G_LIST_MODEL(theme_model), nullptr));
    gtk_box_append(GTK_BOX(theme_row), theme_lbl);
    gtk_box_append(GTK_BOX(theme_row), GTK_WIDGET(theme_dropdown_));
    gtk_box_append(GTK_BOX(vbox), theme_row);

    gtk_notebook_append_page(notebook_, vbox, gtk_label_new("General"));
}

void SettingsDialog::build_ui_tab() {
    auto* vbox = gtk_box_new(GTK_ORIENTATION_VERTICAL, 10);
    gtk_widget_set_margin_top(vbox, 10);
    gtk_widget_set_margin_bottom(vbox, 10);
    gtk_widget_set_margin_start(vbox, 10);
    gtk_widget_set_margin_end(vbox, 10);

    auto* hide_row = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 8);
    auto* hide_lbl = gtk_label_new("Auto-hide delay (ms):");
    auto_hide_spin_ = GTK_SPIN_BUTTON(gtk_spin_button_new_with_range(0, 30000, 100));
    gtk_spin_button_set_value(auto_hide_spin_, 3000);
    gtk_box_append(GTK_BOX(hide_row), hide_lbl);
    gtk_box_append(GTK_BOX(hide_row), GTK_WIDGET(auto_hide_spin_));
    gtk_box_append(GTK_BOX(vbox), hide_row);

    show_monitor_switch_ = GTK_SWITCH(gtk_switch_new());
    auto* mon_row = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 8);
    auto* mon_lbl = gtk_label_new("Show monitor selector");
    gtk_widget_set_hexpand(mon_lbl, TRUE);
    gtk_widget_set_halign(mon_lbl, GTK_ALIGN_START);
    gtk_box_append(GTK_BOX(mon_row), mon_lbl);
    gtk_box_append(GTK_BOX(mon_row), GTK_WIDGET(show_monitor_switch_));
    gtk_box_append(GTK_BOX(vbox), mon_row);

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

    auto* deb_row = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 8);
    auto* deb_lbl = gtk_label_new("Debounce delay (ms):");
    debounce_spin_ = GTK_SPIN_BUTTON(gtk_spin_button_new_with_range(50, 1000, 50));
    gtk_spin_button_set_value(debounce_spin_, 300);
    gtk_box_append(GTK_BOX(deb_row), deb_lbl);
    gtk_box_append(GTK_BOX(deb_row), GTK_WIDGET(debounce_spin_));
    gtk_box_append(GTK_BOX(vbox), deb_row);

    remember_brightness_switch_ = GTK_SWITCH(gtk_switch_new());
    gtk_switch_set_active(remember_brightness_switch_, TRUE);
    auto* rem_row = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 8);
    auto* rem_lbl = gtk_label_new("Remember brightness per monitor");
    gtk_widget_set_hexpand(rem_lbl, TRUE);
    gtk_widget_set_halign(rem_lbl, GTK_ALIGN_START);
    gtk_box_append(GTK_BOX(rem_row), rem_lbl);
    gtk_box_append(GTK_BOX(rem_row), GTK_WIDGET(remember_brightness_switch_));
    gtk_box_append(GTK_BOX(vbox), rem_row);

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

    auto* to_row = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 8);
    auto* to_lbl = gtk_label_new("Command timeout (ms):");
    timeout_spin_ = GTK_SPIN_BUTTON(gtk_spin_button_new_with_range(500, 30000, 100));
    gtk_spin_button_set_value(timeout_spin_, 3000);
    gtk_box_append(GTK_BOX(to_row), to_lbl);
    gtk_box_append(GTK_BOX(to_row), GTK_WIDGET(timeout_spin_));
    gtk_box_append(GTK_BOX(vbox), to_row);

    auto* ret_row = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 8);
    auto* ret_lbl = gtk_label_new("Max retries:");
    retries_spin_ = GTK_SPIN_BUTTON(gtk_spin_button_new_with_range(0, 10, 1));
    gtk_spin_button_set_value(retries_spin_, 1);
    gtk_box_append(GTK_BOX(ret_row), ret_lbl);
    gtk_box_append(GTK_BOX(ret_row), GTK_WIDGET(retries_spin_));
    gtk_box_append(GTK_BOX(vbox), ret_row);

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

} // namespace twinkle::ui::widgets

#pragma GCC diagnostic pop
