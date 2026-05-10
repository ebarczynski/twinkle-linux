#pragma once
/// @file monitor.hpp
/// @brief Monitor struct and detector.

#include "twinkle/ddc/error.hpp"
#include <chrono>
#include <cstdint>
#include <set>
#include <string>
#include <vector>

namespace twinkle::ddc {

/// Whether this is an external DDC/CI monitor or internal backlight.
enum class MonitorType : uint8_t {
    External,  ///< DDC/CI over I2C
    Internal,  ///< Kernel backlight sysfs
};

/// Monitor capabilities queried from ddcutil.
struct MonitorCapabilities {
    std::set<uint8_t> supported_vcp_codes;
    uint16_t max_brightness{100};
    uint16_t max_contrast{100};
    bool supports_input_source{false};
    bool supports_power_control{false};
    bool supports_audio{false};

    [[nodiscard]] bool supports_vcp(uint8_t code) const noexcept {
        return supported_vcp_codes.contains(code);
    }
};

/// Represents a physical monitor connected to the system.
struct Monitor {
    int32_t bus{-1};
    std::string model{"Unknown Monitor"};
    std::string serial;
    std::string manufacturer;
    std::string edid_data;
    MonitorCapabilities capabilities;
    MonitorType monitor_type{MonitorType::External};
    std::string backlight_path;  ///< /sys/class/backlight/... (internal only)

    // ── Factory methods ──────────────────────────────────────

    /// Create an external DDC/CI monitor.
    [[nodiscard]] static Monitor external(int32_t bus) {
        return Monitor{.bus = bus, .monitor_type = MonitorType::External};
    }

    /// Create an internal backlight display.
    [[nodiscard]] static Monitor internal(std::string_view name, std::string_view path) {
        return Monitor{
            .model = std::string{name},
            .manufacturer = "Internal",
            .monitor_type = MonitorType::Internal,
            .backlight_path = std::string{path},
        };
    }

    // ── Accessors ────────────────────────────────────────────

    /// Human-readable display name.
    [[nodiscard]] std::string display_name() const {
        if (monitor_type == MonitorType::Internal) {
            return std::format("{} (Internal)", model);
        }
        if (!manufacturer.empty() && model != "Unknown Monitor") {
            if (!serial.empty() && serial != "Unknown")
                return std::format("{} {} ({})", manufacturer, model, serial);
            return std::format("{} {}", manufacturer, model);
        }
        if (model != "Unknown Monitor") {
            if (!serial.empty() && serial != "Unknown")
                return std::format("{} ({})", model, serial);
            return model;
        }
        return std::format("Monitor (bus {})", bus);
    }

    /// Unique identifier for config caching.
    [[nodiscard]] std::string unique_id() const {
        if (monitor_type == MonitorType::Internal)
            return std::format("internal_{}", model);
        if (!serial.empty() && serial != "Unknown")
            return serial;
        return std::format("{}_bus{}", model, bus);
    }

    /// Validate monitor data.
    [[nodiscard]] DDCVoid validate() const {
        if (monitor_type == MonitorType::External && bus < 0)
            return std::unexpected(DDCError::MonitorNotFound);
        return {};
    }
};

/// Detects monitors on the system (DDC/CI + backlight).
class MonitorDetector {
public:
    explicit MonitorDetector(class CommandExecutor& executor)
        : executor_(executor) {}

    /// Detect all monitors (internal backlight + external DDC/CI).
    [[nodiscard]] DDCResult<std::vector<Monitor>> detect_monitors();

private:
    CommandExecutor& executor_;

    /// Scan /sys/class/backlight for internal displays.
    void detect_internal_backlights(std::vector<Monitor>& out);

    /// Parse ddcutil detect output.
    [[nodiscard]] DDCResult<std::vector<Monitor>> parse_detect_output(std::string_view output);

    /// Get capabilities for a bus.
    [[nodiscard]] DDCResult<MonitorCapabilities> get_capabilities(int32_t bus);

    /// Parse capabilities output.
    [[nodiscard]] MonitorCapabilities parse_capabilities(std::string_view output);
};

} // namespace twinkle::ddc
