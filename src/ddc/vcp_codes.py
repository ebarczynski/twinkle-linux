"""
VCP (Virtual Control Panel) code definitions for DDC/CI operations.

This module provides a comprehensive dictionary of VCP codes with their
names, descriptions, and value ranges. VCP codes are used to communicate
with monitors to control various settings like brightness, contrast, etc.
"""

from __future__ import annotations

from dataclasses import dataclass
from enum import Enum
from typing import Literal


class ValueType(Enum):
    """Type of values a VCP code can accept."""

    CONTINUOUS = "continuous"  # Numeric range (e.g., 0-100)
    NON_CONTINUOUS = "non_continuous"  # Enum-like values
    READ_ONLY = "read_only"  # Read-only value
    WRITE_ONLY = "write_only"  # Write-only value


@dataclass
class VCPCodeInfo:
    """
    Information about a VCP code.

    Attributes:
        code: The VCP code as a hexadecimal value (0x00-0xFF)
        name: Human-readable name of the VCP code
        description: Detailed description of what the VCP code controls
        value_type: Type of values this VCP code accepts
        min_value: Minimum value for continuous VCP codes
        max_value: Maximum value for continuous VCP codes
        values: Dictionary of enum values for non-continuous VCP codes
    """

    code: int
    name: str
    description: str
    value_type: ValueType = ValueType.CONTINUOUS
    min_value: int = 0
    max_value: int = 100
    values: dict[int, str] | None = None

    def __post_init__(self) -> None:
        """Validate the VCP code info."""
        if not 0x00 <= self.code <= 0xFF:
            raise ValueError(f"VCP code must be between 0x00 and 0xFF, got 0x{self.code:02X}")

        if self.value_type == ValueType.CONTINUOUS:
            if self.min_value > self.max_value:
                raise ValueError(f"min_value ({self.min_value}) must be <= max_value ({self.max_value})")
        elif self.value_type == ValueType.NON_CONTINUOUS and not self.values:
            raise ValueError("Non-continuous VCP codes must have values dictionary")

    def get_value_name(self, value: int) -> str | None:
        """
        Get the human-readable name for a non-continuous VCP code value.

        Args:
            value: The VCP code value

        Returns:
            The human-readable name, or None if not found or not applicable
        """
        if self.value_type != ValueType.NON_CONTINUOUS or not self.values:
            return None
        return self.values.get(value)

    def validate_value(self, value: int) -> bool:
        """
        Validate if a value is valid for this VCP code.

        Args:
            value: The value to validate

        Returns:
            True if the value is valid, False otherwise
        """
        if self.value_type == ValueType.CONTINUOUS:
            return self.min_value <= value <= self.max_value
        elif self.value_type == ValueType.NON_CONTINUOUS:
            return value in (self.values or {})
        return False


# Color temperature presets for VCP code 0x14
COLOR_TEMPERATURE_VALUES: dict[int, str] = {
    0x00: "50K (Kelvin)",
    0x01: "User 1",
    0x02: "User 2",
    0x03: "User 3",
    0x04: "Warm (4000K)",
    0x05: "5000K",
    0x06: "6500K (sRGB)",
    0x07: "7500K",
    0x08: "8200K",
    0x09: "9300K",
    0x0A: "10000K",
    0x0B: "11500K",
    0x0C: "Native",
}

# Input source values for VCP code 0x60
INPUT_SOURCE_VALUES: dict[int, str] = {
    0x00: "Auto",
    0x01: "VGA",
    0x02: "DVI-1",
    0x03: "DVI-2",
    0x04: "Composite",
    0x05: "S-Video",
    0x06: "Tuner",
    0x07: "Component",
    0x08: "DisplayPort-1",
    0x09: "DisplayPort-2",
    0x0A: "DisplayPort-3",
    0x0B: "HDMI-1",
    0x0C: "HDMI-2",
    0x0D: "HDMI-3",
    0x0E: "HDMI-4",
    0x0F: "DisplayPort-1 (alt)",
    0x10: "DisplayPort-2 (alt)",
    0x11: "DisplayPort-3 (alt)",
    0x12: "DisplayPort-4",
    0x13: "DisplayPort-5",
    0x14: "DisplayPort-6",
    0x15: "USB-C",
}

# Power mode values for VCP code 0xD6
POWER_MODE_VALUES: dict[int, str] = {
    0x00: "On",
    0x01: "Standby",
    0x02: "Suspend",
    0x03: "Off (Soft)",
    0x04: "Off (Hard)",
}

# Display technology type values for VCP code 0x86
DISPLAY_TECHNOLOGY_VALUES: dict[int, str] = {
    0x00: "CRT",
    0x01: "LCD",
    0x02: "OLED",
    0x03: "Plasma",
    0x04: "LED",
    0x05: "DLP",
    0x06: "LCoS",
    0x07: "Other",
}

# Main VCP code definitions
VCP_CODES: dict[int, VCPCodeInfo] = {
    # Image adjustments
    0x10: VCPCodeInfo(
        code=0x10,
        name="Brightness",
        description="Display brightness level",
        value_type=ValueType.CONTINUOUS,
        min_value=0,
        max_value=100,
    ),
    0x11: VCPCodeInfo(
        code=0x11,
        name="Contrast",
        description="Display contrast level",
        value_type=ValueType.CONTINUOUS,
        min_value=0,
        max_value=100,
    ),
    0x12: VCPCodeInfo(
        code=0x12,
        name="Contrast (Alt)",
        description="Alternative contrast control",
        value_type=ValueType.CONTINUOUS,
        min_value=0,
        max_value=100,
    ),
    0x13: VCPCodeInfo(
        code=0x13,
        name="Red Gain",
        description="Red color channel gain",
        value_type=ValueType.CONTINUOUS,
        min_value=0,
        max_value=100,
    ),
    0x14: VCPCodeInfo(
        code=0x14,
        name="Color Temperature",
        description="Color temperature preset",
        value_type=ValueType.NON_CONTINUOUS,
        values=COLOR_TEMPERATURE_VALUES,
    ),
    0x15: VCPCodeInfo(
        code=0x15,
        name="Green Gain",
        description="Green color channel gain",
        value_type=ValueType.CONTINUOUS,
        min_value=0,
        max_value=100,
    ),
    0x16: VCPCodeInfo(
        code=0x16,
        name="Blue Gain",
        description="Blue color channel gain",
        value_type=ValueType.CONTINUOUS,
        min_value=0,
        max_value=100,
    ),
    0x17: VCPCodeInfo(
        code=0x17,
        name="Red Drive",
        description="Red color channel drive",
        value_type=ValueType.CONTINUOUS,
        min_value=0,
        max_value=100,
    ),
    0x18: VCPCodeInfo(
        code=0x18,
        name="Green Drive",
        description="Green color channel drive",
        value_type=ValueType.CONTINUOUS,
        min_value=0,
        max_value=100,
    ),
    0x19: VCPCodeInfo(
        code=0x19,
        name="Blue Drive",
        description="Blue color channel drive",
        value_type=ValueType.CONTINUOUS,
        min_value=0,
        max_value=100,
    ),
    # Geometry adjustments
    0x20: VCPCodeInfo(
        code=0x20,
        name="Horizontal Position",
        description="Horizontal screen position",
        value_type=ValueType.CONTINUOUS,
        min_value=0,
        max_value=100,
    ),
    0x21: VCPCodeInfo(
        code=0x21,
        name="Vertical Position",
        description="Vertical screen position",
        value_type=ValueType.CONTINUOUS,
        min_value=0,
        max_value=100,
    ),
    0x22: VCPCodeInfo(
        code=0x22,
        name="Horizontal Size",
        description="Horizontal screen size",
        value_type=ValueType.CONTINUOUS,
        min_value=0,
        max_value=100,
    ),
    0x23: VCPCodeInfo(
        code=0x23,
        name="Vertical Size",
        description="Vertical screen size",
        value_type=ValueType.CONTINUOUS,
        min_value=0,
        max_value=100,
    ),
    # Audio controls
    0x60: VCPCodeInfo(
        code=0x60,
        name="Input Source",
        description="Video input source selection",
        value_type=ValueType.NON_CONTINUOUS,
        values=INPUT_SOURCE_VALUES,
    ),
    0x62: VCPCodeInfo(
        code=0x62,
        name="Speaker Volume",
        description="Monitor speaker volume",
        value_type=ValueType.CONTINUOUS,
        min_value=0,
        max_value=100,
    ),
    0x6C: VCPCodeInfo(
        code=0x6C,
        name="Audio Mute",
        description="Monitor speaker mute state",
        value_type=ValueType.NON_CONTINUOUS,
        values={0x00: "Unmuted", 0x01: "Muted"},
    ),
    0x8D: VCPCodeInfo(
        code=0x8D,
        name="Audio Mute (Alt)",
        description="Alternative audio mute control",
        value_type=ValueType.NON_CONTINUOUS,
        values={0x00: "Unmuted", 0x01: "Muted"},
    ),
    # Display information
    0x86: VCPCodeInfo(
        code=0x86,
        name="Display Technology Type",
        description="Type of display technology",
        value_type=ValueType.NON_CONTINUOUS,
        values=DISPLAY_TECHNOLOGY_VALUES,
    ),
    0x87: VCPCodeInfo(
        code=0x87,
        name="Display Usage Time",
        description="Total hours the display has been used",
        value_type=ValueType.READ_ONLY,
        min_value=0,
        max_value=65535,
    ),
    # DDC/CI control
    0xB6: VCPCodeInfo(
        code=0xB6,
        name="Control Mode",
        description="DDC/CI control mode",
        value_type=ValueType.NON_CONTINUOUS,
        values={0x00: "Disabled", 0x01: "Enabled"},
    ),
    0xC0: VCPCodeInfo(
        code=0xC0,
        name="VCP Version",
        description="VCP version supported by the monitor",
        value_type=ValueType.READ_ONLY,
        min_value=0,
        max_value=255,
    ),
    0xC6: VCPCodeInfo(
        code=0xC6,
        name="Application Enable Key",
        description="DDC/CI enable key (write-only)",
        value_type=ValueType.WRITE_ONLY,
        min_value=0,
        max_value=255,
    ),
    0xC8: VCPCodeInfo(
        code=0xC8,
        name="Display Firmware Level",
        description="Firmware version of the display",
        value_type=ValueType.READ_ONLY,
        min_value=0,
        max_value=65535,
    ),
    0xCC: VCPCodeInfo(
        code=0xCC,
        name="OSD Language",
        description="On-screen display language",
        value_type=ValueType.NON_CONTINUOUS,
        values={
            0x00: "English",
            0x01: "French",
            0x02: "German",
            0x03: "Italian",
            0x04: "Spanish",
            0x05: "Swedish",
            0x06: "Russian",
            0x07: "Simplified Chinese",
            0x08: "Traditional Chinese",
        },
    ),
    # Power control
    0xD6: VCPCodeInfo(
        code=0xD6,
        name="Power Mode",
        description="Display power state",
        value_type=ValueType.NON_CONTINUOUS,
        values=POWER_MODE_VALUES,
    ),
    0xDF: VCPCodeInfo(
        code=0xDF,
        name="VCP Default",
        description="Reset all VCP codes to factory defaults",
        value_type=ValueType.WRITE_ONLY,
        min_value=0,
        max_value=0,
    ),
}


def get_vcp_info(code: int) -> VCPCodeInfo | None:
    """
    Get information about a VCP code.

    Args:
        code: The VCP code (0x00-0xFF)

    Returns:
        VCPCodeInfo object if the code is known, None otherwise
    """
    return VCP_CODES.get(code)


def get_vcp_name(code: int) -> str | None:
    """
    Get the human-readable name of a VCP code.

    Args:
        code: The VCP code (0x00-0xFF)

    Returns:
        The VCP code name if known, None otherwise
    """
    info = get_vcp_info(code)
    return info.name if info else None


def is_valid_vcp_code(code: int) -> bool:
    """
    Check if a VCP code is valid (in the known range).

    Args:
        code: The VCP code to check

    Returns:
        True if the code is valid, False otherwise
    """
    return 0x00 <= code <= 0xFF


def get_value_name(code: int, value: int) -> str | None:
    """
    Get the human-readable name for a non-continuous VCP code value.

    Args:
        code: The VCP code
        value: The VCP code value

    Returns:
        The human-readable name, or None if not found or not applicable
    """
    info = get_vcp_info(code)
    if info:
        return info.get_value_name(value)
    return None


def validate_vcp_value(code: int, value: int) -> bool:
    """
    Validate if a value is valid for a given VCP code.

    Args:
        code: The VCP code
        value: The value to validate

    Returns:
        True if the value is valid, False otherwise
    """
    info = get_vcp_info(code)
    if info:
        return info.validate_value(value)
    return False


def get_common_vcp_codes() -> list[int]:
    """
    Get a list of commonly supported VCP codes.

    Returns:
        List of VCP codes that are commonly supported across monitors
    """
    return [0x10, 0x12, 0x14, 0x60, 0x62, 0xD6]
