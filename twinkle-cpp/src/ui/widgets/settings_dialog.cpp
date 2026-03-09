#include "twinkle/ui/widgets/settings_dialog.hpp"

namespace twinkle::ui::widgets {

SettingsDialog::SettingsDialog()
    : dialog_(nullptr, g_object_unref),
      notebook_(nullptr, g_object_unref) {}

SettingsDialog::~SettingsDialog() {
    hide();
}

bool SettingsDialog::initialize() {
    return create_dialog();
}

void SettingsDialog::show() {
    if (dialog_) {
        gtk_window_present(GTK_WINDOW(dialog_.get()));
    }
}

void SettingsDialog::hide() {
    if (dialog_) {
        gtk_widget_hide(GTK_WIDGET(dialog_.get()));
    }
}

bool SettingsDialog::create_dialog() {
    // Create dialog
    dialog_.reset(gtk_dialog_new_with_buttons(
        "Settings",
        nullptr,
        GTK_DIALOG_MODAL | GTK_DIALOG_DESTROY_WITH_PARENT,
        "_Apply", GTK_RESPONSE_APPLY,
        "_Close", GTK_RESPONSE_CLOSE,
        nullptr));

    gtk_window_set_default_size(GTK_WINDOW(dialog_.get()), 500, 400);

    // Get content area
    GtkWidget* content_area = gtk_dialog_get_content_area(GTK_DIALOG(dialog_.get()));

    // Create notebook (tabs)
    if (!create_tabs()) {
        return false;
    }

    gtk_box_pack_start(GTK_BOX(content_area), GTK_WIDGET(notebook_.get()), TRUE, TRUE, 0);

    // Connect response signal
    g_signal_connect(dialog_.get(), "response",
                   G_CALLBACK(on_response), this);

    gtk_widget_show_all(GTK_WIDGET(dialog_.get()));

    return true;
}

bool SettingsDialog::create_tabs() {
    notebook_.reset(gtk_notebook_new());

    if (!create_general_tab() ||
        !create_behavior_tab() ||
        !create_advanced_tab()) {
        return false;
    }

    return true;
}

bool SettingsDialog::create_general_tab() {
    GtkWidget* vbox = gtk_box_new(GTK_ORIENTATION_VERTICAL, 10);
    gtk_container_set_border_width(GTK_CONTAINER(vbox), 10);

    // Autostart checkbox
    autostart_checkbox_.reset(gtk_check_button_new_with_label("Start on system login"));
    gtk_box_pack_start(GTK_BOX(vbox), GTK_WIDGET(autostart_checkbox_.get()), FALSE, FALSE, 0);

    // Theme combo
    GtkWidget* hbox = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 5);
    GtkWidget* label = gtk_label_new("Theme:");
    theme_combo_.reset(gtk_combo_box_text_new());
    gtk_combo_box_text_append_text(GTK_COMBO_BOX_TEXT(theme_combo_.get()), "Auto");
    gtk_combo_box_text_append_text(GTK_COMBO_BOX_TEXT(theme_combo_.get()), "Light");
    gtk_combo_box_text_append_text(GTK_COMBO_BOX_TEXT(theme_combo_.get()), "Dark");
    gtk_box_pack_start(GTK_BOX(hbox), label, FALSE, FALSE, 0);
    gtk_box_pack_start(GTK_BOX(hbox), GTK_WIDGET(theme_combo_.get()), TRUE, TRUE, 0);
    gtk_box_pack_start(GTK_BOX(vbox), hbox, FALSE, FALSE, 0);

    gtk_notebook_append_page(GTK_NOTEBOOK(notebook_.get()), vbox, gtk_label_new("General"));

    return true;
}

bool SettingsDialog::create_behavior_tab() {
    GtkWidget* vbox = gtk_box_new(GTK_ORIENTATION_VERTICAL, 10);
    gtk_container_set_border_width(GTK_CONTAINER(vbox), 10);

    // Auto-hide delay
    GtkWidget* hbox1 = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 5);
    GtkWidget* label1 = gtk_label_new("Auto-hide delay (ms):");
    auto_hide_spin_.reset(gtk_spin_button_new_with_range(1000, 10000, 100));
    gtk_spin_button_set_value(GTK_SPIN_BUTTON(auto_hide_spin_.get()), 3000);
    gtk_box_pack_start(GTK_BOX(hbox1), label1, FALSE, FALSE, 0);
    gtk_box_pack_start(GTK_BOX(hbox1), GTK_WIDGET(auto_hide_spin_.get()), TRUE, TRUE, 0);
    gtk_box_pack_start(GTK_BOX(vbox), hbox1, FALSE, FALSE, 0);

    // Brightness step size
    GtkWidget* hbox2 = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 5);
    GtkWidget* label2 = gtk_label_new("Brightness step size:");
    step_size_spin_.reset(gtk_spin_button_new_with_range(1, 10, 1));
    gtk_spin_button_set_value(GTK_SPIN_BUTTON(step_size_spin_.get()), 1);
    gtk_box_pack_start(GTK_BOX(hbox2), label2, FALSE, FALSE, 0);
    gtk_box_pack_start(GTK_BOX(hbox2), GTK_WIDGET(step_size_spin_.get()), TRUE, TRUE, 0);
    gtk_box_pack_start(GTK_BOX(vbox), hbox2, FALSE, FALSE, 0);

    // Notifications checkbox
    notifications_checkbox_.reset(gtk_check_button_new_with_label("Show notifications"));
    gtk_box_pack_start(GTK_BOX(vbox), GTK_WIDGET(notifications_checkbox_.get()), FALSE, FALSE, 0);

    gtk_notebook_append_page(GTK_NOTEBOOK(notebook_.get()), vbox, gtk_label_new("Behavior"));

    return true;
}

bool SettingsDialog::create_advanced_tab() {
    GtkWidget* vbox = gtk_box_new(GTK_ORIENTATION_VERTICAL, 10);
    gtk_container_set_border_width(GTK_CONTAINER(vbox), 10);

    // Command timeout
    GtkWidget* hbox1 = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 5);
    GtkWidget* label1 = gtk_label_new("Command timeout (ms):");
    timeout_spin_.reset(gtk_spin_button_new_with_range(1000, 10000, 100));
    gtk_spin_button_set_value(GTK_SPIN_BUTTON(timeout_spin_.get()), 5000);
    gtk_box_pack_start(GTK_BOX(hbox1), label1, FALSE, FALSE, 0);
    gtk_box_pack_start(GTK_BOX(hbox1), GTK_WIDGET(timeout_spin_.get()), TRUE, TRUE, 0);
    gtk_box_pack_start(GTK_BOX(vbox), hbox1, FALSE, FALSE, 0);

    // Max retries
    GtkWidget* hbox2 = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 5);
    GtkWidget* label2 = gtk_label_new("Max retries:");
    retries_spin_.reset(gtk_spin_button_new_with_range(1, 5, 1));
    gtk_spin_button_set_value(GTK_SPIN_BUTTON(retries_spin_.get()), 3);
    gtk_box_pack_start(GTK_BOX(hbox2), label2, FALSE, FALSE, 0);
    gtk_box_pack_start(GTK_BOX(hbox2), GTK_WIDGET(retries_spin_.get()), TRUE, TRUE, 0);
    gtk_box_pack_start(GTK_BOX(vbox), hbox2, FALSE, FALSE, 0);

    // Debug mode checkbox
    debug_checkbox_.reset(gtk_check_button_new_with_label("Debug mode"));
    gtk_box_pack_start(GTK_BOX(vbox), GTK_WIDGET(debug_checkbox_.get()), FALSE, FALSE, 0);

    gtk_notebook_append_page(GTK_NOTEBOOK(notebook_.get()), vbox, gtk_label_new("Advanced"));

    return true;
}

void SettingsDialog::on_apply(GtkButton* button, gpointer user_data) {
    SettingsDialog* self = static_cast<SettingsDialog*>(user_data);

    if (self->apply_callback_) {
        self->apply_callback_();
    }
}

void SettingsDialog::on_close(GtkButton* button, gpointer user_data) {
    SettingsDialog* self = static_cast<SettingsDialog*>(user_data);
    self->hide();
}

void SettingsDialog::on_response(GtkDialog* dialog, gint response_id, gpointer user_data) {
    SettingsDialog* self = static_cast<SettingsDialog*>(user_data);

    if (response_id == GTK_RESPONSE_APPLY) {
        if (self->apply_callback_) {
            self->apply_callback_();
        }
    } else {
        self->hide();
    }
}

} // namespace twinkle::ui::widgets
