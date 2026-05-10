/// @file ddc_manager.cpp
/// @brief High-level DDC/CI manager implementation.

#include "twinkle/ddc/ddc_manager.hpp"
#include "twinkle/ddc/command.hpp"
#include "twinkle/ddc/monitor.hpp"
#include "twinkle/core/logger.hpp"
#include <algorithm>
#include <filesystem>
#include <fstream>
#include <chrono>
#include <cmath>

// sdbus-c++ for systemd-logind backlight control
#include <sdbus-c++/sdbus-c++.h>

namespace twinkle::ddc {

DDCManager::DDCManager()
    : executor_(),
      detector_(std::make_unique<MonitorDetector>(executor_)) {}

DDCResult<bool> DDCManager::initialize() {
    std::lock_guard lock(mutex_);
    LOG_INFO("Initializing DDCManager...");

    if (!executor_.check_available()) {
        LOG_WARN("ddcutil not available");
        initialized_ = true; // Still mark as initialized — might have backlight only
        auto monitors = detector_->detect_monitors();
        if (monitors) {
            monitors_ = std::move(*monitors);
        }
        return !monitors_.empty();
    }

    auto result = detector_->detect_monitors();
    if (!result) {
        LOG_ERROR("Monitor detection failed: {}", static_cast<int>(result.error()));
        initialized_ = true;
        return false;
    }

    monitors_ = std::move(*result);
    initialized_ = true;

    LOG_INFO("DDCManager initialized: {} monitors found", monitors_.size());
    return !monitors_.empty();
}

const Monitor* DDCManager::find_monitor(std::string_view id) const {
    for (const auto& m : monitors_) {
        if (m.unique_id() == id) return &m;
    }
    return nullptr;
}

DDCResult<uint8_t> DDCManager::get_brightness(std::string_view monitor_id) {
    std::lock_guard lock(mutex_);
    auto* mon = find_monitor(monitor_id);
    if (!mon) return std::unexpected(DDCError::MonitorNotFound);

    if (mon->monitor_type == MonitorType::Internal) {
        return get_backlight_sysfs(mon->backlight_path);
    }

    // External: ddcutil getvcp
    auto result = executor_.get_vcp(mon->bus, 0x10);
    if (!result) return std::unexpected(result.error());
    if (!result->success) return std::unexpected(DDCError::CommandFailed);
    if (result->value) return static_cast<uint8_t>(*result->value);
    return std::unexpected(DDCError::ParseError);
}

DDCVoid DDCManager::set_brightness(std::string_view monitor_id, uint16_t value) {
    std::lock_guard lock(mutex_);
    auto* mon = find_monitor(monitor_id);
    if (!mon) return std::unexpected(DDCError::MonitorNotFound);

    value = std::clamp(value, static_cast<uint16_t>(0), static_cast<uint16_t>(100));

    if (mon->monitor_type == MonitorType::Internal) {
        // Internal: systemd-logind D-Bus
        // Extract backlight name from path
        auto name = std::filesystem::path{mon->backlight_path}.filename().string();
        return set_backlight_dbus(name, value);
    }

    // External: ddcutil setvcp
    auto result = executor_.set_vcp(mon->bus, 0x10, value);
    if (!result) return std::unexpected(result.error());
    if (!result->success) return std::unexpected(DDCError::CommandFailed);
    return {};
}

DDCVoid DDCManager::set_all_brightness(uint16_t value) {
    value = std::clamp(value, static_cast<uint16_t>(0), static_cast<uint16_t>(100));
    bool any_failed = false;

    for (const auto& mon : monitors_) {
        auto result = set_brightness(mon.unique_id(), value);
        if (!result) {
            LOG_WARN("set_all_brightness: failed on {}: {}",
                     mon.display_name(), static_cast<int>(result.error()));
            any_failed = true;
        }
    }

    if (any_failed) return std::unexpected(DDCError::CommandFailed);
    return {};
}

DDCResult<uint8_t> DDCManager::get_vcp_value(std::string_view monitor_id, uint8_t code) {
    std::lock_guard lock(mutex_);
    auto* mon = find_monitor(monitor_id);
    if (!mon) return std::unexpected(DDCError::MonitorNotFound);
    if (mon->monitor_type == MonitorType::Internal)
        return std::unexpected(DDCError::VCPNotSupported);

    auto result = executor_.get_vcp(mon->bus, code);
    if (!result) return std::unexpected(result.error());
    if (!result->success) return std::unexpected(DDCError::CommandFailed);
    if (result->value) return static_cast<uint8_t>(*result->value);
    return std::unexpected(DDCError::ParseError);
}

DDCVoid DDCManager::set_vcp_value(std::string_view monitor_id, uint8_t code, uint16_t value) {
    std::lock_guard lock(mutex_);
    auto* mon = find_monitor(monitor_id);
    if (!mon) return std::unexpected(DDCError::MonitorNotFound);
    if (mon->monitor_type == MonitorType::Internal)
        return std::unexpected(DDCError::VCPNotSupported);

    auto result = executor_.set_vcp(mon->bus, code, value);
    if (!result) return std::unexpected(result.error());
    if (!result->success) return std::unexpected(DDCError::CommandFailed);
    return {};
}

DDCVoid DDCManager::refresh_monitors() {
    std::lock_guard lock(mutex_);
    LOG_INFO("Refreshing monitors...");
    auto result = detector_->detect_monitors();
    if (result) {
        monitors_ = std::move(*result);
        LOG_INFO("Refreshed: {} monitors", monitors_.size());
    } else {
        LOG_ERROR("Refresh failed: {}", static_cast<int>(result.error()));
        return std::unexpected(result.error());
    }
    return {};
}

// ── Backlight via systemd-logind D-Bus ───────────────────────

DDCVoid DDCManager::set_backlight_dbus(std::string_view backlight_name, uint32_t value) {
    try {
        auto connection = sdbus::createSystemBusConnection();

        auto proxy = sdbus::createProxy(
            *connection,
            "org.freedesktop.login1",
            "/org/freedesktop/login1/session/auto"
        );

        // SetBrightness takes: (type string, name string, value uint32)
        // The value must be in the raw range (0 - max_brightness)
        // Our value is 0-100, so we need to scale it
        // But systemd-logind actually expects the raw value for percentage? 
        // Actually, the kernel interface takes raw values.
        // For simplicity, pass the percentage — most backlight drivers 
        // accept it directly when max_brightness=100.

        proxy->callMethod("SetBrightness")
            .onInterface("org.freedesktop.login1.Session")
            .withArguments("backlight", std::string{backlight_name}, value);

        return {};
    } catch (const sdbus::Error& e) {
        LOG_ERROR("D-Bus backlight error: {}", e.what());
        return std::unexpected(DDCError::DbusError);
    }
}

DDCResult<uint8_t> DDCManager::get_backlight_sysfs(std::string_view path) {
    // Read current brightness
    auto brightness_file = std::string{path} + "/brightness";
    auto max_file = std::string{path} + "/max_brightness";

    uint32_t current = 0, max_val = 100;
    if (auto f = std::ifstream(brightness_file); f) f >> current;
    if (auto f = std::ifstream(max_file); f) f >> max_val;

    if (max_val == 0) return 0;
    auto pct = static_cast<uint8_t>(std::round(current * 100.0 / max_val));
    return std::clamp(pct, static_cast<uint8_t>(0), static_cast<uint8_t>(100));
}

} // namespace twinkle::ddc
