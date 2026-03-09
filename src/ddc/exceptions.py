"""
Custom exceptions for DDC/CI operations.

This module defines all custom exception classes used throughout the DDC/CI
backend for handling various error conditions that can occur during monitor
communication.
"""


class DDCError(Exception):
    """Base exception class for all DDC/CI related errors."""

    pass


class DDCNotAvailableError(DDCError):
    """
    Raised when ddcutil is not available or not installed on the system.

    This can occur when:
    - ddcutil is not installed
    - ddcutil is not in the system PATH
    - The ddcutil executable cannot be executed
    """

    def __init__(self, message: str = "ddcutil is not available on this system") -> None:
        super().__init__(message)


class MonitorNotFoundError(DDCError):
    """
    Raised when a specified monitor cannot be found.

    This can occur when:
    - A monitor with the given bus number does not exist
    - A monitor with the given serial number is not connected
    - The monitor has been disconnected since last detection
    """

    def __init__(
        self,
        message: str = "Monitor not found",
        bus: int | None = None,
        serial: str | None = None,
    ) -> None:
        self.bus = bus
        self.serial = serial
        if bus is not None:
            message = f"Monitor on bus {bus} not found"
        elif serial is not None:
            message = f"Monitor with serial {serial} not found"
        super().__init__(message)


class VCPNotSupportedError(DDCError):
    """
    Raised when a VCP code is not supported by the monitor.

    This can occur when:
    - The monitor does not implement the requested VCP code
    - The VCP code is read-only or write-only when the opposite is requested
    - The VCP code value is outside the supported range
    """

    def __init__(
        self,
        message: str = "VCP code not supported",
        vcp_code: int | None = None,
    ) -> None:
        self.vcp_code = vcp_code
        if vcp_code is not None:
            message = f"VCP code 0x{vcp_code:02X} is not supported by this monitor"
        super().__init__(message)


class PermissionError(DDCError):
    """
    Raised when there are insufficient permissions to access I2C devices.

    This can occur when:
    - The user is not in the i2c group
    - The I2C device permissions are too restrictive
    - udev rules are not properly configured
    """

    def __init__(
        self,
        message: str = "Insufficient permissions to access I2C devices",
        bus: int | None = None,
    ) -> None:
        self.bus = bus
        if bus is not None:
            message = (
                f"Insufficient permissions to access I2C device on bus {bus}. "
                "Add your user to the i2c group or configure udev rules."
            )
        super().__init__(message)


class CommandExecutionError(DDCError):
    """
    Raised when a ddcutil command fails to execute properly.

    This can occur when:
    - The command times out
    - The command returns a non-zero exit code
    - The command output cannot be parsed
    - The monitor does not respond to the command
    """

    def __init__(
        self,
        message: str = "DDC/CI command execution failed",
        command: str | None = None,
        exit_code: int | None = None,
        stderr: str | None = None,
    ) -> None:
        self.command = command
        self.exit_code = exit_code
        self.stderr = stderr
        super().__init__(message)

    def __str__(self) -> str:
        parts = [super().__str__()]
        if self.command:
            parts.append(f"Command: {self.command}")
        if self.exit_code is not None:
            parts.append(f"Exit code: {self.exit_code}")
        if self.stderr:
            parts.append(f"Error output: {self.stderr}")
        return " | ".join(parts)


class TimeoutError(DDCError):
    """
    Raised when a DDC/CI operation times out.

    Monitors can take varying amounts of time to respond to DDC/CI commands.
    This exception is raised when the configured timeout is exceeded.
    """

    def __init__(
        self,
        message: str = "DDC/CI operation timed out",
        timeout_seconds: float | None = None,
    ) -> None:
        self.timeout_seconds = timeout_seconds
        if timeout_seconds is not None:
            message = f"DDC/CI operation timed out after {timeout_seconds} seconds"
        super().__init__(message)


class InvalidValueError(DDCError):
    """
    Raised when an invalid value is provided for a VCP code.

    This can occur when:
    - The value is outside the valid range for the VCP code
    - The value is of the wrong type
    - The value is not a valid enum value for the VCP code
    """

    def __init__(
        self,
        message: str = "Invalid value for VCP code",
        vcp_code: int | None = None,
        value: int | None = None,
        valid_range: tuple[int, int] | None = None,
    ) -> None:
        self.vcp_code = vcp_code
        self.value = value
        self.valid_range = valid_range
        if vcp_code is not None and value is not None:
            message = f"Invalid value {value} for VCP code 0x{vcp_code:02X}"
            if valid_range:
                message += f" (valid range: {valid_range[0]}-{valid_range[1]})"
        super().__init__(message)
