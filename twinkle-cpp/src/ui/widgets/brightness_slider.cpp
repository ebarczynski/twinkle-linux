#include "twinkle/ui/widgets/brightness_slider.hpp"

namespace twinkle::ui::widgets {

BrightnessSlider::BrightnessSlider()
    : slider_(nullptr, g_object_unref) {}

BrightnessSlider::~BrightnessSlider() {
    if (debounce_source_id_) {
        g_source_remove(debounce_source_id_);
    }
}

bool BrightnessSlider::initialize() {
    slider_.reset(gtk_scale_new_with_range(GTK_ORIENTATION_HORIZONTAL,
                                                    0.0, 100.0, 1.0));
    gtk_range_set_value(GTK_RANGE(slider_.get()), current_value_);
    gtk_widget_set_size_request(GTK_WIDGET(slider_.get()), 200, -1);

    // Add marks
    gtk_scale_add_mark(GTK_SCALE(slider_.get()), 25, GTK_POS_BOTTOM, "25%");
    gtk_scale_add_mark(GTK_SCALE(slider_.get()), 50, GTK_POS_BOTTOM, "50%");
    gtk_scale_add_mark(GTK_SCALE(slider_.get()), 75, GTK_POS_BOTTOM, "75%");

    g_signal_connect(slider_.get(), "value-changed",
                   G_CALLBACK(on_value_changed), this);

    return true;
}

void BrightnessSlider::set_value(uint8_t value) {
    current_value_ = value;
    if (slider_) {
        gtk_range_set_value(GTK_RANGE(slider_.get()), value);
    }
}

void BrightnessSlider::set_enabled(bool enabled) {
    if (slider_) {
        gtk_widget_set_sensitive(GTK_WIDGET(slider_.get()), enabled);
    }
}

void BrightnessSlider::on_value_changed(GtkRange* range, gpointer user_data) {
    BrightnessSlider* self = static_cast<BrightnessSlider*>(user_data);

    // Remove existing debounce source
    if (self->debounce_source_id_) {
        g_source_remove(self->debounce_source_id_);
        self->debounce_source_id_ = 0;
    }

    // Schedule new debounce
    self->debounce_source_id_ = g_timeout_add(
        self->debounce_ms_,
        [](gpointer data) -> gboolean {
            BrightnessSlider* slider = static_cast<BrightnessSlider*>(data);
            slider->debounce_source_id_ = 0;

            if (slider->callback_) {
                slider->callback_(slider->current_value_);
            }

            return G_SOURCE_REMOVE;
        },
        self);
}

} // namespace twinkle::ui::widgets
