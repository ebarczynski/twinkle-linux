"""
Monitor detection and management for DDC/CI operations.

This module provides classes for representing physical monitors and detecting
connected monitors via DDC/CI protocol.
"""

from __future__ import annotations

import logging
import re
from dataclasses import dataclass, field
from datetime import datetime
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from collections.abc import Set

from src.ddc.command import CommandExecutor, CommandResult
from src.ddc.exceptions import CommandExecutionError, MonitorNotFoundError, VCPNotSupportedError
from src.ddc.vcp_codes import VCPCodeInfo, get_vcp_info

logger = logging.getLogger(__name__)


@dataclass
class MonitorCapabilities:
    """
    Capabilities of a monitor.

    Attributes:
        supported_vcp_codes: Set of VCP codes supported by the monitor
        max_brightness: Maximum brightness value
        max_contrast: Maximum contrast value
        supports_input_source: Whether input source selection is supported
        supports_power_control: Whether power control is supported
        supports_audio: Whether audio controls are supported
    """

    supported_vcp_codes: Set[int] = field(default_factory=set)
    max_brightness: int = 100
    max_contrast: int = 100
    supports_input_source: bool = False
    supports_power_control: bool = False
    supports_audio: bool = False

    def supports_vcp(self, vcp_code: int) -> bool:
        """Check if a VCP code is supported."""
        return vcp_code in self.supported_vcp_codes


@dataclass
class Monitor:
    """
    Represents a physical monitor connected to the system.

    Attributes:
        bus: I2C bus number for this monitor
        model: Monitor model name
        serial: Monitor serial number
        manufacturer: Monitor manufacturer name
        edid_data: Raw EDID data from the monitor
        capabilities: Monitor capabilities
        last_seen: Timestamp when this monitor was last detected
        _cached_values: Cached VCP values
    """

    bus: int
    model: str = "Unknown Monitor"
    serial: str = ""
    manufacturer: str = ""
    edid_data: str = ""
    capabilities: MonitorCapabilities = field(default_factory=MonitorCapabilities)
    last_seen: datetime = field(default_factory=datetime.now)
    _cached_values: dict[int, int] = field(default_factory=dict)

    def __post_init__(self) -> None:
        """Validate monitor data."""
        if self.bus < 0:
            raise ValueError(f"Invalid bus number: {self.bus}")

    @property
    def display_name(self) -> str:
        """Get a human-readable display name for this monitor."""
        if self.serial and self.serial != "Unknown":
            return f"{self.model} ({self.serial})"
        return self.model

    @property
    def unique_id(self) -> str:
        """Get a unique identifier for this monitor."""
        if self.serial and self.serial != "Unknown":
            return self.serial
        # Fallback to model + bus if serial is not available
        return f"{self.model}_bus{self.bus}"

    def get_cached_value(self, vcp_code: int) -> int | None:
        """
        Get a cached VCP value.

        Args:
            vcp_code: The VCP code to retrieve

        Returns:
            The cached value, or None if not cached
        """
        return self._cached_values.get(vcp_code)

    def set_cached_value(self, vcp_code: int, value: int) -> None:
        """
        Cache a VCP value.

        Args:
            vcp_code: The VCP code
            value: The value to cache
        """
        self._cached_values[vcp_code] = value

    def clear_cache(self) -> None:
        """Clear all cached VCP values."""
        self._cached_values.clear()

    def invalidate_cache(self, vcp_code: int) -> None:
        """
        Invalidate a specific cached VCP value.

        Args:
            vcp_code: The VCP code to invalidate
        """
        self._cached_values.pop(vcp_code, None)

    def to_dict(self) -> dict:
        """Convert monitor to dictionary for serialization."""
        return {
            "bus": self.bus,
            "model": self.model,
            "serial": self.serial,
            "manufacturer": self.manufacturer,
            "display_name": self.display_name,
            "unique_id": self.unique_id,
            "last_seen": self.last_seen.isoformat(),
            "capabilities": {
                "supported_vcp_codes": list(self.capabilities.supported_vcp_codes),
                "max_brightness": self.capabilities.max_brightness,
                "max_contrast": self.capabilities.max_contrast,
                "supports_input_source": self.capabilities.supports_input_source,
                "supports_power_control": self.capabilities.supports_power_control,
                "supports_audio": self.capabilities.supports_audio,
            },
        }

    @classmethod
    def from_dict(cls, data: dict) -> Monitor:
        """Create a Monitor from a dictionary."""
        caps_data = data.get("capabilities", {})
        capabilities = MonitorCapabilities(
            supported_vcp_codes=set(caps_data.get("supported_vcp_codes", [])),
            max_brightness=caps_data.get("max_brightness", 100),
            max_contrast=caps_data.get("max_contrast", 100),
            supports_input_source=caps_data.get("supports_input_source", False),
            supports_power_control=caps_data.get("supports_power_control", False),
            supports_audio=caps_data.get("supports_audio", False),
        )

        return cls(
            bus=data["bus"],
            model=data.get("model", "Unknown Monitor"),
            serial=data.get("serial", ""),
            manufacturer=data.get("manufacturer", ""),
            capabilities=capabilities,
        )


class MonitorDetector:
    """
    Detector for discovering and identifying connected monitors.

    This class uses ddcutil to scan for connected monitors, parse their
    EDID data, and build Monitor objects with their capabilities.
    """

    # Patterns for parsing ddcutil detect output
    BUS_PATTERN = re.compile(r"I2C bus:\s*/dev/i2c-(\d+)")
    EDID_MODEL_PATTERN = re.compile(r"Model:\s*(.+)")
    EDID_SERIAL_PATTERN = re.compile(r"Serial number:\s*(.+)")
    EDID_MFG_PATTERN = re.compile(r"Manufacturing ID:\s*(.+)")
    EDID_MFG_NAME_PATTERN = re.compile(r"Manufacturer ID:\s*(.+)")
    EDID_TYPE_PATTERN = re.compile(r"Display type:\s*(.+)")
    EDID_TEXT_PATTERN = re.compile(r"EDID text:\s*(.+)")
    VCP_INFO_PATTERN = re.compile(r"VCP Code\s+(0x[0-9A-Fa-f]{2})")

    def __init__(self, executor: CommandExecutor | None = None) -> None:
        """
        Initialize the MonitorDetector.

        Args:
            executor: CommandExecutor instance, or None to create a new one
        """
        self.executor = executor or CommandExecutor()
        self._detected_monitors: dict[str, Monitor] = {}

    def _parse_edid_info(self, output: str) -> dict[str, str]:
        """
        Parse EDID information from ddcutil detect output.

        Args:
            output: The output from ddcutil detect command

        Returns:
            Dictionary with parsed EDID fields
        """
        info = {
            "model": "Unknown Monitor",
            "serial": "",
            "manufacturer": "",
            "manufacturing_id": "",
            "display_type": "",
        }

        lines = output.split("\n")
        for line in lines:
            line = line.strip()

            match = self.EDID_MODEL_PATTERN.search(line)
            if match:
                info["model"] = match.group(1).strip()
                continue

            match = self.EDID_SERIAL_PATTERN.search(line)
            if match:
                info["serial"] = match.group(1).strip()
                continue

            match = self.EDID_MFG_PATTERN.search(line)
            if match:
                info["manufacturing_id"] = match.group(1).strip()
                continue

            match = self.EDID_MFG_NAME_PATTERN.search(line)
            if match:
                info["manufacturer"] = match.group(1).strip()
                continue

            match = self.EDID_TYPE_PATTERN.search(line)
            if match:
                info["display_type"] = match.group(1).strip()
                continue

            match = self.EDID_TEXT_PATTERN.search(line)
            if match:
                # Use EDID text as model if model is unknown
                if info["model"] == "Unknown Monitor":
                    info["model"] = match.group(1).strip()
                continue

        return info

    def _parse_vcp_capabilities(self, output: str) -> Set[int]:
        """
        Parse VCP capabilities from ddcutil vcpinfo output.

        Args:
            output: The output from ddcutil vcpinfo command

        Returns:
            Set of supported VCP codes
        """
        vcp_codes: Set[int] = set()
        for match in self.VCP_INFO_PATTERN.finditer(output):
            vcp_hex = match.group(1)
            try:
                vcp_code = int(vcp_hex, 16)
                vcp_codes.add(vcp_code)
            except ValueError:
                continue
        return vcp_codes

    def _detect_monitor_capabilities(self, bus: int) -> MonitorCapabilities:
        """
        Detect capabilities of a monitor on a specific bus.

        Args:
            bus: I2C bus number

        Returns:
            MonitorCapabilities object
        """
        capabilities = MonitorCapabilities()

        try:
            # Query VCP info
            result = self.executor.vcp_info(bus=bus, timeout=10.0)
            if result.success:
                vcp_codes = self._parse_vcp_capabilities(result.stdout)
                capabilities.supported_vcp_codes = vcp_codes

                # Check specific capabilities
                capabilities.supports_input_source = 0x60 in vcp_codes
                capabilities.supports_power_control = 0xD6 in vcp_codes
                capabilities.supports_audio = 0x62 in vcp_codes or 0x6C in vcp_codes

                # Get max brightness and contrast from VCP info
                # This is a simplified approach - real implementation would parse more carefully
                capabilities.max_brightness = 100
                capabilities.max_contrast = 100

            logger.debug(f"Detected capabilities for bus {bus}: {vcp_codes}")

        except CommandExecutionError as e:
            logger.warning(f"Failed to detect capabilities for bus {bus}: {e}")

        return capabilities

    def detect_monitors(self) -> list[Monitor]:
        """
        Detect all connected monitors.

        Returns:
            List of Monitor objects representing connected monitors

        Raises:
            CommandExecutionError: If detection fails
        """
        logger.info("Detecting connected monitors...")

        try:
            result = self.executor.detect_monitors(timeout=15.0)
        except CommandExecutionError as e:
            logger.error(f"Monitor detection failed: {e}")
            raise

        if not result.success:
            logger.error(f"Monitor detection command failed: {result.stderr}")
            raise CommandExecutionError(
                message="Monitor detection command failed",
                stderr=result.stderr,
            )

        # Parse the output to find all monitors
        monitors: list[Monitor] = []
        lines = result.stdout.split("\n")

        current_bus: int | None = None
        edid_section: list[str] = []

        for line in lines:
            # Check for I2C bus line
            bus_match = self.BUS_PATTERN.search(line)
            if bus_match:
                # Process previous monitor if any
                if current_bus is not None and edid_section:
                    monitor = self._create_monitor_from_edid(current_bus, "\n".join(edid_section))
                    monitors.append(monitor)

                # Start new monitor
                current_bus = int(bus_match.group(1))
                edid_section = [line]
            elif current_bus is not None:
                # Collect EDID section
                edid_section.append(line)

        # Process the last monitor
        if current_bus is not None and edid_section:
            monitor = self._create_monitor_from_edid(current_bus, "\n".join(edid_section))
            monitors.append(monitor)

        # Detect capabilities for each monitor
        for monitor in monitors:
            try:
                monitor.capabilities = self._detect_monitor_capabilities(monitor.bus)
            except CommandExecutionError:
                logger.warning(f"Failed to detect capabilities for monitor {monitor.display_name}")

        # Update internal cache
        self._detected_monitors = {m.unique_id: m for m in monitors}

        logger.info(f"Detected {len(monitors)} monitor(s)")
        for monitor in monitors:
            logger.info(f"  - {monitor.display_name} (bus {monitor.bus})")

        return monitors

    def _create_monitor_from_edid(self, bus: int, edid_output: str) -> Monitor:
        """
        Create a Monitor object from EDID output.

        Args:
            bus: I2C bus number
            edid_output: The EDID section from ddcutil detect output

        Returns:
            Monitor object
        """
        edid_info = self._parse_edid_info(edid_output)

        # Use model name, fallback to generic name
        model = edid_info["model"] if edid_info["model"] else f"Monitor on bus {bus}"

        # Use serial, fallback to empty string
        serial = edid_info["serial"] if edid_info["serial"] else ""

        # Use manufacturer name, fallback to manufacturing ID
        manufacturer = (
            edid_info["manufacturer"]
            if edid_info["manufacturer"]
            else edid_info.get("manufacturing_id", "")
        )

        return Monitor(
            bus=bus,
            model=model,
            serial=serial,
            manufacturer=manufacturer,
            edid_data=edid_output,
        )

    def find_monitor_by_bus(self, bus: int) -> Monitor | None:
        """
        Find a monitor by its I2C bus number.

        Args:
            bus: I2C bus number

        Returns:
            Monitor object if found, None otherwise
        """
        for monitor in self._detected_monitors.values():
            if monitor.bus == bus:
                return monitor
        return None

    def find_monitor_by_serial(self, serial: str) -> Monitor | None:
        """
        Find a monitor by its serial number.

        Args:
            serial: Monitor serial number

        Returns:
            Monitor object if found, None otherwise
        """
        return self._detected_monitors.get(serial)

    def find_monitor_by_unique_id(self, unique_id: str) -> Monitor | None:
        """
        Find a monitor by its unique ID.

        Args:
            unique_id: Monitor unique ID

        Returns:
            Monitor object if found, None otherwise
        """
        return self._detected_monitors.get(unique_id)

    def get_all_monitors(self) -> list[Monitor]:
        """
        Get all detected monitors.

        Returns:
            List of Monitor objects
        """
        return list(self._detected_monitors.values())

    def refresh(self) -> list[Monitor]:
        """
        Refresh the list of detected monitors.

        Returns:
            Updated list of Monitor objects
        """
        return self.detect_monitors()
