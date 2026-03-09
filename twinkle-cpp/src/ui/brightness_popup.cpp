#include "twinkle/ui/brightness_popup.hpp"
#include <gtk/gtk.h>

namespace twinkle::ui {

BrightnessPopup::BrightnessPopup()
    : window_(nullptr, g_object_unref),
      brightness_slider_(nullptr, g_object_unref),
      monitor_selector_(nullptr, g_object_unref),
      status_label_(nullptr, g_object_unref) {}

BrightnessPopup::~BrightnessPopup() {
    hide();
}

bool BrightnessPopup::initialize() {
    return create_window();
}

void BrightnessPopup::show() {
    if (window_) {
        gtk_window_present(GTK_WINDOW(window_.get()));
    }
}

void BrightnessPopup::hide() {
    if (window_) {
        gtk_widget_hide(GTK_WIDGET(window_.get()));
    }
}

void BrightnessPopup::set_brightness(uint8_t brightness) {
    current_brightness_ = brightness;
    if (brightness_slider_) {
        gtk_range_set_value(GTK_RANGE(brightness_slider_.get()), brightness);
    }
}

uint8_t BrightnessPopup::get_brightness() const noexcept {
    return current_brightness_;
}

void BrightnessPopup::set_monitors(const std::vector<std::string>& monitors) {
    if (!monitor_selector_) {
        return;
    }

    // Clear existing items
    gtk_combo_box_text_remove_all(GTK_COMBO_BOX_TEXT(monitor_selector_.get()));

    // Add monitors
    for (const auto& monitor : monitors) {
        gtk_combo_box_text_append_text(GTK_COMBO_BOX_TEXT(monitor_selector_.get()),
                                      monitor.c_str());
    }

    // Show/hide monitor selector
    set_monitor_selector_visible(monitors.size() > 1);
}

void BrightnessPopup::set_selected_monitor(const std::string& monitor_id) {
    if (!monitor_selector_) {
        return;
    }

    GtkTreeModel* model = gtk_combo_box_get_model(GTK_COMBO_BOX(monitor_selector_.get()));
    GtkTreeIter iter;
    gboolean valid = gtk_tree_model_get_iter_first(model, &iter);

    while (valid) {
        gchar* text;
        gtk_tree_model_get(model, &iter, 0, &text, -1);

        if (text && monitor_id == text) {
            gtk_combo_box_set_active_iter(GTK_COMBO_BOX(monitor_selector_.get()), &iter);
            g_free(text);
            break;
        }

        g_free(text);
        valid = gtk_tree_model_iter_next(model, &iter);
    }
}

void BrightnessPopup::set_monitor_selector_visible(bool visible) {
    monitor_selector_visible_ = visible;
    if (monitor_selector_) {
        gtk_widget_set_visible(GTK_WIDGET(monitor_selector_.get()), visible);
    }
}

bool BrightnessPopup::create_window() {
    // Create popup window
    window_.reset(gtk_window_new(GTK_WINDOW_POPUP));
    gtk_window_set_title(GTK_WINDOW(window_.get()), "Brightness Control");
    gtk_window_set_resizable(GTK_WINDOW(window_.get()), FALSE);
    gtk_window_set_decorated(GTK_WINDOW(window_.get()), FALSE);
    gtk_window_set_skip_taskbar_hint(GTK_WINDOW(window_.get()), TRUE);
    gtk_window_set_skip_pager_hint(GTK_WINDOW(window_.get()), TRUE);

    // Create main container
    GtkWidget* vbox = gtk_box_new(GTK_ORIENTATION_VERTICAL, 5);
    gtk_container_add(GTK_CONTAINER(window_.get()), vbox);

    // Create components
    if (!create_brightness_slider() ||
        !create_monitor_selector() ||
        !create_preset_buttons() ||
        !create_status_label()) {
        return false;
    }

    // Add components to container
    gtk_box_pack_start(GTK_BOX(vbox), GTK_WIDGET(monitor_selector_.get()), FALSE, FALSE, 0);
    gtk_box_pack_start(GTK_BOX(vbox), GTK_WIDGET(brightness_slider_.get()), FALSE, FALSE, 0);
    gtk_box_pack_start(GTK_BOX(vbox), GTK_WIDGET(status_label_.get()), FALSE, FALSE, 0);

    // Add preset buttons
    GtkWidget* hbox = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 5);
    gtk_box_pack_start(GTK_BOX(vbox), hbox, FALSE, FALSE, 0);

    for (auto& button : preset_buttons_) {
        gtk_box_pack_start(GTK_BOX(hbox), button.get(), TRUE, TRUE, 0);
    }

    gtk_widget_show_all(vbox);

    // Connect signals
    g_signal_connect(window_.get(), "delete-event",
                   G_CALLBACK(on_window_hide), this);

    return true;
}

bool BrightnessPopup::create_brightness_slider() {
    brightness_slider_.reset(gtk_scale_new_with_range(GTK_ORIENTATION_HORIZONTAL,
                                                    0.0, 100.0, 1.0));
    gtk_range_set_value(GTK_RANGE(brightness_slider_.get()), current_brightness_);
    gtk_widget_set_size_request(GTK_WIDGET(brightness_slider_.get()), 200, -1);

    // Add marks
    gtk_scale_add_mark(GTK_SCALE(brightness_slider_.get()), 25, GTK_POS_BOTTOM, "25%");
    gtk_scale_add_mark(GTK_SCALE(brightness_slider_.get()), 50, GTK_POS_BOTTOM, "50%");
    gtk_scale_add_mark(GTK_SCALE(brightness_slider_.get()), 75, GTK_POS_BOTTOM, "75%");

    g_signal_connect(brightness_slider_.get(), "value-changed",
                   G_CALLBACK(on_brightness_changed), this);

    return true;
}

bool BrightnessPopup::create_monitor_selector() {
    monitor_selector_.reset(gtk_combo_box_text_new());
    gtk_widget_set_halign(GTK_WIDGET(monitor_selector_.get()), GTK_ALIGN_START);
    gtk_widget_set_visible(GTK_WIDGET(monitor_selector_.get()), monitor_selector_visible_);

    g_signal_connect(monitor_selector_.get(), "changed",
                   G_CALLBACK(on_monitor_changed), this);

    return true;
}

bool BrightnessPopup::create_preset_buttons() {
    const uint8_t presets[] = {25, 50, 75, 100};
    const char* labels[] = {"25%", "50%", "75%", "100%"};

    for (size_t i = 0; i < 4; ++i) {
        GtkWidget* button = gtk_button_new_with_label(labels[i]);
        g_signal_connect(button, "clicked",
                       G_CALLBACK(on_preset_click),
                       GUINT_TO_POINTER(presets[i]));
        preset_buttons_.emplace_back(button, g_object_unref);
    }

    return true;
}

bool BrightnessPopup::create_status_label() {
    status_label_.reset(gtk_label_new(""));
    gtk_widget_set_halign(GTK_WIDGET(status_label_.get()), GTK_ALIGN_CENTER);
    return true;
}

void BrightnessPopup::on_brightness_changed(GtkRange* range, gpointer user_data) {
    BrightnessPopup* self = static_cast<BrightnessPopup*>(user_data);
    double value = gtk_range_get_value(range);
    self->current_brightness_ = static_cast<uint8_t>(value);

    if (self->brightness_callback_) {
        self->brightness_callback_(self->current_brightness_);
    }
}

void BrightnessPopup::on_monitor_changed(GtkComboBox* combo, gpointer user_data) {
    BrightnessPopup* self = static_cast<BrightnessPopup*>(user_data);

    if (self->monitor_callback_) {
        gchar* text = gtk_combo_box_text_get_active_text(GTK_COMBO_BOX_TEXT(combo));
        if (text) {
            self->monitor_callback_(text);
            g_free(text);
        }
    }
}

void BrightnessPopup::on_preset_click(GtkButton* button, gpointer user_data) {
    BrightnessPopup* self = static_cast<BrightnessPopup*>(
        g_object_get_data(G_OBJECT(button), "self"));

    if (self) {
        uint8_t value = static_cast<uint8_t>(GPOINTER_TO_UINT(user_data));
        self->set_brightness(value);

        if (self->brightness_callback_) {
            self->brightness_callback_(value);
        }
    }
}

gboolean BrightnessPopup::on_window_hide(GtkWidget* widget, GdkEvent* event, gpointer user_data) {
    BrightnessPopup* self = static_cast<BrightnessPopup*>(user_data);
    gtk_widget_hide(widget);
    return TRUE;
}

} // namespace twinkle::ui
