#include "twinkle/core/logger.hpp"
#include "twinkle/core/config_manager.hpp"
#include "twinkle/ddc/ddc_manager.hpp"
#include "twinkle/ddc/error.hpp"
#include "twinkle/ui/tray_icon.hpp"
#include "twinkle/ui/brightness_popup.hpp"
#include <gtk/gtk.h>
#include <csignal>
#include <iostream>
#include <memory>

namespace {

std::unique_ptr<twinkle::core::ConfigManager> g_config;
std::unique_ptr<twinkle::ddc::DDCManager> g_ddc_manager;
std::unique_ptr<twinkle::ui::TrayIcon> g_tray_icon;
std::unique_ptr<twinkle::ui::BrightnessPopup> g_brightness_popup;

// Signal handler for graceful shutdown
void signal_handler(int signal) {
    LOG_INFO("Received signal {}, shutting down...", signal);
    gtk_main_quit();
}

// Initialize application
bool initialize_application() {
    // Initialize logger
    auto& logger = twinkle::core::Logger::instance();
    logger.initialize("/tmp/twinkle-linux.log", twinkle::core::LogLevel::Info);

    LOG_INFO("Twinkle Linux starting...");

    // Initialize GTK
    if (!gtk_init_check(nullptr, nullptr)) {
        LOG_ERROR("Failed to initialize GTK");
        return false;
    }

    // Load configuration
    g_config = std::make_unique<twinkle::core::ConfigManager>();
    if (auto result = g_config->load(); !result.has_value()) {
        LOG_WARNING("Failed to load config: {}. Using defaults.", result.error());
        g_config->reset_to_defaults();
    }

    // Initialize DDC manager
    g_ddc_manager = std::make_unique<twinkle::ddc::DDCManager>();
    if (auto result = g_ddc_manager->initialize(); !result.has_value()) {
        LOG_ERROR("Failed to initialize DDC manager: {}", to_string(result.error()));
        return false;
    }

    // Initialize tray icon
    g_tray_icon = std::make_unique<twinkle::ui::TrayIcon>();
    if (!g_tray_icon->initialize()) {
        LOG_ERROR("Failed to initialize tray icon");
        return false;
    }

    // Initialize brightness popup
    g_brightness_popup = std::make_unique<twinkle::ui::BrightnessPopup>();
    if (!g_brightness_popup->initialize()) {
        LOG_ERROR("Failed to initialize brightness popup");
        return false;
    }

    // Connect callbacks
    g_tray_icon->set_brightness_callback([]() {
        g_brightness_popup->show();
    });

    g_tray_icon->set_quit_callback([]() {
        LOG_INFO("Quit requested");
        gtk_main_quit();
    });

    g_brightness_popup->set_brightness_callback([](uint8_t brightness) {
        const auto& monitors = g_ddc_manager->monitors();
        if (!monitors.empty()) {
            if (auto result = g_ddc_manager->set_brightness(monitors[0].bus(), brightness);
                !result.has_value()) {
                LOG_ERROR("Failed to set brightness: {}", to_string(result.error()));
            }
        }
    });

    // Show tray icon
    g_tray_icon->show();

    LOG_INFO("Application initialized successfully");
    return true;
}

// Cleanup application
void cleanup_application() {
    LOG_INFO("Cleaning up...");

    g_brightness_popup.reset();
    g_tray_icon.reset();
    g_ddc_manager.reset();

    // Save configuration
    if (g_config) {
        if (auto result = g_config->save(); !result.has_value()) {
            LOG_ERROR("Failed to save config: {}", result.error());
        }
    }

    LOG_INFO("Twinkle Linux shut down");
}

} // anonymous namespace

int main(int argc, char* argv[]) {
    // Parse command line arguments
    for (int i = 1; i < argc; ++i) {
        std::string_view arg(argv[i]);
        if (arg == "--version" || arg == "-v") {
            std::cout << "Twinkle Linux v1.0.0\n";
            return 0;
        } else if (arg == "--help" || arg == "-h") {
            std::cout << "Twinkle Linux - GUI application for controlling external monitor brightness\n"
                      << "\nUsage: twinkle-linux [OPTIONS]\n"
                      << "\nOptions:\n"
                      << "  -h, --help     Show this help message\n"
                      << "  -v, --version  Show version information\n";
            return 0;
        }
    }

    // Setup signal handlers
    std::signal(SIGINT, signal_handler);
    std::signal(SIGTERM, signal_handler);

    // Initialize application
    if (!initialize_application()) {
        std::cerr << "Failed to initialize application\n";
        return 1;
    }

    // Run GTK main loop
    gtk_main();

    // Cleanup
    cleanup_application();

    return 0;
}
