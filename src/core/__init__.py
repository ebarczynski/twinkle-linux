"""
Core application logic for Twinkle Linux.

This module contains the core application components including:
- Application controller (TwinkleApp)
- Configuration management
- Event bus implementation
- Custom exceptions
"""

from .app import TwinkleApp
from .config import (
    AppConfig,
    BehaviorConfig,
    ConfigurationManager,
    MonitorConfig,
    ShortcutsConfig,
    UIConfig,
)
from .logging import (
    disable_console_logging,
    enable_console_logging,
    get_logger,
    set_log_level,
    setup_logging,
)

__all__ = [
    "TwinkleApp",
    "AppConfig",
    "BehaviorConfig",
    "ConfigurationManager",
    "MonitorConfig",
    "ShortcutsConfig",
    "UIConfig",
    "setup_logging",
    "get_logger",
    "set_log_level",
    "disable_console_logging",
    "enable_console_logging",
]
