/// @file main.cpp
/// @brief Twinkle Linux C++26 — GTK4 monitor brightness control application.
///
/// Entry point. Mirrors the Rust implementation's architecture:
/// 1. Create GTK Application (unique instance via app ID)
/// 2. On activate: create hidden parent window, DDCManager, ConfigManager
/// 3. Build BrightnessPopup (card-based per-monitor sliders)
/// 4. Create TrayIcon (SNI over D-Bus)
/// 5. Process tray commands via callback

#include "twinkle/ddc/ddc_manager.hpp"
#include "twinkle/ddc/monitor.hpp"
#include "twinkle/core/config.hpp"
#include "twinkle/core/logger.hpp"
#include "twinkle/ui/brightness_popup.hpp"
#include "twinkle/ui/tray_icon.hpp"

#include <gtk/gtk.h>
#include <memory>
#include <functional>

using namespace twinkle;

/// Application state — shared between components.
struct AppState {
    std::shared_ptr<ddc::DDCManager> ddc_manager;
    std::shared_ptr<core::ConfigManager> config_manager;
    std::unique_ptr<ui::BrightnessPopup> popup;
    std::unique_ptr<ui::TrayIcon> tray;
    GtkWindow* parent_window{nullptr};
    bool initialized{false};
};

/// Handle tray commands (runs on main thread via g_idle_add).
static void handle_tray_command(AppState* state, ui::TrayCommand cmd) {
    switch (cmd) {
        case ui::TrayCommand::ShowBrightness:
            if (state->popup) state->popup->popup();
            break;
        case ui::TrayCommand::SetAllBrightness10:
            state->ddc_manager->set_all_brightness(10);
            if (state->popup) state->popup->popup(); // refresh
            break;
        case ui::TrayCommand::SetAllBrightness20:
            state->ddc_manager->set_all_brightness(20);
            if (state->popup) state->popup->popup();
            break;
        case ui::TrayCommand::SetAllBrightness40:
            state->ddc_manager->set_all_brightness(40);
            if (state->popup) state->popup->popup();
            break;
        case ui::TrayCommand::SetAllBrightness60:
            state->ddc_manager->set_all_brightness(60);
            if (state->popup) state->popup->popup();
            break;
        case ui::TrayCommand::SetAllBrightness80:
            state->ddc_manager->set_all_brightness(80);
            if (state->popup) state->popup->popup();
            break;
        case ui::TrayCommand::SetAllBrightness100:
            state->ddc_manager->set_all_brightness(100);
            if (state->popup) state->popup->popup();
            break;
        case ui::TrayCommand::ShowSettings:
            LOG_INFO("Show settings requested");
            break;
        case ui::TrayCommand::ShowAbout:
            LOG_INFO("Show about requested");
            break;
        case ui::TrayCommand::Quit:
            g_application_quit(G_APPLICATION(gtk_window_get_application(state->parent_window)));
            break;
    }
}

/// Application activate callback — builds all UI.
static void on_activate(GtkApplication* app, gpointer user_data) {
    auto* state = static_cast<AppState*>(user_data);
    LOG_INFO("Application activated");

    // Create hidden parent window for popover/dialog parenting
    auto* window = gtk_application_window_new(app);
    gtk_window_set_title(GTK_WINDOW(window), "Twinkle Linux");
    gtk_window_set_default_size(GTK_WINDOW(window), 1, 1);
    gtk_window_set_decorated(GTK_WINDOW(window), FALSE);
    state->parent_window = GTK_WINDOW(window);

    // Initialize ConfigManager
    state->config_manager = std::make_shared<core::ConfigManager>();
    if (auto result = state->config_manager->load(); !result) {
        LOG_WARN("Failed to load config, using defaults");
    }

    // Initialize DDCManager
    state->ddc_manager = std::make_shared<ddc::DDCManager>();
    auto init_result = state->ddc_manager->initialize();
    if (!init_result) {
        LOG_ERROR("DDC init error");
    } else if (!*init_result) {
        LOG_WARN("No monitors detected");
    } else {
        LOG_INFO("DDC initialized successfully");
    }

    // Create BrightnessPopup
    state->popup = std::make_unique<ui::BrightnessPopup>(
        state->parent_window, state->ddc_manager, state->config_manager);
    LOG_INFO("BrightnessPopup created");

    // Create TrayIcon with command callback
    auto* state_ptr = state;
    state->tray = std::make_unique<ui::TrayIcon>(
        state->ddc_manager,
        state->config_manager,
        [state_ptr](ui::TrayCommand cmd) {
            // Dispatch to main thread via g_idle_add
            auto* cmd_ptr = new ui::TrayCommand(cmd);
            g_idle_add_full(G_PRIORITY_HIGH_IDLE,
                [](gpointer data) -> gboolean {
                    auto* c = static_cast<std::pair<AppState*, ui::TrayCommand>*>(data);
                    handle_tray_command(c->first, c->second);
                    delete c;
                    return G_SOURCE_REMOVE;
                },
                new std::pair{state_ptr, cmd},
                [](gpointer data) { delete static_cast<std::pair<AppState*, ui::TrayCommand>*>(data); });
        });
    LOG_INFO("TrayIcon created");

    state->initialized = true;
    LOG_INFO("Initialization complete");
}

int main(int argc, char* argv[]) {
    core::Logger::instance().initialize("", core::LogLevel::Info);
    LOG_INFO("Starting Twinkle Linux C++ v0.1.0");

    // Create GTK Application with unique ID (single instance)
    auto* app = gtk_application_new(
        "com.github.ebarczynski.TwinkleLinux",
        G_APPLICATION_FLAGS_NONE);

    AppState state;

    g_signal_connect(app, "activate", G_CALLBACK(on_activate), &state);

    auto status = g_application_run(G_APPLICATION(app), argc, argv);
    g_object_unref(app);

    LOG_INFO("Twinkle Linux exited with code {}", status);
    return status;
}
