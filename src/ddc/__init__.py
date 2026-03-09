"""
DDC/CI abstraction layer for Twinkle Linux.

This module provides the DDC/CI communication layer including:
- DDCManager: Main interface for DDC/CI operations
- Monitor: Representation of a physical monitor
- MonitorDetector: Monitor detection and identification
- CommandExecutor: ddcutil subprocess management
- VCP code definitions and helper functions
- Custom exception classes for DDC/CI errors
"""

from src.ddc.command import CommandExecutor, CommandResult
from src.ddc.ddc_manager import DDCManager
from src.ddc.exceptions import (
    CommandExecutionError,
    DDCError,
    DDCNotAvailableError,
    InvalidValueError,
    MonitorNotFoundError,
    PermissionError,
    TimeoutError,
    VCPNotSupportedError,
)
from src.ddc.monitor import Monitor, MonitorCapabilities, MonitorDetector
from src.ddc.vcp_codes import (
    COLOR_TEMPERATURE_VALUES,
    DISPLAY_TECHNOLOGY_VALUES,
    INPUT_SOURCE_VALUES,
    POWER_MODE_VALUES,
    VCPCodeInfo,
    VCP_CODES,
    ValueType,
    get_common_vcp_codes,
    get_vcp_info,
    get_vcp_name,
    get_value_name,
    is_valid_vcp_code,
    validate_vcp_value,
)

__all__ = [
    # Main classes
    "DDCManager",
    "Monitor",
    "MonitorDetector",
    "MonitorCapabilities",
    "CommandExecutor",
    "CommandResult",
    # VCP codes
    "VCPCodeInfo",
    "VCP_CODES",
    "ValueType",
    "COLOR_TEMPERATURE_VALUES",
    "DISPLAY_TECHNOLOGY_VALUES",
    "INPUT_SOURCE_VALUES",
    "POWER_MODE_VALUES",
    # VCP helper functions
    "get_vcp_info",
    "get_vcp_name",
    "get_value_name",
    "is_valid_vcp_code",
    "validate_vcp_value",
    "get_common_vcp_codes",
    # Exceptions
    "DDCError",
    "DDCNotAvailableError",
    "MonitorNotFoundError",
    "VCPNotSupportedError",
    "PermissionError",
    "CommandExecutionError",
    "TimeoutError",
    "InvalidValueError",
]
