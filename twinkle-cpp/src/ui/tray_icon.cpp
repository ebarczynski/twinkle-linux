/// @file tray_icon.cpp
/// @brief System tray icon using StatusNotifierItem over D-Bus.
///
/// Implements the org.freedesktop.StatusNotifierItem D-Bus interface
/// using sdbus-c++. This is the same approach as ksni in Rust.

#include "twinkle/ui/tray_icon.hpp"
#include "twinkle/core/logger.hpp"
#include <sdbus-c++/sdbus-c++.h>

namespace twinkle::ui {

TrayIcon::TrayIcon(std::shared_ptr<ddc::DDCManager> ddc,
                   std::shared_ptr<core::ConfigManager> cfg,
                   std::function<void(TrayCommand)> on_command)
    : on_command_(std::move(on_command)) {

    try {
        connection_ = sdbus::createSessionBusConnection();

        // Create the SNI object at a unique path
        static int instance_id = 0;
        auto object_path = std::format("/org/freedesktop/StatusNotifierItem/twinkle_{}", instance_id++);

        sni_object_ = sdbus::createObject(*connection_, object_path);

        // Register the StatusNotifierItem interface
        sni_object_->registerMethod("Activate")
            .onInterface("org.freedesktop.StatusNotifierItem")
            .implementedAs([this](int32_t x, int32_t y) {
                if (on_command_) on_command_(TrayCommand::ShowBrightness);
            });

        sni_object_->registerMethod("ContextMenu")
            .onInterface("org.freedesktop.StatusNotifierItem")
            .implementedAs([this](int32_t x, int32_t y) {
                // Context menu is provided via D-Bus menu or just activate
                if (on_command_) on_command_(TrayCommand::ShowBrightness);
            });

        sni_object_->registerMethod("SecondaryActivate")
            .onInterface("org.freedesktop.StatusNotifierItem")
            .implementedAs([this](int32_t x, int32_t y) {
                if (on_command_) on_command_(TrayCommand::ShowBrightness);
            });

        sni_object_->registerMethod("Scroll")
            .onInterface("org.freedesktop.StatusNotifierItem")
            .implementedAs([](int32_t delta, const std::string& orientation) {});

        // Properties
        sni_object_->registerProperty("Id")
            .onInterface("org.freedesktop.StatusNotifierItem")
            .withGetter([]() { return std::string{"twinkle-linux"}; });

        sni_object_->registerProperty("Title")
            .onInterface("org.freedesktop.StatusNotifierItem")
            .withGetter([]() { return std::string{"Twinkle Linux"}; });

        sni_object_->registerProperty("Status")
            .onInterface("org.freedesktop.StatusNotifierItem")
            .withGetter([]() { return std::string{"Active"}; });

        sni_object_->registerProperty("IconName")
            .onInterface("org.freedesktop.StatusNotifierItem")
            .withGetter([]() { return std::string{"display-brightness"}; });

        sni_object_->registerProperty("IconThemePath")
            .onInterface("org.freedesktop.StatusNotifierItem")
            .withGetter([]() { return std::string{""}; });

        sni_object_->registerProperty("ToolTip")
            .onInterface("org.freedesktop.StatusNotifierItem")
            .withGetter([]() -> sdbus::Struct<std::string, std::vector<std::string>, std::string, std::string> {
                return sdbus::make_struct(
                    std::string{"Twinkle Linux"},
                    std::vector<std::string>{},
                    std::string{"Monitor brightness control"},
                    std::string{}
                );
            });

        sni_object_->registerProperty("Category")
            .onInterface("org.freedesktop.StatusNotifierItem")
            .withGetter([]() { return std::string{"Hardware"}; });

        sni_object_->registerProperty("ItemIsMenu")
            .onInterface("org.freedesktop.StatusNotifierItem")
            .withGetter([]() { return false; });

        sni_object_->finishRegistration();

        // Register with the StatusNotifierWatcher
        auto watcher_proxy = sdbus::createProxy(
            *connection_,
            "org.freedesktop.StatusNotifierWatcher",
            "/StatusNotifierWatcher");

        auto service_name = connection_->getUniqueName();
        watcher_proxy->callMethod("RegisterStatusNotifierItem")
            .onInterface("org.freedesktop.StatusNotifierWatcher")
            .withArguments(service_name + object_path);

        // Process D-Bus events in a background thread
        connection_->enterEventLoopAsync();

        LOG_INFO("SNI tray icon registered on D-Bus");

    } catch (const sdbus::Error& e) {
        LOG_ERROR("Failed to create SNI tray: {}", e.what());
    }
}

TrayIcon::~TrayIcon() {
    if (connection_) {
        connection_->leaveEventLoop();
    }
}

} // namespace twinkle::ui
