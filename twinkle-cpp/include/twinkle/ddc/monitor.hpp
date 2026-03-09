#pragma once

#include "twinkle/ddc/vcp_codes.hpp"
#include <cstdint>
#include <string>
#include <string_view>
#include <vector>

namespace twinkle::ddc {

/// Monitor capabilities
struct MonitorCapabilities {
    std::vector<uint16_t> supported_vcp_codes; ///< Supported VCP codes
    bool supports_brightness{true};             ///< Supports brightness control
    bool supports_contrast{true};              ///< Supports contrast control
    bool supports_volume{false};               ///< Supports volume control
    bool supports_input_source{true};          ///< Supports input source selection
    bool supports_color_temp{true};            ///< Supports color temperature
};

/// Monitor representation
class Monitor {
public:
    Monitor(uint8_t bus, std::string model, std::string serial,
            std::string manufacturer, MonitorCapabilities caps);

    // Non-copyable but movable (RAII best practice)
    Monitor(const Monitor&) = delete;
    Monitor& operator=(const Monitor&) = delete;
    Monitor(Monitor&&) noexcept = default;
    Monitor& operator=(Monitor&&) noexcept = default;
    ~Monitor() = default;

    /// Get monitor bus number
    [[nodiscard]] uint8_t bus() const noexcept { return bus_; }

    /// Get monitor model name
    [[nodiscard]] const std::string& model() const noexcept { return model_; }

    /// Get monitor serial number
    [[nodiscard]] const std::string& serial() const noexcept { return serial_; }

    /// Get monitor manufacturer
    [[nodiscard]] const std::string& manufacturer() const noexcept { return manufacturer_; }

    /// Get monitor capabilities
    [[nodiscard]] const MonitorCapabilities& capabilities() const noexcept { return caps_; }

    /// Get unique ID for the monitor
    [[nodiscard]] std::string unique_id() const;

    /// Get display name (model + serial if available)
    [[nodiscard]] std::string display_name() const;

    /// Check if VCP code is supported
    [[nodiscard]] bool supports_vcp(uint16_t code) const noexcept;

private:
    uint8_t bus_;
    std::string model_;
    std::string serial_;
    std::string manufacturer_;
    MonitorCapabilities caps_;
};

/// Monitor detector for discovering connected monitors
class MonitorDetector {
public:
    MonitorDetector() = default;
    ~MonitorDetector() = default;

    // Non-copyable but movable
    MonitorDetector(const MonitorDetector&) = delete;
    MonitorDetector& operator=(const MonitorDetector&) = delete;
    MonitorDetector(MonitorDetector&&) noexcept = default;
    MonitorDetector& operator=(MonitorDetector&&) noexcept = default;

    /// Detect all connected monitors
    [[nodiscard]] std::vector<Monitor> detect_monitors() const;

    /// Find monitor by bus number
    [[nodiscard]] const Monitor* find_by_bus(uint8_t bus) const;

    /// Find monitor by serial number
    [[nodiscard]] const Monitor* find_by_serial(std::string_view serial) const;

private:
    /// Parse EDID data to extract monitor information
    [[nodiscard]] bool parse_edid(uint8_t bus, std::string& model,
                                  std::string& serial, std::string& manufacturer) const;

    /// Query monitor capabilities
    [[nodiscard]] MonitorCapabilities query_capabilities(uint8_t bus) const;
};

} // namespace twinkle::ddc
