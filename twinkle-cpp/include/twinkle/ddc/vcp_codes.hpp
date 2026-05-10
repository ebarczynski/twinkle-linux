#pragma once
/// @file vcp_codes.hpp
/// @brief VCP (Virtual Control Panel) code definitions.
///
/// Defines all known VCP codes as a strongly-typed enum with constexpr
/// metadata. In C++26 with reflection, to_string/from_string would be
/// auto-generated. For now we use an X-macro table.

#include <cstdint>
#include <optional>
#include <string_view>

namespace twinkle::ddc::vcp {

/// VCP code identifier.
enum class VCP : uint8_t {
    // Display controls
    Brightness       = 0x10,
    Contrast         = 0x12,
    AutoColorSetup   = 0x1F,
    RedGain          = 0x16,
    GreenGain        = 0x18,
    BlueGain         = 0x1A,
    Focus            = 0x1C,
    HorizSize        = 0x22,
    VertSize         = 0x32,
    HorizPosition    = 0x20,
    VertPosition     = 0x30,
    Tilt             = 0x34,
    RedBlackLevel    = 0x36,
    GreenBlackLevel  = 0x38,
    BlueBlackLevel   = 0x3A,
    ClockPhase       = 0x3E,
    ClockPixel       = 0x3C,
    // Power / DPMS
    DPMSControl      = 0xD6,
    ActiveControl    = 0x52,
    // Input
    InputSource      = 0x60,
    // Audio
    AudioMute        = 0x8D,
    Volume           = 0x62,
    // Color / Temperature
    ColorTemperature = 0x14,
    // Miscellaneous
    OSD              = 0xCA,
    Language         = 0xCC,
    PowerMode        = 0xD6,
    Technology       = 0xB6,
    // Capabilities
    Capabilities     = 0xF0,
    // OEM
    ManufacturerSpecific = 0xF5,
};

/// Value type classification.
enum class ValueType : uint8_t {
    Continuous,    ///< Slider (0-100 range)
    NonContinuous, ///< Dropdown (discrete values)
    ReadOnly,
    WriteOnly,
};

/// Info about a specific VCP code.
struct VCPCodeInfo {
    VCP code;
    const char* name;
    ValueType value_type;
    uint16_t min_value;
    uint16_t max_value;
};

// ─── VCP Code Registry (constexpr table) ─────────────────────

struct VCPEntry {
    uint8_t code;
    const char* name;
    ValueType type;
    uint16_t min_val;
    uint16_t max_val;
};

/// All known VCP codes — constexpr lookup table.
/// C++26 reflection would auto-generate this from the enum.
static constexpr VCPEntry kVCPTable[] = {
    {0x10, "Brightness",       ValueType::Continuous,    0, 100},
    {0x12, "Contrast",         ValueType::Continuous,    0, 100},
    {0x14, "Color Temperature",ValueType::NonContinuous,  0, 20},
    {0x16, "Red Gain",         ValueType::Continuous,    0, 255},
    {0x18, "Green Gain",       ValueType::Continuous,    0, 255},
    {0x1A, "Blue Gain",        ValueType::Continuous,    0, 255},
    {0x20, "Horizontal Position", ValueType::Continuous, 0, 100},
    {0x30, "Vertical Position",  ValueType::Continuous, 0, 100},
    {0x60, "Input Source",     ValueType::NonContinuous,  0, 15},
    {0x62, "Volume",           ValueType::Continuous,    0, 100},
    {0x8D, "Audio Mute",       ValueType::NonContinuous,  0, 2},
    {0xB6, "Display Technology",ValueType::ReadOnly,     0, 0},
    {0xCA, "OSD",              ValueType::NonContinuous,  0, 1},
    {0xD6, "DPMS / Power",     ValueType::NonContinuous,  0, 5},
    {0xF0, "Capabilities",     ValueType::ReadOnly,      0, 0},
};

/// Get VCP info by code. constexpr-friendly.
[[nodiscard]] constexpr std::optional<VCPEntry> get_vcp_info(uint8_t code) noexcept {
    for (const auto& entry : kVCPTable) {
        if (entry.code == code) return entry;
    }
    return std::nullopt;
}

/// Get VCP code name, or "Unknown (0xNN)".
[[nodiscard]] inline std::string vcp_code_name(uint8_t code) {
    if (auto info = get_vcp_info(code)) {
        return info->name;
    }
    char buf[32];
    std::snprintf(buf, sizeof(buf), "Unknown (0x%02X)", code);
    return buf;
}

/// Validate that a VCP value is within the expected range.
[[nodiscard]] constexpr bool validate_vcp_value(uint8_t code, uint16_t value) noexcept {
    if (auto info = get_vcp_info(code)) {
        return value >= info->min_val && value <= info->max_val;
    }
    return true; // Unknown code — allow
}

/// Common VCP codes used for brightness control features.
[[nodiscard]] constexpr auto common_vcp_codes() noexcept {
    struct { uint8_t codes[5]; uint8_t count{5}; } result{
        {0x10, 0x12, 0x14, 0x60, 0x62}, 5
    };
    return result;
}

} // namespace twinkle::ddc::vcp
