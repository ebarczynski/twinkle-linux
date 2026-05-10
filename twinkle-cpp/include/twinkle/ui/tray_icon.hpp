#pragma once
/// @file tray_icon.hpp
/// @brief System tray icon using StatusNotifierItem over D-Bus (sdbus-c++).
///
/// GTK4 does not include GtkStatusIcon (removed). Instead, we implement
/// the freedesktop.org StatusNotifierItem protocol via sdbus-c++.
/// This matches the Rust ksni approach.

#include <sdbus-c++/sdbus-c++.h>
#include <cstdint>
#include <functional>
#include <memory>
#include <string>

namespace twinkle::ddc { class DDCManager; }
namespace twinkle::core { class ConfigManager; }
namespace twinkle::ui { class BrightnessPopup; }

namespace twinkle::ui {

/// Commands from tray menu to the main GTK thread.
enum class TrayCommand {
    ShowBrightness,
    SetAllBrightness10,
    SetAllBrightness20,
    SetAllBrightness40,
    SetAllBrightness60,
    SetAllBrightness80,
    SetAllBrightness100,
    ShowSettings,
    ShowAbout,
    Quit,
};

/// System tray icon using SNI over D-Bus.
class TrayIcon {
public:
    TrayIcon(std::shared_ptr<ddc::DDCManager> ddc,
             std::shared_ptr<core::ConfigManager> cfg,
             std::function<void(TrayCommand)> on_command);
    ~TrayIcon();

    TrayIcon(const TrayIcon&) = delete;
    TrayIcon& operator=(const TrayIcon&) = delete;

private:
    std::unique_ptr<sdbus::IConnection> connection_;
    std::unique_ptr<sdbus::IObject> sni_object_;
    std::function<void(TrayCommand)> on_command_;

    void register_sni();
    void emit_signal(const char* signal_name);
};

} // namespace twinkle::ui
