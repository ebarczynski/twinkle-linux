#pragma once

#include "twinkle/ddc/command_executor.hpp"
#include "twinkle/ddc/error.hpp"
#include "twinkle/ddc/monitor.hpp"
#include "twinkle/ddc/vcp_codes.hpp"
#include <cstdint>
#include <functional>
#include <memory>
#include <optional>
#include <string>
#include <unordered_map>
#include <vector>

namespace twinkle::ddc {

/// Callback type for monitor changes
using MonitorCallback = std::function<void(const std::vector<Monitor>&)>;

/// DDC Manager - Main interface for DDC/CI operations
class DDCManager {
public:
    explicit DDCManager(CommandConfig config = {});
    ~DDCManager() = default;

    // Non-copyable but movable
    DDCManager(const DDCManager&) = delete;
    DDCManager& operator=(const DDCManager&) = delete;
    DDCManager(DDCManager&&) noexcept = default;
    DDCManager& operator=(DDCManager&&) noexcept = default;

    /// Initialize DDC manager and detect monitors
    [[nodiscard]] Result<void> initialize();

    /// Get all detected monitors
    [[nodiscard]] const std::vector<Monitor>& monitors() const noexcept { return monitors_; }

    /// Get monitor by bus number
    [[nodiscard]] const Monitor* find_monitor(uint8_t bus) const;

    /// Get monitor by unique ID
    [[nodiscard]] const Monitor* find_monitor(std::string_view unique_id) const;

    /// Get VCP value
    [[nodiscard]] Result<uint8_t> get_vcp(uint8_t bus, uint16_t code) const;

    /// Set VCP value
    [[nodiscard]] Result<void> set_vcp(uint8_t bus, uint16_t code, uint8_t value);

    /// Get brightness
    [[nodiscard]] Result<uint8_t> get_brightness(uint8_t bus) const;

    /// Set brightness
    [[nodiscard]] Result<void> set_brightness(uint8_t bus, uint8_t value);

    /// Adjust brightness by delta
    [[nodiscard]] Result<void> adjust_brightness(uint8_t bus, int8_t delta);

    /// Get contrast
    [[nodiscard]] Result<uint8_t> get_contrast(uint8_t bus) const;

    /// Set contrast
    [[nodiscard]] Result<void> set_contrast(uint8_t bus, uint8_t value);

    /// Get volume
    [[nodiscard]] Result<uint8_t> get_volume(uint8_t bus) const;

    /// Set volume
    [[nodiscard]] Result<void> set_volume(uint8_t bus, uint8_t value);

    /// Get input source
    [[nodiscard]] Result<uint8_t> get_input_source(uint8_t bus) const;

    /// Set input source
    [[nodiscard]] Result<void> set_input_source(uint8_t bus, uint8_t value);

    /// Get color temperature
    [[nodiscard]] Result<uint8_t> get_color_temperature(uint8_t bus) const;

    /// Set color temperature
    [[nodiscard]] Result<void> set_color_temperature(uint8_t bus, uint8_t value);

    /// Refresh monitor list
    [[nodiscard]] Result<void> refresh_monitors();

    /// Check if user has I2C permissions
    [[nodiscard]] bool has_permissions() const noexcept;

    /// Set callback for monitor changes
    void set_monitor_callback(MonitorCallback callback) {
        monitor_callback_ = std::move(callback);
    }

private:
    CommandExecutor executor_;
    MonitorDetector detector_;
    std::vector<Monitor> monitors_;
    MonitorCallback monitor_callback_;

    /// Cache for VCP values to reduce monitor communication
    struct VCPValue {
        uint8_t value;
        std::chrono::steady_clock::time_point timestamp;
    };

    std::unordered_map<std::string, VCPValue> vcp_cache_;

    /// Get cached VCP value or fetch from monitor
    [[nodiscard]] Result<uint8_t> get_cached_vcp(uint8_t bus, uint16_t code);

    /// Set VCP value and update cache
    [[nodiscard]] Result<void> set_cached_vcp(uint8_t bus, uint16_t code, uint8_t value);

    /// Invalidate cache for a monitor
    void invalidate_cache(uint8_t bus);

    /// Generate cache key
    [[nodiscard]] std::string cache_key(uint8_t bus, uint16_t code) const;
};

} // namespace twinkle::ddc
