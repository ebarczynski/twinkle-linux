#pragma once
/// @file ddc_manager.hpp
/// @brief High-level DDC/CI manager — main interface for the UI layer.

#include "twinkle/ddc/command.hpp"
#include "twinkle/ddc/monitor.hpp"
#include "twinkle/ddc/vcp_codes.hpp"
#include <chrono>
#include <map>
#include <memory>
#include <mutex>
#include <string>
#include <unordered_map>
#include <vector>

namespace twinkle::ddc {

/// Main interface for DDC/CI operations.
///
/// Thread-safe: all public methods acquire an internal mutex.
/// The UI layer calls these from GTK signal handlers (main thread)
/// or from g_timeout callbacks.
class DDCManager {
public:
    DDCManager();
    ~DDCManager() = default;

    DDCManager(const DDCManager&) = delete;
    DDCManager& operator=(const DDCManager&) = delete;

    /// Initialize: detect monitors, check ddcutil availability.
    /// Returns true if at least one monitor was found.
    [[nodiscard]] DDCResult<bool> initialize();

    /// Get all detected monitors.
    [[nodiscard]] const std::vector<Monitor>& monitors() const noexcept { return monitors_; }

    /// Find monitor by unique_id.
    [[nodiscard]] const Monitor* find_monitor(std::string_view id) const;

    /// Get brightness for a specific monitor (0-100).
    [[nodiscard]] DDCResult<uint8_t> get_brightness(std::string_view monitor_id);

    /// Set brightness for a specific monitor.
    [[nodiscard]] DDCVoid set_brightness(std::string_view monitor_id, uint16_t value);

    /// Set brightness for ALL monitors.
    [[nodiscard]] DDCVoid set_all_brightness(uint16_t value);

    /// Get VCP value for a monitor.
    [[nodiscard]] DDCResult<uint8_t> get_vcp_value(std::string_view monitor_id, uint8_t code);

    /// Set VCP value for a monitor.
    [[nodiscard]] DDCVoid set_vcp_value(std::string_view monitor_id, uint8_t code, uint16_t value);

    /// Refresh monitor list (re-detect).
    [[nodiscard]] DDCVoid refresh_monitors();

    /// Whether the manager has been initialized.
    [[nodiscard]] bool is_initialized() const noexcept { return initialized_; }

private:
    CommandExecutor executor_;
    std::unique_ptr<MonitorDetector> detector_;
    std::vector<Monitor> monitors_;
    bool initialized_{false};
    mutable std::mutex mutex_;

    // ── Backlight via systemd-logind D-Bus ───────────────────

    /// Set internal display brightness via systemd-logind.
    /// Uses org.freedesktop.login1.Session.SetBrightness
    [[nodiscard]] DDCVoid set_backlight_dbus(std::string_view backlight_name, uint32_t value);

    /// Read current backlight value from sysfs.
    [[nodiscard]] DDCResult<uint8_t> get_backlight_sysfs(std::string_view path);
};

} // namespace twinkle::ddc
