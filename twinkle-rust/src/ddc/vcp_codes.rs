//! VCP (Virtual Control Panel) code definitions for DDC/CI operations.

use std::collections::HashMap;

/// Type of values a VCP code can accept.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueType {
    /// Numeric range (e.g., 0-100)
    Continuous,
    /// Enum-like values
    NonContinuous,
    /// Read-only value
    ReadOnly,
    /// Write-only value
    WriteOnly,
}

/// Information about a VCP code.
#[derive(Debug, Clone)]
pub struct VCPCodeInfo {
    /// The VCP code as a hexadecimal value (0x00-0xFF)
    pub code: u8,
    /// Human-readable name of the VCP code
    pub name: &'static str,
    /// Detailed description of what the VCP code controls
    pub description: &'static str,
    /// Type of values this VCP code accepts
    pub value_type: ValueType,
    /// Minimum value for continuous VCP codes
    pub min_value: u16,
    /// Maximum value for continuous VCP codes
    pub max_value: u16,
    /// Dictionary of enum values for non-continuous VCP codes
    pub values: Option<HashMap<u16, &'static str>>,
}

impl VCPCodeInfo {
    /// Get the human-readable name for a non-continuous VCP code value.
    pub fn get_value_name(&self, value: u16) -> Option<&'static str> {
        if self.value_type != ValueType::NonContinuous {
            return None;
        }
        self.values.as_ref()?.get(&value).copied()
    }

    /// Validate if a value is valid for this VCP code.
    pub fn validate_value(&self, value: u16) -> bool {
        match self.value_type {
            ValueType::Continuous => self.min_value <= value && value <= self.max_value,
            ValueType::NonContinuous => self.values.as_ref().map_or(false, |v| v.contains_key(&value)),
            _ => false,
        }
    }
}

/// Common VCP codes for monitor control.
pub fn get_common_vcp_codes() -> Vec<VCPCodeInfo> {
    vec![
        VCPCodeInfo {
            code: 0x10,
            name: "Brightness",
            description: "Controls the screen brightness level",
            value_type: ValueType::Continuous,
            min_value: 0,
            max_value: 100,
            values: None,
        },
        VCPCodeInfo {
            code: 0x12,
            name: "Contrast",
            description: "Controls the screen contrast level",
            value_type: ValueType::Continuous,
            min_value: 0,
            max_value: 100,
            values: None,
        },
        VCPCodeInfo {
            code: 0x14,
            name: "Color Temperature",
            description: "Controls the color temperature preset",
            value_type: ValueType::NonContinuous,
            min_value: 0,
            max_value: 0,
            values: Some(color_temperature_values()),
        },
        VCPCodeInfo {
            code: 0x60,
            name: "Input Source",
            description: "Selects the video input source",
            value_type: ValueType::NonContinuous,
            min_value: 0,
            max_value: 0,
            values: Some(input_source_values()),
        },
        VCPCodeInfo {
            code: 0x62,
            name: "Audio Speaker Volume",
            description: "Controls the built-in speaker volume",
            value_type: ValueType::Continuous,
            min_value: 0,
            max_value: 100,
            values: None,
        },
        VCPCodeInfo {
            code: 0x8D,
            name: "Audio Mute",
            description: "Mutes the built-in speakers",
            value_type: ValueType::NonContinuous,
            min_value: 0,
            max_value: 0,
            values: Some(audio_mute_values()),
        },
        VCPCodeInfo {
            code: 0xD6,
            name: "Power Mode",
            description: "Controls the monitor power state",
            value_type: ValueType::NonContinuous,
            min_value: 0,
            max_value: 0,
            values: Some(power_mode_values()),
        },
        VCPCodeInfo {
            code: 0x86,
            name: "Display Technology Type",
            description: "Reports the display technology type",
            value_type: ValueType::NonContinuous,
            min_value: 0,
            max_value: 0,
            values: Some(display_technology_values()),
        },
        VCPCodeInfo {
            code: 0x02,
            name: "New Control Value",
            description: "Indicates a new control value is available",
            value_type: ValueType::ReadOnly,
            min_value: 0,
            max_value: 0,
            values: None,
        },
        VCPCodeInfo {
            code: 0x01,
            name: "VCP Version",
            description: "Reports the supported VCP version",
            value_type: ValueType::ReadOnly,
            min_value: 0,
            max_value: 0,
            values: None,
        },
    ]
}

/// Get information about a specific VCP code.
pub fn get_vcp_info(code: u8) -> Option<VCPCodeInfo> {
    get_common_vcp_codes().into_iter().find(|info| info.code == code)
}

/// Color temperature presets for VCP code 0x14.
fn color_temperature_values() -> HashMap<u16, &'static str> {
    let mut map = HashMap::new();
    map.insert(0x00, "50K (Kelvin)");
    map.insert(0x01, "User 1");
    map.insert(0x02, "User 2");
    map.insert(0x03, "User 3");
    map.insert(0x04, "Warm (4000K)");
    map.insert(0x05, "5000K");
    map.insert(0x06, "6500K (sRGB)");
    map.insert(0x07, "7500K");
    map.insert(0x08, "8200K");
    map.insert(0x09, "9300K");
    map.insert(0x0A, "10000K");
    map.insert(0x0B, "11500K");
    map.insert(0x0C, "Native");
    map
}

/// Input source values for VCP code 0x60.
fn input_source_values() -> HashMap<u16, &'static str> {
    let mut map = HashMap::new();
    map.insert(0x00, "Auto");
    map.insert(0x01, "VGA");
    map.insert(0x02, "DVI-1");
    map.insert(0x03, "DVI-2");
    map.insert(0x04, "Composite");
    map.insert(0x05, "S-Video");
    map.insert(0x06, "Tuner");
    map.insert(0x07, "Component");
    map.insert(0x08, "DisplayPort-1");
    map.insert(0x09, "DisplayPort-2");
    map.insert(0x0A, "DisplayPort-3");
    map.insert(0x0B, "HDMI-1");
    map.insert(0x0C, "HDMI-2");
    map.insert(0x0D, "HDMI-3");
    map.insert(0x0E, "HDMI-4");
    map.insert(0x0F, "DisplayPort-1 (alt)");
    map.insert(0x10, "DisplayPort-2 (alt)");
    map.insert(0x11, "DisplayPort-3 (alt)");
    map.insert(0x12, "DisplayPort-4");
    map.insert(0x13, "DisplayPort-5");
    map.insert(0x14, "DisplayPort-6");
    map.insert(0x15, "USB-C");
    map
}

/// Power mode values for VCP code 0xD6.
fn power_mode_values() -> HashMap<u16, &'static str> {
    let mut map = HashMap::new();
    map.insert(0x00, "On");
    map.insert(0x01, "Standby");
    map.insert(0x02, "Suspend");
    map.insert(0x03, "Off (Soft)");
    map.insert(0x04, "Off (Hard)");
    map
}

/// Display technology type values for VCP code 0x86.
fn display_technology_values() -> HashMap<u16, &'static str> {
    let mut map = HashMap::new();
    map.insert(0x00, "CRT");
    map.insert(0x01, "LCD");
    map.insert(0x02, "OLED");
    map.insert(0x03, "Plasma");
    map.insert(0x04, "LED");
    map.insert(0x05, "DLP");
    map.insert(0x06, "LCoS");
    map.insert(0x07, "DLPLCD");
    map.insert(0x08, "LCOS");
    map
}

/// Audio mute values for VCP code 0x8D.
fn audio_mute_values() -> HashMap<u16, &'static str> {
    let mut map = HashMap::new();
    map.insert(0x00, "Mute Off");
    map.insert(0x01, "Mute On");
    map
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_vcp_info() {
        let info = get_vcp_info(0x10);
        assert!(info.is_some());
        assert_eq!(info.unwrap().name, "Brightness");
    }

    #[test]
    fn test_validate_value_continuous() {
        let info = get_vcp_info(0x10).unwrap();
        assert!(info.validate_value(50));
        assert!(info.validate_value(0));
        assert!(info.validate_value(100));
        assert!(!info.validate_value(101));
    }

    #[test]
    fn test_validate_value_non_continuous() {
        let info = get_vcp_info(0x14).unwrap();
        assert!(info.validate_value(0x06));
        assert!(!info.validate_value(0xFF));
    }

    #[test]
    fn test_get_value_name() {
        let info = get_vcp_info(0x14).unwrap();
        assert_eq!(info.get_value_name(0x06), Some("6500K (sRGB)"));
        assert_eq!(info.get_value_name(0xFF), None);
    }
}
