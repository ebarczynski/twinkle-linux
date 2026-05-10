/// @file main.cpp
/// @brief Twinkle Linux C++26 — GTK4 monitor brightness control application.

#include "twinkle/ddc/ddc_manager.hpp"
#include "twinkle/ddc/monitor.hpp"
#include "twinkle/core/config.hpp"
#include "twinkle/core/logger.hpp"
#include "twinkle/ui/brightness_popup.hpp"
#include "twinkle/ui/tray_icon.hpp"

#include <gtk/gtk.h>
#include <memory>
#include <utility>

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

/// Safely call set_all_brightness, discarding the result (fire-and-forget UI action).
static void safe_set_all(AppState* state, uint16_t value) {
    (void)state->ddc_manager->set_all_brightness(value);
}

/// Handle tray commands.
static void handle_tray_command(AppState* state, ui::TrayCommand cmd) {
    switch (cmd) {
        case ui::TrayCommand::ShowBrightness:
            if (state->popup) state->popup->popup();
            break;
        case ui::TrayCommand::SetAllBrightness10:
            safe_set_all(state, 10);
            if (state->popup) state->popup->popup();
            break;
        case ui::TrayCommand::SetAllBrightness20:
            safe_set_all(state, 20);
            if (state->popup) state->popup->popup();
            break;
        case ui::TrayCommand::SetAllBrightness40:
            safe_set_all(state, 40);
            if (state->popup) state->popup->popup();
            break;
        case ui::TrayCommand::SetAllBrightness60:
            safe_set_all(state, 60);
            if (state->popup) state->popup->popup();
            break;
        case ui::TrayCommand::SetAllBrightness80:
            safe_set_all(state, 80);
            if (state->popup) state->popup->popup();
            break;
        case ui::TrayCommand::SetAllBrightness100:
            safe_set_all(state, 100);
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

/// Application activate callback.
static void on_activate(GtkApplication* app, gpointer user_data) {
    auto* state = static_cast<AppState*>(user_data);
    LOG_INFO("Application activated");

    auto* window = gtk_application_window_new(app);
    gtk_window_set_title(GTK_WINDOW(window), "Twinkle Linux");
    gtk_window_set_default_size(GTK_WINDOW(window), 1, 1);
    gtk_window_set_decorated(GTK_WINDOW(window), FALSE);
    state->parent_window = GTK_WINDOW(window);

    state->config_manager = std::make_shared<core::ConfigManager>();
    if (auto result = state->config_manager->load(); !result) {
        LOG_WARN("Failed to load config, using defaults");
    }

    state->ddc_manager = std::make_shared<ddc::DDCManager>();
    auto init_result = state->ddc_manager->initialize();
    if (!init_result) {
        LOG_ERROR("DDC init error");
    } else if (!*init_result) {
        LOG_WARN("No monitors detected");
    } else {
        LOG_INFO("DDC initialized successfully");
    }

    state->popup = std::make_unique<ui::BrightnessPopup>(
        state->parent_window, state->ddc_manager, state->config_manager);

    auto* state_ptr = state;
    state->tray = std::make_unique<ui::TrayIcon>(
        state->ddc_manager,
        state->config_manager,
        [state_ptr](ui::TrayCommand cmd) {
            // Dispatch to main thread
            auto* payload = new std::pair<AppState*, ui::TrayCommand>(state_ptr, cmd);
            g_idle_add_full(G_PRIORITY_HIGH_IDLE,
                [](gpointer data) -> gboolean {
                    auto* p = static_cast<std::pair<AppState*, ui::TrayCommand>*>(data);
                    handle_tray_command(p->first, p->second);
                    delete p;
                    return G_SOURCE_REMOVE;
                },
                payload,
                [](gpointer data) { delete static_cast<std::pair<AppState*, ui::TrayCommand>*>(data); });
        });

    state->initialized = true;
    LOG_INFO("Initialization complete");
}

int main(int argc, char* argv[]) {
    core::Logger::instance().initialize("", core::LogLevel::Info);
    LOG_INFO("Starting Twinkle Linux C++ v0.1.0");

    auto* app = gtk_application_new(
        "com.github.ebarczynski.TwinkleLinux",
        G_APPLICATION_DEFAULT_FLAGS);

    AppState state;
    g_signal_connect(app, "activate", G_CALLBACK(on_activate), &state);

    auto status = g_application_run(G_APPLICATION(app), argc, argv);
    g_object_unref(app);

    LOG_INFO("Twinkle Linux exited with code {}", status);
    return status;
}
