#pragma once
/// @file brightness_popup.hpp
/// @brief GTK4 brightness popup with card-based per-monitor sliders.

#include <gtk/gtk.h>
#include "twinkle/ddc/monitor.hpp"
#include <cstdint>
#include <memory>
#include <string>
#include <vector>

namespace twinkle::ddc { class DDCManager; }
namespace twinkle::core { class ConfigManager; }

namespace twinkle::ui {

/// A single monitor card with its slider.
struct MonitorCard {
    std::string monitor_id;
    GtkWidget* card_widget{nullptr};
    GtkAdjustment* adjustment{nullptr};
    GtkWidget* value_label{nullptr};
    bool suppress{false};  ///< Skip next value_changed if programmatic
    guint debounce_id{0};  ///< g_timeout source ID
};

/// GTK4 brightness popup window.
class BrightnessPopup {
public:
    BrightnessPopup(GtkWindow* parent,
                    std::shared_ptr<ddc::DDCManager> ddc,
                    std::shared_ptr<core::ConfigManager> cfg);
    ~BrightnessPopup();

    BrightnessPopup(const BrightnessPopup&) = delete;
    BrightnessPopup& operator=(const BrightnessPopup&) = delete;

    /// Show the popup and refresh monitor cards.
    void popup();

    /// Hide the popup.
    void popdown();

    /// Get the underlying GTK window.
    [[nodiscard]] GtkWindow* window() const noexcept { return window_; }

private:
    GtkWindow* window_{nullptr};
    GtkWidget* cards_container_{nullptr};
    GtkWidget* override_adjustment_{nullptr};
    GtkWidget* override_value_label_{nullptr};
    bool override_suppress_{false};
    guint override_debounce_id_{0};

    std::shared_ptr<ddc::DDCManager> ddc_;
    std::shared_ptr<core::ConfigManager> config_;
    std::vector<MonitorCard> cards_;

    /// Build the UI widgets.
    void build_ui(GtkWindow* parent);

    /// Build a single monitor card.
    [[nodiscard]] GtkWidget* build_card(const ddc::Monitor& mon, uint8_t brightness);

    /// Connect the override slider.
    void connect_override();

    /// Connect a per-monitor slider.
    void connect_card_slider(MonitorCard& card, std::string monitor_id);

    /// Load CSS.
    static void load_css();
};

} // namespace twinkle::ui
