"""
UI module for Twinkle Linux.

This module provides the user interface components for Twinkle Linux,
including the system tray icon and brightness control widgets.
"""

from .brightness_popup import BrightnessPopup
from .ui_main import UIMain

__all__ = ["BrightnessPopup", "UIMain"]
