#include "twinkle/ddc/monitor.hpp"
#include "twinkle/ddc/command_executor.hpp"
#include <algorithm>

namespace twinkle::ddc {

Monitor::Monitor(uint8_t bus, std::string model, std::string serial,
               std::string manufacturer, MonitorCapabilities caps)
    : bus_(bus), model_(std::move(model)), serial_(std::move(serial)),
      manufacturer_(std::move(manufacturer)), caps_(std::move(caps)) {}

std::string Monitor::unique_id() const {
    if (!serial_.empty()) {
        return manufacturer_ + "-" + model_ + "-" + serial_;
    }
    return manufacturer_ + "-" + model_ + "-" + std::to_string(bus_);
}

std::string Monitor::display_name() const {
    if (!serial_.empty()) {
        return model_ + " (" + serial_ + ")";
    }
    return model_;
}

bool Monitor::supports_vcp(uint16_t code) const noexcept {
    return std::find(caps_.supported_vcp_codes.begin(),
                   caps_.supported_vcp_codes.end(), code) !=
           caps_.supported_vcp_codes.end();
}

std::vector<Monitor> MonitorDetector::detect_monitors() const {
    std::vector<Monitor> monitors;

    // Execute ddcutil detect command
    CommandExecutor executor;
    auto result = executor.detect_monitors();

    if (!result.has_value()) {
        return monitors;
    }

    // Parse output to extract monitor information
    // This is simplified - in production, parse the actual ddcutil output
    // For now, return empty list
    return monitors;
}

const Monitor* MonitorDetector::find_by_bus(uint8_t bus) const {
    // This would search through detected monitors
    // For now, return nullptr
    return nullptr;
}

const Monitor* MonitorDetector::find_by_serial(std::string_view serial) const {
    // This would search through detected monitors
    // For now, return nullptr
    return nullptr;
}

bool MonitorDetector::parse_edid(uint8_t bus, std::string& model,
                                 std::string& serial,
                                 std::string& manufacturer) const {
    // Parse EDID data to extract monitor information
    // This is simplified - in production, use proper EDID parsing
    // For now, return false
    return false;
}

MonitorCapabilities MonitorDetector::query_capabilities(uint8_t bus) const {
    MonitorCapabilities caps;
    caps.supports_brightness = true;
    caps.supports_contrast = true;
    caps.supports_volume = false;
    caps.supports_input_source = true;
    caps.supports_color_temp = true;
    caps.supported_vcp_codes = {0x10, 0x12, 0x14, 0x60};
    return caps;
}

} // namespace twinkle::ddc
