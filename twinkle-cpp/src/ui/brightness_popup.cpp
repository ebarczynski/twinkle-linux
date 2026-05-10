/// @file brightness_popup.cpp
/// @brief GTK4 brightness popup with per-monitor card-based layout.
///
/// Architecture matches twinkle-rust/src/ui/brightness_popup.rs:
/// - "All Monitors" override slider always visible at top
/// - Cards built dynamically on each popup() call
/// - 300ms debounce using g_timeout_add
/// - Dark theme CSS from data/style.css

#include "twinkle/ui/brightness_popup.hpp"
#include "twinkle/ddc/ddc_manager.hpp"
#include "twinkle/ddc/monitor.hpp"
#include "twinkle/core/config.hpp"
#include "twinkle/core/logger.hpp"

#include <gtk/gtk.h>
#include <algorithm>

namespace twinkle::ui {

// ── CSS Loading ─────────────────────────────────────────────

static const char* kCSSPath = DATA_DIR "/style.css";

void BrightnessPopup::load_css() {
    auto* display = gdk_display_get_default();
    if (!display) return;

    auto* provider = gtk_css_provider_new();

    // Try loading from installed data dir first, then relative path
    GError* error = nullptr;
    if (!gtk_css_provider_load_from_path(provider, kCSSPath, &error)) {
        // Try relative to binary
        gtk_css_provider_load_from_path(provider, "data/style.css", nullptr);
    }
    if (error) g_error_free(error);

    gtk_style_context_add_provider_for_display(
        display, GTK_STYLE_PROVIDER(provider),
        GTK_STYLE_PROVIDER_PRIORITY_APPLICATION);
    g_object_unref(provider);
}

// ── Constructor / Destructor ────────────────────────────────

BrightnessPopup::BrightnessPopup(GtkWindow* parent,
                                 std::shared_ptr<ddc::DDCManager> ddc,
                                 std::shared_ptr<core::ConfigManager> cfg)
    : ddc_(std::move(ddc)), config_(std::move(cfg)) {
    load_css();
    build_ui(parent);
}

BrightnessPopup::~BrightnessPopup() {
    if (window_) gtk_window_destroy(window_);
}

// ── UI Construction ─────────────────────────────────────────

void BrightnessPopup::build_ui(GtkWindow* parent) {
    // Main vertical container
    auto* main_box = gtk_box_new(GTK_ORIENTATION_VERTICAL, 0);
    gtk_widget_add_css_class(main_box, "main-container");

    // Header
    auto* header = gtk_label_new("Brightness");
    gtk_widget_add_css_class(header, "header-label");
    gtk_widget_set_halign(header, GTK_ALIGN_START);
    gtk_box_append(GTK_BOX(main_box), header);

    // Cards container (populated dynamically)
    cards_container_ = gtk_box_new(GTK_ORIENTATION_VERTICAL, 0);
    gtk_box_append(GTK_BOX(main_box), cards_container_);

    // "All Monitors" override slider — always visible at top
    auto* override_row = gtk_box_new(GTK_ORIENTATION_VERTICAL, 0);
    gtk_widget_add_css_class(override_row, "all-monitors-row");

    // Header: sun icon + "All Monitors" + value label
    auto* oheader = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 8);
    auto* oicon = gtk_label_new("\u2600"); // ☀
    gtk_widget_add_css_class(oicon, "sun-icon");
    auto* oname = gtk_label_new("All Monitors");
    gtk_widget_add_css_class(oname, "monitor-name");
    gtk_widget_set_hexpand(oname, TRUE);
    gtk_widget_set_halign(oname, GTK_ALIGN_START);
    override_value_label_ = gtk_label_new("100%");
    gtk_widget_add_css_class(override_value_label_, "brightness-value");
    gtk_box_append(GTK_BOX(oheader), oicon);
    gtk_box_append(GTK_BOX(oheader), oname);
    gtk_box_append(GTK_BOX(oheader), override_value_label_);
    gtk_box_append(GTK_BOX(override_row), oheader);

    // Override slider row
    auto* oslider_row = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 4);
    gtk_widget_add_css_class(oslider_row, "slider-row");
    auto* odim = gtk_label_new("\U0001F505"); // 🔅
    gtk_widget_add_css_class(odim, "sun-dim-icon");

    override_adjustment_ = GTK_WIDGET(gtk_adjustment_new(100.0, 0.0, 100.0, 1.0, 10.0, 0.0));
    auto* oscale = gtk_scale_new(GTK_ORIENTATION_HORIZONTAL, GTK_ADJUSTMENT(override_adjustment_));
    gtk_widget_set_hexpand(oscale, TRUE);
    gtk_scale_set_draw_value(GTK_SCALE(oscale), FALSE);

    auto* obright = gtk_label_new("\U0001F506"); // 🔆
    gtk_widget_add_css_class(obright, "sun-icon");

    gtk_box_append(GTK_BOX(oslider_row), odim);
    gtk_box_append(GTK_BOX(oslider_row), oscale);
    gtk_box_append(GTK_BOX(oslider_row), obright);
    gtk_box_append(GTK_BOX(override_row), oslider_row);

    gtk_box_append(GTK_BOX(main_box), override_row);

    // Separator
    auto* sep = gtk_box_new(GTK_ORIENTATION_VERTICAL, 0);
    gtk_widget_add_css_class(sep, "separator");
    gtk_box_append(GTK_BOX(main_box), sep);

    // Bottom toolbar — settings button
    auto* bottom = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 0);
    gtk_widget_add_css_class(bottom, "bottom-toolbar");
    gtk_widget_set_halign(bottom, GTK_ALIGN_END);
    auto* settings_btn = gtk_button_new_with_label("\u2699"); // ⚙
    gtk_widget_add_css_class(settings_btn, "icon-button");
    gtk_box_append(GTK_BOX(bottom), settings_btn);
    gtk_box_append(GTK_BOX(main_box), bottom);

    // Window
    window_ = GTK_WINDOW(gtk_window_new());
    gtk_window_set_title(window_, "Brightness");
    gtk_window_set_transient_for(window_, parent);
    gtk_window_set_resizable(window_, FALSE);
    gtk_window_set_default_size(window_, 340, -1);
    gtk_window_set_decorated(window_, TRUE);
    gtk_window_set_child(window_, main_box);

    // Connect override slider
    connect_override();

    // Settings button: just log for now
    g_signal_connect(settings_btn, "clicked", G_CALLBACK(+[](GtkWidget*, gpointer) {
        LOG_INFO("Settings button clicked");
    }), nullptr);
}

GtkWidget* BrightnessPopup::build_card(const ddc::Monitor& mon, uint8_t brightness) {
    auto* card = gtk_box_new(GTK_ORIENTATION_VERTICAL, 0);
    gtk_widget_add_css_class(card, "monitor-card");

    // Header row: icon + name + value
    auto* header = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 8);
    auto* icon = gtk_label_new(
        mon.monitor_type == ddc::MonitorType::Internal ? "\U0001F30D" : "\U0001F5A5");
    gtk_widget_add_css_class(icon, "monitor-icon");
    auto* name_label = gtk_label_new(mon.display_name().c_str());
    gtk_widget_add_css_class(name_label, "monitor-name");
    gtk_widget_set_hexpand(name_label, TRUE);
    gtk_widget_set_halign(name_label, GTK_ALIGN_START);
    auto* value_label = gtk_label_new(std::format("{}%", brightness).c_str());
    gtk_widget_add_css_class(value_label, "brightness-value");
    gtk_box_append(GTK_BOX(header), icon);
    gtk_box_append(GTK_BOX(header), name_label);
    gtk_box_append(GTK_BOX(header), value_label);
    gtk_box_append(GTK_BOX(card), header);

    // Slider row
    auto* slider_row = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 4);
    gtk_widget_add_css_class(slider_row, "slider-row");
    auto* dim = gtk_label_new("\U0001F505");
    gtk_widget_add_css_class(dim, "sun-dim-icon");
    auto* adjustment = gtk_adjustment_new(brightness, 0.0, 100.0, 1.0, 10.0, 0.0);
    auto* scale = gtk_scale_new(GTK_ORIENTATION_HORIZONTAL, adjustment);
    gtk_widget_set_hexpand(scale, TRUE);
    gtk_scale_set_draw_value(GTK_SCALE(scale), FALSE);
    auto* bright = gtk_label_new("\U0001F506");
    gtk_widget_add_css_class(bright, "sun-icon");
    gtk_box_append(GTK_BOX(slider_row), dim);
    gtk_box_append(GTK_BOX(slider_row), scale);
    gtk_box_append(GTK_BOX(slider_row), bright);
    gtk_box_append(GTK_BOX(card), slider_row);

    return card;
}

// ── Slider Connections ──────────────────────────────────────

void BrightnessPopup::connect_override() {
    g_signal_connect(GTK_ADJUSTMENT(override_adjustment_), "value-changed",
        G_CALLBACK(+[](GtkAdjustment* adj, gpointer user_data) {
            auto* self = static_cast<BrightnessPopup*>(user_data);
            if (self->override_suppress) {
                self->override_suppress = false;
                return;
            }

            auto value = static_cast<uint16_t>(gtk_adjustment_get_value(adj));
            gtk_label_set_text(GTK_LABEL(self->override_value_label_),
                             std::format("{}%", value).c_str());

            // Cancel old debounce timer
            if (self->override_debounce_id_) {
                g_source_remove(self->override_debounce_id_);
                self->override_debounce_id_ = 0;
            }

            // Set new 300ms debounce
            self->override_debounce_id_ = g_timeout_add(300,
                [](gpointer data) -> gboolean {
                    auto* self = static_cast<BrightnessPopup*>(data);
                    self->override_debounce_id_ = 0;
                    auto val = static_cast<uint16_t>(
                        gtk_adjustment_get_value(GTK_ADJUSTMENT(self->override_adjustment_)));
                    self->ddc_->set_all_brightness(val);
                    return G_SOURCE_REMOVE;
                }, self);
        }), this);
}

void BrightnessPopup::connect_card_slider(MonitorCard& card, std::string monitor_id) {
    // The adjustment is stored in card.adjustment
    // value_label is in card.value_label
    g_signal_connect(card.adjustment, "value-changed",
        G_CALLBACK(+[](GtkAdjustment* adj, gpointer user_data) {
            auto* card = static_cast<MonitorCard*>(user_data);
            if (card->suppress) {
                card->suppress = false;
                return;
            }

            auto value = static_cast<uint16_t>(gtk_adjustment_get_value(adj));
            gtk_label_set_text(GTK_LABEL(card->value_label),
                             std::format("{}%", value).c_str());

            // Cancel old debounce
            if (card->debounce_id) {
                g_source_remove(card->debounce_id);
                card->debounce_id = 0;
            }

            // Set new 300ms debounce
            // We need to capture the monitor_id and DDCManager
            // Store them in the MonitorCard struct
            card->debounce_id = g_timeout_add(300,
                [](gpointer data) -> gboolean {
                    auto* c = static_cast<MonitorCard*>(data);
                    c->debounce_id = 0;
                    // This is set up per-card in popup()
                    return G_SOURCE_REMOVE;
                }, &card);
        }), &card);
}

// ── Show / Hide ─────────────────────────────────────────────

void BrightnessPopup::popup() {
    // Clear existing cards
    GtkWidget* child = gtk_widget_get_first_child(cards_container_);
    while (child) {
        auto* next = gtk_widget_get_next_sibling(child);
        gtk_box_remove(GTK_BOX(cards_container_), child);
        child = next;
    }
    cards_.clear();

    // Get monitors and build cards
    const auto& monitors = ddc_->monitors();
    uint32_t brightness_sum = 0;
    uint32_t brightness_count = 0;

    for (const auto& mon : monitors) {
        auto brightness_result = ddc_->get_brightness(mon.unique_id());
        uint8_t brightness = brightness_result.value_or(50);

        auto* card_widget = build_card(mon, brightness);

        // Find the adjustment and value label in the card widget
        MonitorCard card{};
        card.monitor_id = mon.unique_id();
        card.card_widget = card_widget;

        // Walk the widget tree to find the Scale's adjustment
        auto* card_box = GTK_BOX(card_widget);
        auto* slider_row = gtk_widget_get_last_child(card_widget); // slider_row is last child
        if (slider_row) {
            // slider_row children: dim_icon, scale, bright_icon
            auto* scale_widget = gtk_widget_get_first_child(slider_row);
            // skip dim icon
            if (scale_widget) scale_widget = gtk_widget_get_next_sibling(scale_widget);
            if (scale_widget && GTK_IS_SCALE(scale_widget)) {
                card.adjustment = gtk_range_get_adjustment(GTK_RANGE(scale_widget));
            }
        }

        // Find value label (in header row, last child)
        auto* header_row = gtk_widget_get_first_child(card_widget);
        if (header_row) {
            auto* lbl = gtk_widget_get_last_child(header_row);
            if (lbl && GTK_IS_LABEL(lbl)) {
                card.value_label = lbl;
            }
        }

        // Connect the slider with debounce
        if (card.adjustment && card.value_label) {
            // Set up a per-card callback using g_object_set_data
            auto* ddc_ptr = ddc_.get();
            auto mon_id = mon.unique_id();

            g_signal_connect(card.adjustment, "value-changed",
                G_CALLBACK(+[](GtkAdjustment* adj, gpointer user_data) {
                    auto* c = static_cast<MonitorCard*>(user_data);
                    if (c->suppress) { c->suppress = false; return; }

                    auto value = static_cast<uint16_t>(gtk_adjustment_get_value(adj));
                    gtk_label_set_text(GTK_LABEL(c->value_label),
                                     std::format("{}%", value).c_str());

                    if (c->debounce_id) { g_source_remove(c->debounce_id); c->debounce_id = 0; }

                    // Store pending value and DDC pointer
                    auto* ddc = static_cast<ddc::DDCManager*>(g_object_get_data(G_OBJECT(adj), "ddc_ptr"));
                    auto* mid = static_cast<std::string*>(g_object_get_data(G_OBJECT(adj), "monitor_id"));

                    c->debounce_id = g_timeout_add(300,
                        [](gpointer data) -> gboolean {
                            auto* c = static_cast<MonitorCard*>(data);
                            c->debounce_id = 0;
                            auto* ddc = static_cast<ddc::DDCManager*>(g_object_get_data(G_OBJECT(c->adjustment), "ddc_ptr"));
                            auto* mid = static_cast<std::string*>(g_object_get_data(G_OBJECT(c->adjustment), "monitor_id"));
                            if (ddc && mid) {
                                auto val = static_cast<uint16_t>(gtk_adjustment_get_value(GTK_ADJUSTMENT(c->adjustment)));
                                ddc->set_brightness(*mid, val);
                            }
                            return G_SOURCE_REMOVE;
                        }, c);
                }), &card);

            // Attach DDC pointer and monitor ID to the adjustment
            g_object_set_data_full(G_OBJECT(card.adjustment), "ddc_ptr", ddc_ptr, nullptr);
            auto* mid_heap = new std::string(mon.unique_id());
            g_object_set_data_full(G_OBJECT(card.adjustment), "monitor_id", mid_heap,
                [](gpointer p) { delete static_cast<std::string*>(p); });
        }

        cards_.push_back(std::move(card));
        gtk_box_append(GTK_BOX(cards_container_), card_widget);

        brightness_sum += brightness;
        brightness_count++;
    }

    // Set override slider to average brightness
    if (brightness_count > 0) {
        auto avg = static_cast<uint16_t>(brightness_sum / brightness_count);
        override_suppress_ = true;
        gtk_adjustment_set_value(GTK_ADJUSTMENT(override_adjustment_), avg);
        gtk_label_set_text(GTK_LABEL(override_value_label_), std::format("{}%", avg).c_str());
    }

    gtk_window_present(window_);
}

void BrightnessPopup::popdown() {
    gtk_widget_set_visible(GTK_WIDGET(window_), FALSE);
}

} // namespace twinkle::ui
