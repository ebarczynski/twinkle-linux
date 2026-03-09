#pragma once

#include <cstdint>
#include <string_view>
#include <unordered_map>

namespace twinkle::ddc {

/// VCP code value type
enum class VCPType : uint8_t {
    Continuous,    ///< Continuous value (0-255)
    NonContinuous, ///< Discrete value (enumeration)
    ReadOnly,      ///< Read-only value
    WriteOnly      ///< Write-only value
};

/// VCP code information
struct VCPInfo {
    uint16_t code;          ///< VCP code
    std::string_view name;   ///< Human-readable name
    std::string_view description; ///< Description
    VCPType type;           ///< Value type
    uint8_t min_value;      ///< Minimum value (for continuous)
    uint8_t max_value;      ///< Maximum value (for continuous)
};

/// Common VCP codes
namespace vcp {
    constexpr inline uint16_t Brightness        = 0x10;
    constexpr inline uint16_t Contrast         = 0x12;
    constexpr inline uint16_t ColorTemperature = 0x14;
    constexpr inline uint16_t InputSource      = 0x60;
    constexpr inline uint16_t Volume           = 0x62;
    constexpr inline uint16_t PowerMode        = 0xD6;
}

/// Input source values
namespace input_source {
    constexpr inline uint8_t DP1  = 0x0F;
    constexpr inline uint8_t DP2  = 0x10;
    constexpr inline uint8_t HDMI1 = 0x11;
    constexpr inline uint8_t HDMI2 = 0x12;
    constexpr inline uint8_t VGA  = 0x01;
    constexpr inline uint8_t DVI  = 0x03;
}

/// Color temperature values (in Kelvin)
namespace color_temp {
    constexpr inline uint8_t K5000  = 0x04;
    constexpr inline uint8_t K6500  = 0x05;
    constexpr inline uint8_t K7500  = 0x06;
    constexpr inline uint8_t K8200  = 0x07;
    constexpr inline uint8_t K9300  = 0x08;
    constexpr inline uint8_t K10000 = 0x0B;
}

/// Get VCP code information
[[nodiscard]] constexpr const VCPInfo* get_vcp_info(uint16_t code) noexcept {
    static constexpr VCPInfo vcp_codes[] = {
        {vcp::Brightness, "Brightness", "Display brightness level", VCPType::Continuous, 0, 100},
        {vcp::Contrast, "Contrast", "Display contrast level", VCPType::Continuous, 0, 100},
        {vcp::ColorTemperature, "Color Temperature", "Display color temperature", VCPType::NonContinuous, 0, 255},
        {vcp::InputSource, "Input Source", "Display input source", VCPType::NonContinuous, 0, 255},
        {vcp::Volume, "Volume", "Audio volume level", VCPType::Continuous, 0, 100},
        {vcp::PowerMode, "Power Mode", "Display power mode", VCPType::NonContinuous, 0, 255},
    };

    for (const auto& info : vcp_codes) {
        if (info.code == code) {
            return &info;
        }
    }
    return nullptr;
}

/// Get human-readable name for VCP code
[[nodiscard]] constexpr std::string_view get_vcp_name(uint16_t code) noexcept {
    if (const auto* info = get_vcp_info(code)) {
        return info->name;
    }
    return "Unknown";
}

/// Validate VCP value
[[nodiscard]] constexpr bool validate_vcp_value(uint16_t code, uint8_t value) noexcept {
    if (const auto* info = get_vcp_info(code)) {
        if (info->type == VCPType::Continuous) {
            return value >= info->min_value && value <= info->max_value;
        }
        return true; // Non-continuous values are typically validated by the monitor
    }
    return false;
}

} // namespace twinkle::ddc
