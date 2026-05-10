/// @file brightness_slider.cpp
/// @brief Debounced brightness slider widget.

#include "twinkle/ui/widgets/brightness_slider.hpp"

namespace twinkle::ui::widgets {

BrightnessSlider::BrightnessSlider() {
    adjustment_ = gtk_adjustment_new(50.0, 0.0, 100.0, 1.0, 5.0, 0.0);

    container_ = gtk_box_new(GTK_ORIENTATION_VERTICAL, 4);
    gtk_widget_set_margin_top(container_, 8);
    gtk_widget_set_margin_bottom(container_, 8);
    gtk_widget_set_margin_start(container_, 8);
    gtk_widget_set_margin_end(container_, 8);

    auto* label = gtk_label_new("Brightness");
    gtk_widget_set_halign(label, GTK_ALIGN_START);
    gtk_box_append(GTK_BOX(container_), label);

    scale_ = gtk_scale_new(GTK_ORIENTATION_HORIZONTAL, adjustment_);
    gtk_widget_set_hexpand(scale_, TRUE);
    gtk_scale_set_draw_value(GTK_SCALE(scale_), TRUE);

    // Add marks
    gtk_scale_add_mark(GTK_SCALE(scale_), 0.0, GTK_POS_BOTTOM, "0");
    gtk_scale_add_mark(GTK_SCALE(scale_), 25.0, GTK_POS_BOTTOM, "25");
    gtk_scale_add_mark(GTK_SCALE(scale_), 50.0, GTK_POS_BOTTOM, "50");
    gtk_scale_add_mark(GTK_SCALE(scale_), 75.0, GTK_POS_BOTTOM, "75");
    gtk_scale_add_mark(GTK_SCALE(scale_), 100.0, GTK_POS_BOTTOM, "100");

    gtk_box_append(GTK_BOX(container_), scale_);

    // Connect value-changed
    g_signal_connect(adjustment_, "value-changed",
        G_CALLBACK(BrightnessSlider::on_value_changed), this);
}

BrightnessSlider::~BrightnessSlider() {
    if (debounce_source_id_) {
        g_source_remove(debounce_source_id_);
    }
}

void BrightnessSlider::set_value(uint8_t value) {
    current_value_ = value;
    suppress_ = true;
    gtk_adjustment_set_value(adjustment_, value);
}

void BrightnessSlider::set_on_change(std::function<void(uint16_t)> callback) {
    callback_ = std::move(callback);
}

void BrightnessSlider::set_enabled(bool enabled) {
    gtk_widget_set_sensitive(scale_, enabled);
}

void BrightnessSlider::on_value_changed(GtkAdjustment* adj, gpointer user_data) {
    auto* self = static_cast<BrightnessSlider*>(user_data);

    if (self->suppress_) {
        self->suppress_ = false;
        return;
    }

    self->current_value_ = static_cast<uint8_t>(gtk_adjustment_get_value(adj));

    // Cancel existing debounce timer
    if (self->debounce_source_id_) {
        g_source_remove(self->debounce_source_id_);
        self->debounce_source_id_ = 0;
    }

    // Set new 300ms debounce timer
    self->debounce_source_id_ = g_timeout_add(300,
        [](gpointer data) -> gboolean {
            auto* s = static_cast<BrightnessSlider*>(data);
            s->debounce_source_id_ = 0;
            if (s->callback_) {
                s->callback_(s->current_value_);
            }
            return G_SOURCE_REMOVE;
        }, self);
}

} // namespace twinkle::ui::widgets
