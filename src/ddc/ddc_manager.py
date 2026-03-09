"""
DDC/CI Manager - Main interface for DDC/CI operations.

This module provides the DDCManager class, which serves as the main
interface for all DDC/CI operations including monitor detection,
brightness control, and VCP value management.
"""

from __future__ import annotations

import logging
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from collections.abc import Callable

from src.ddc.command import CommandExecutor, CommandResult
from src.ddc.exceptions import (
    DDCNotAvailableError,
    MonitorNotFoundError,
    PermissionError,
    TimeoutError,
    VCPNotSupportedError,
)
from src.ddc.monitor import Monitor, MonitorDetector, MonitorCapabilities
from src.ddc.vcp_codes import get_vcp_info, get_common_vcp_codes

logger = logging.getLogger(__name__)


class DDCManager:
    """
    Main interface for DDC/CI operations.

    This class provides a high-level API for interacting with monitors
    via the DDC/CI protocol. It handles monitor detection, VCP value
    reading and writing, and provides caching for improved performance.
    """

    # Default configuration
    DEFAULT_CACHE_TTL = 5.0  # seconds - time to live for cached values
    DEFAULT_BRIGHTNESS_MIN = 0
    DEFAULT_BRIGHTNESS_MAX = 100

    def __init__(
        self,
        executor: CommandExecutor | None = None,
        cache_ttl: float = DEFAULT_CACHE_TTL,
    ) -> None:
        """
        Initialize the DDCManager.

        Args:
            executor: CommandExecutor instance, or None to create a new one
            cache_ttl: Time-to-live for cached VCP values in seconds
        """
        self.executor = executor or CommandExecutor()
        self.detector = MonitorDetector(self.executor)
        self.cache_ttl = cache_ttl

        self._monitors: dict[str, Monitor] = {}
        self._initialized = False

    def initialize(self) -> bool:
        """
        Initialize the DDCManager and detect available monitors.

        Returns:
            True if initialization succeeded, False otherwise
        """
        logger.info("Initializing DDCManager...")

        # Check if ddcutil is available
        if not self.executor.check_ddcutil_available():
            logger.error("ddcutil is not available on this system")
            return False

        # Detect monitors
        try:
            monitors = self.detector.detect_monitors()
            self._monitors = {m.unique_id: m for m in monitors}
            self._initialized = True

            logger.info(f"DDCManager initialized with {len(monitors)} monitor(s)")
            return True

        except Exception as e:
            logger.error(f"Failed to initialize DDCManager: {e}")
            return False

    def is_available(self) -> bool:
        """
        Check if DDC/CI is available on the system.

        Returns:
            True if ddcutil is available, False otherwise
        """
        return self.executor.check_ddcutil_available()

    def check_permissions(self, bus: int | None = None) -> bool:
        """
        Check if the user has permission to access I2C devices.

        Args:
            bus: I2C bus number to check, or None to check default

        Returns:
            True if permissions are OK, False otherwise
        """
        try:
            # Try to query a common VCP code to check permissions
            result = self.executor.get_vcp(bus=bus, vcp_code=0x10, timeout=2.0)
            return result.success or "Permission denied" not in result.stderr
        except PermissionError:
            return False
        except Exception:
            # Other exceptions might indicate other issues, but not permission
            return True

    def get_monitors(self) -> list[Monitor]:
        """
        Get all detected monitors.

        Returns:
            List of Monitor objects
        """
        if not self._initialized:
            self.initialize()
        return list(self._monitors.values())

    def get_monitor_by_bus(self, bus: int) -> Monitor:
        """
        Get a monitor by its I2C bus number.

        Args:
            bus: I2C bus number

        Returns:
            Monitor object

        Raises:
            MonitorNotFoundError: If no monitor is found on the bus
        """
        for monitor in self._monitors.values():
            if monitor.bus == bus:
                return monitor
        raise MonitorNotFoundError(bus=bus)

    def get_monitor_by_serial(self, serial: str) -> Monitor:
        """
        Get a monitor by its serial number.

        Args:
            serial: Monitor serial number

        Returns:
            Monitor object

        Raises:
            MonitorNotFoundError: If no monitor is found with the serial
        """
        monitor = self._monitors.get(serial)
        if monitor is None:
            raise MonitorNotFoundError(serial=serial)
        return monitor

    def get_monitor_by_unique_id(self, unique_id: str) -> Monitor:
        """
        Get a monitor by its unique ID.

        Args:
            unique_id: Monitor unique ID

        Returns:
            Monitor object

        Raises:
            MonitorNotFoundError: If no monitor is found with the unique ID
        """
        monitor = self._monitors.get(unique_id)
        if monitor is None:
            raise MonitorNotFoundError()
        return monitor

    def refresh_monitors(self) -> list[Monitor]:
        """
        Refresh the list of detected monitors.

        Returns:
            Updated list of Monitor objects
        """
        logger.info("Refreshing monitor list...")
        monitors = self.detector.refresh()
        self._monitors = {m.unique_id: m for m in monitors}
        return monitors

    def get_vcp(
        self,
        monitor: Monitor | int | str,
        vcp_code: int,
        use_cache: bool = True,
        timeout: float | None = None,
    ) -> int:
        """
        Get the current value of a VCP code for a monitor.

        Args:
            monitor: Monitor object, bus number, serial, or unique ID
            vcp_code: VCP code to query
            use_cache: Whether to use cached values if available
            timeout: Timeout in seconds, or None to use default

        Returns:
            Current value of the VCP code

        Raises:
            MonitorNotFoundError: If the monitor is not found
            VCPNotSupportedError: If the VCP code is not supported
            CommandExecutionError: If the command fails
        """
        # Resolve monitor
        if isinstance(monitor, Monitor):
            monitor_obj = monitor
        elif isinstance(monitor, int):
            monitor_obj = self.get_monitor_by_bus(monitor)
        else:
            monitor_obj = self.get_monitor_by_unique_id(monitor)

        # Check if VCP code is supported
        if not monitor_obj.capabilities.supports_vcp(vcp_code):
            vcp_info = get_vcp_info(vcp_code)
            raise VCPNotSupportedError(
                vcp_code=vcp_code,
                message=f"VCP code 0x{vcp_code:02X} ({vcp_info.name if vcp_info else 'Unknown'}) "
                f"is not supported by {monitor_obj.display_name}",
            )

        # Check cache first
        if use_cache:
            cached = monitor_obj.get_cached_value(vcp_code)
            if cached is not None:
                logger.debug(f"Using cached value for VCP 0x{vcp_code:02X}: {cached}")
                return cached

        # Query the monitor
        logger.debug(f"Querying VCP 0x{vcp_code:02X} for {monitor_obj.display_name}")
        result = self.executor.get_vcp(bus=monitor_obj.bus, vcp_code=vcp_code, timeout=timeout)

        if not result.success:
            raise CommandExecutionError(
                message=f"Failed to get VCP 0x{vcp_code:02X} value",
                command=result.command,
                exit_code=result.exit_code,
                stderr=result.stderr,
            )

        if result.value is None:
            raise CommandExecutionError(
                message=f"Failed to parse VCP 0x{vcp_code:02X} value from output"
            )

        # Cache the value
        monitor_obj.set_cached_value(vcp_code, result.value)

        return result.value

    def set_vcp(
        self,
        monitor: Monitor | int | str,
        vcp_code: int,
        value: int,
        timeout: float | None = None,
    ) -> bool:
        """
        Set the value of a VCP code for a monitor.

        Args:
            monitor: Monitor object, bus number, serial, or unique ID
            vcp_code: VCP code to set
            value: Value to set
            timeout: Timeout in seconds, or None to use default

        Returns:
            True if successful

        Raises:
            MonitorNotFoundError: If the monitor is not found
            VCPNotSupportedError: If the VCP code is not supported
            CommandExecutionError: If the command fails
        """
        # Resolve monitor
        if isinstance(monitor, Monitor):
            monitor_obj = monitor
        elif isinstance(monitor, int):
            monitor_obj = self.get_monitor_by_bus(monitor)
        else:
            monitor_obj = self.get_monitor_by_unique_id(monitor)

        # Check if VCP code is supported
        if not monitor_obj.capabilities.supports_vcp(vcp_code):
            vcp_info = get_vcp_info(vcp_code)
            raise VCPNotSupportedError(
                vcp_code=vcp_code,
                message=f"VCP code 0x{vcp_code:02X} ({vcp_info.name if vcp_info else 'Unknown'}) "
                f"is not supported by {monitor_obj.display_name}",
            )

        # Validate value
        vcp_info = get_vcp_info(vcp_code)
        if vcp_info and not vcp_info.validate_value(value):
            raise ValueError(
                f"Invalid value {value} for VCP 0x{vcp_code:02X} "
                f"(valid range: {vcp_info.min_value}-{vcp_info.max_value})"
            )

        # Set the value
        logger.debug(
            f"Setting VCP 0x{vcp_code:02X} to {value} for {monitor_obj.display_name}"
        )
        result = self.executor.set_vcp(
            bus=monitor_obj.bus, vcp_code=vcp_code, value=value, timeout=timeout
        )

        if not result.success:
            raise CommandExecutionError(
                message=f"Failed to set VCP 0x{vcp_code:02X} to {value}",
                command=result.command,
                exit_code=result.exit_code,
                stderr=result.stderr,
            )

        # Update cache
        monitor_obj.set_cached_value(vcp_code, value)

        return True

    def get_brightness(
        self,
        monitor: Monitor | int | str,
        use_cache: bool = True,
        timeout: float | None = None,
    ) -> int:
        """
        Get the current brightness level for a monitor.

        Args:
            monitor: Monitor object, bus number, serial, or unique ID
            use_cache: Whether to use cached values if available
            timeout: Timeout in seconds, or None to use default

        Returns:
            Current brightness value (0-100)

        Raises:
            MonitorNotFoundError: If the monitor is not found
            VCPNotSupportedError: If brightness is not supported
            CommandExecutionError: If the command fails
        """
        return self.get_vcp(monitor, 0x10, use_cache=use_cache, timeout=timeout)

    def set_brightness(
        self,
        monitor: Monitor | int | str,
        value: int,
        timeout: float | None = None,
    ) -> bool:
        """
        Set the brightness level for a monitor.

        Args:
            monitor: Monitor object, bus number, serial, or unique ID
            value: Brightness value (0-100)
            timeout: Timeout in seconds, or None to use default

        Returns:
            True if successful

        Raises:
            MonitorNotFoundError: If the monitor is not found
            VCPNotSupportedError: If brightness is not supported
            CommandExecutionError: If the command fails
        """
        # Normalize brightness to 0-100 range
        value = max(self.DEFAULT_BRIGHTNESS_MIN, min(self.DEFAULT_BRIGHTNESS_MAX, value))
        return self.set_vcp(monitor, 0x10, value, timeout=timeout)

    def adjust_brightness(
        self,
        monitor: Monitor | int | str,
        delta: int,
        timeout: float | None = None,
    ) -> int:
        """
        Adjust the brightness level for a monitor by a delta.

        Args:
            monitor: Monitor object, bus number, serial, or unique ID
            delta: Amount to adjust (positive or negative)
            timeout: Timeout in seconds, or None to use default

        Returns:
            New brightness value (0-100)

        Raises:
            MonitorNotFoundError: If the monitor is not found
            VCPNotSupportedError: If brightness is not supported
            CommandExecutionError: If the command fails
        """
        current = self.get_brightness(monitor, use_cache=False, timeout=timeout)
        new_value = current + delta
        new_value = max(self.DEFAULT_BRIGHTNESS_MIN, min(self.DEFAULT_BRIGHTNESS_MAX, new_value))
        self.set_brightness(monitor, new_value, timeout=timeout)
        return new_value

    def get_contrast(
        self,
        monitor: Monitor | int | str,
        use_cache: bool = True,
        timeout: float | None = None,
    ) -> int:
        """
        Get the current contrast level for a monitor.

        Args:
            monitor: Monitor object, bus number, serial, or unique ID
            use_cache: Whether to use cached values if available
            timeout: Timeout in seconds, or None to use default

        Returns:
            Current contrast value (0-100)

        Raises:
            MonitorNotFoundError: If the monitor is not found
            VCPNotSupportedError: If contrast is not supported
            CommandExecutionError: If the command fails
        """
        return self.get_vcp(monitor, 0x12, use_cache=use_cache, timeout=timeout)

    def set_contrast(
        self,
        monitor: Monitor | int | str,
        value: int,
        timeout: float | None = None,
    ) -> bool:
        """
        Set the contrast level for a monitor.

        Args:
            monitor: Monitor object, bus number, serial, or unique ID
            value: Contrast value (0-100)
            timeout: Timeout in seconds, or None to use default

        Returns:
            True if successful

        Raises:
            MonitorNotFoundError: If the monitor is not found
            VCPNotSupportedError: If contrast is not supported
            CommandExecutionError: If the command fails
        """
        value = max(0, min(100, value))
        return self.set_vcp(monitor, 0x12, value, timeout=timeout)

    def get_volume(
        self,
        monitor: Monitor | int | str,
        use_cache: bool = True,
        timeout: float | None = None,
    ) -> int:
        """
        Get the current volume level for a monitor.

        Args:
            monitor: Monitor object, bus number, serial, or unique ID
            use_cache: Whether to use cached values if available
            timeout: Timeout in seconds, or None to use default

        Returns:
            Current volume value (0-100)

        Raises:
            MonitorNotFoundError: If the monitor is not found
            VCPNotSupportedError: If volume is not supported
            CommandExecutionError: If the command fails
        """
        return self.get_vcp(monitor, 0x62, use_cache=use_cache, timeout=timeout)

    def set_volume(
        self,
        monitor: Monitor | int | str,
        value: int,
        timeout: float | None = None,
    ) -> bool:
        """
        Set the volume level for a monitor.

        Args:
            monitor: Monitor object, bus number, serial, or unique ID
            value: Volume value (0-100)
            timeout: Timeout in seconds, or None to use default

        Returns:
            True if successful

        Raises:
            MonitorNotFoundError: If the monitor is not found
            VCPNotSupportedError: If volume is not supported
            CommandExecutionError: If the command fails
        """
        value = max(0, min(100, value))
        return self.set_vcp(monitor, 0x62, value, timeout=timeout)

    def get_input_source(
        self,
        monitor: Monitor | int | str,
        use_cache: bool = True,
        timeout: float | None = None,
    ) -> int:
        """
        Get the current input source for a monitor.

        Args:
            monitor: Monitor object, bus number, serial, or unique ID
            use_cache: Whether to use cached values if available
            timeout: Timeout in seconds, or None to use default

        Returns:
            Current input source value

        Raises:
            MonitorNotFoundError: If the monitor is not found
            VCPNotSupportedError: If input source is not supported
            CommandExecutionError: If the command fails
        """
        return self.get_vcp(monitor, 0x60, use_cache=use_cache, timeout=timeout)

    def set_input_source(
        self,
        monitor: Monitor | int | str,
        value: int,
        timeout: float | None = None,
    ) -> bool:
        """
        Set the input source for a monitor.

        Args:
            monitor: Monitor object, bus number, serial, or unique ID
            value: Input source value
            timeout: Timeout in seconds, or None to use default

        Returns:
            True if successful

        Raises:
            MonitorNotFoundError: If the monitor is not found
            VCPNotSupportedError: If input source is not supported
            CommandExecutionError: If the command fails
        """
        return self.set_vcp(monitor, 0x60, value, timeout=timeout)

    def get_power_mode(
        self,
        monitor: Monitor | int | str,
        use_cache: bool = True,
        timeout: float | None = None,
    ) -> int:
        """
        Get the current power mode for a monitor.

        Args:
            monitor: Monitor object, bus number, serial, or unique ID
            use_cache: Whether to use cached values if available
            timeout: Timeout in seconds, or None to use default

        Returns:
            Current power mode value

        Raises:
            MonitorNotFoundError: If the monitor is not found
            VCPNotSupportedError: If power mode is not supported
            CommandExecutionError: If the command fails
        """
        return self.get_vcp(monitor, 0xD6, use_cache=use_cache, timeout=timeout)

    def set_power_mode(
        self,
        monitor: Monitor | int | str,
        value: int,
        timeout: float | None = None,
    ) -> bool:
        """
        Set the power mode for a monitor.

        Args:
            monitor: Monitor object, bus number, serial, or unique ID
            value: Power mode value (0=On, 1=Standby, 2=Suspend, 3=Off)
            timeout: Timeout in seconds, or None to use default

        Returns:
            True if successful

        Raises:
            MonitorNotFoundError: If the monitor is not found
            VCPNotSupportedError: If power mode is not supported
            CommandExecutionError: If the command fails
        """
        return self.set_vcp(monitor, 0xD6, value, timeout=timeout)

    def get_capabilities(self, monitor: Monitor | int | str) -> MonitorCapabilities:
        """
        Get the capabilities of a monitor.

        Args:
            monitor: Monitor object, bus number, serial, or unique ID

        Returns:
            MonitorCapabilities object

        Raises:
            MonitorNotFoundError: If the monitor is not found
        """
        if isinstance(monitor, Monitor):
            return monitor.capabilities
        elif isinstance(monitor, int):
            monitor_obj = self.get_monitor_by_bus(monitor)
        else:
            monitor_obj = self.get_monitor_by_unique_id(monitor)
        return monitor_obj.capabilities

    def clear_cache(self, monitor: Monitor | int | str | None = None) -> None:
        """
        Clear cached VCP values.

        Args:
            monitor: Monitor object, bus number, serial, or unique ID.
                     If None, clears cache for all monitors.
        """
        if monitor is None:
            for m in self._monitors.values():
                m.clear_cache()
        elif isinstance(monitor, Monitor):
            monitor.clear_cache()
        elif isinstance(monitor, int):
            monitor_obj = self.get_monitor_by_bus(monitor)
            monitor_obj.clear_cache()
        else:
            monitor_obj = self.get_monitor_by_unique_id(monitor)
            monitor_obj.clear_cache()
