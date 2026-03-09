"""
UI Main Module.

This module provides the UIMain class, which serves as the main UI coordinator
for Twinkle Linux, integrating the system tray icon and brightness popup.
"""

from __future__ import annotations

import logging
from pathlib import Path
from typing import TYPE_CHECKING, Optional

from PyQt6.QtCore import QObject, QPoint, pyqtSignal
from PyQt6.QtGui import QIcon
from PyQt6.QtWidgets import QSystemTrayIcon, QMenu, QAction, QApplication

if TYPE_CHECKING:
    from collections.abc import Callable

from .brightness_popup import BrightnessPopup
from src.ddc.exceptions import VCPNotSupportedError

logger = logging.getLogger(__name__)


class UIMain(QObject):
    """
    Main UI coordinator for Twinkle Linux.

    This class manages the system tray icon and brightness popup,
    handling UI events and coordinating with the DDC backend.
    """

    # Signals for UI events
    brightness_changed = pyqtSignal(int, str)  # value, monitor_id
    vcp_changed = pyqtSignal(int, int, str)  # vcp_code, value, monitor_id
    monitor_selected = pyqtSignal(str)  # monitor_id
    settings_requested = pyqtSignal()
    quit_requested = pyqtSignal()

    def __init__(
        self,
        parent: QObject | None = None,
        icon_path: str | Path | None = None,
        stylesheet_path: str | Path | None = None,
        config_manager=None,
    ) -> None:
        """
        Initialize the UI main coordinator.

        Args:
            parent: Parent QObject
            icon_path: Path to application icon (optional)
            stylesheet_path: Path to stylesheet file (optional)
            config_manager: Configuration manager instance (optional)
        """
        super().__init__(parent)

        self._icon_path = Path(icon_path) if icon_path else None
        self._stylesheet_path = Path(stylesheet_path) if stylesheet_path else None
        self._config_manager = config_manager
        self._ddc_manager = None
        self._monitors: dict[str, str] = {}  # unique_id -> display_name
        self._current_monitor: str | None = None
        self._settings_dialog: Optional["SettingsDialog"] = None

        # Create tray icon
        self._tray_icon = QSystemTrayIcon(self)
        self._tray_icon.activated.connect(self._on_tray_activated)

        # Determine initial auto-hide time from config if available
        auto_hide_ms = 3000
        if self._config_manager:
            try:
                config = self._config_manager.get()
                auto_hide_ms = config.behavior.response_timeout_ms
            except Exception:
                pass

        # Create brightness popup
        self._brightness_popup = BrightnessPopup(
            parent=None,
            auto_hide_ms=auto_hide_ms,
            show_monitor_selector=True,  # Enable monitor selector for multi-monitor support
        )
        self._brightness_popup.brightness_changed.connect(self._on_brightness_changed)
        self._brightness_popup.monitor_changed.connect(self._on_monitor_changed)
        self._brightness_popup.vcp_changed.connect(self._on_vcp_changed)

        # Apply stylesheet if provided
        if self._stylesheet_path and self._stylesheet_path.exists():
            self._apply_stylesheet()

        # Set up tray icon
        self._setup_tray_icon()

    def _setup_tray_icon(self) -> None:
        """Set up the system tray icon and context menu."""
        # Set icon
        icon = self._get_icon()
        self._tray_icon.setIcon(icon)

        # Create context menu
        context_menu = QMenu()

        # Show settings action
        settings_action = QAction("Settings", self)
        settings_action.triggered.connect(self._on_settings_requested)
        context_menu.addAction(settings_action)

        # Separator
        context_menu.addSeparator()

        # Quit action
        quit_action = QAction("Quit", self)
        quit_action.triggered.connect(self._on_quit_requested)
        context_menu.addAction(quit_action)

        # Set context menu
        self._tray_icon.setContextMenu(context_menu)

        # Set tooltip
        self._tray_icon.setToolTip("Twinkle Linux - Monitor Brightness Control")

    def _get_icon(self) -> QIcon:
        """
        Get the application icon.

        Returns:
            QIcon instance
        """
        if self._icon_path and self._icon_path.exists():
            return QIcon(str(self._icon_path))

        # Use default Qt icon as fallback
        return QApplication.style().standardIcon(
            QApplication.Style.StandardPixmap.SP_ComputerIcon
        )

    def _apply_stylesheet(self) -> None:
        """Apply the stylesheet to the application."""
        if self._stylesheet_path and self._stylesheet_path.exists():
            try:
                with open(self._stylesheet_path, "r", encoding="utf-8") as f:
                    stylesheet = f.read()
                QApplication.instance().setStyleSheet(stylesheet)
                logger.info(f"Applied stylesheet from {self._stylesheet_path}")
            except Exception as e:
                logger.error(f"Failed to apply stylesheet: {e}")

    def _on_tray_activated(self, reason: QSystemTrayIcon.ActivationReason) -> None:
        """
        Handle tray icon activation.

        Args:
            reason: Activation reason
        """
        if reason == QSystemTrayIcon.ActivationReason.Trigger:
            # Left click - show brightness popup
            self._show_brightness_popup()
        elif reason == QSystemTrayIcon.ActivationReason.Context:
            # Right click - context menu is shown automatically
            pass

    def _show_brightness_popup(self) -> None:
        """Show the brightness popup at the tray icon position."""
        # Get tray icon geometry
        tray_geometry = self._tray_icon.geometry()

        if tray_geometry.isEmpty():
            # Fallback to screen center
            screen = QApplication.primaryScreen()
            screen_geometry = screen.availableGeometry()
            pos = screen_geometry.center()
            pos.setX(pos.x() - self._brightness_popup.width() // 2)
            pos.setY(pos.y() - self._brightness_popup.height() // 2)
        else:
            # Position popup above the tray icon
            pos = tray_geometry.topLeft()
            pos.setY(pos.y() - self._brightness_popup.height() - 10)

        # Update popup with current values
        self._update_popup_values()

        # Show popup
        self._brightness_popup.show_at(pos)

    def _update_popup_values(self) -> None:
        """Update the popup with the current VCP values."""
        if self._ddc_manager and self._current_monitor:
            try:
                # Update brightness
                brightness = self._ddc_manager.get_brightness(
                    self._current_monitor, use_cache=True
                )
                self._brightness_popup.set_brightness(brightness)
                
                # Update other VCP values
                enabled_vcp_codes = self._brightness_popup.get_enabled_vcp_codes()
                for vcp_code in enabled_vcp_codes:
                    if vcp_code == 0x10:  # Skip brightness, already handled
                        continue
                    try:
                        value = self._ddc_manager.get_vcp(
                            self._current_monitor, vcp_code, use_cache=True
                        )
                        self._brightness_popup.set_vcp_value(vcp_code, value)
                    except VCPNotSupportedError:
                        # Monitor doesn't support this VCP code, skip it
                        logger.debug(f"VCP code 0x{vcp_code:02X} not supported by monitor {self._current_monitor}")
                    except Exception as e:
                        logger.warning(f"Failed to get VCP 0x{vcp_code:02X} for monitor {self._current_monitor}: {e}")
                        
                self._brightness_popup.clear_status()
            except Exception as e:
                logger.warning(f"Failed to get current values for monitor {self._current_monitor}: {e}")
                self._brightness_popup.set_status(
                    f"Could not read monitor values: {e}",
                    is_error=True
                )

    def _on_brightness_changed(self, value: int) -> None:
        """
        Handle brightness change from popup.

        Args:
            value: New brightness value
        """
        monitor_id = self._current_monitor or ""
        logger.info(f"Brightness changed to {value} for monitor {monitor_id}")
        self.brightness_changed.emit(value, monitor_id)

        # Update DDC if manager is available
        if self._ddc_manager and self._current_monitor:
            # Check if monitor still exists
            if self._current_monitor not in self._monitors:
                logger.warning(f"Monitor {self._current_monitor} not available, refreshing monitors")
                self._refresh_monitors()
                # Try to set brightness on the new monitor
                if self._current_monitor:
                    self._set_brightness(value, self._current_monitor)
            else:
                self._set_brightness(value, self._current_monitor)
        else:
            logger.warning("No DDC manager or monitor available")
            self._brightness_popup.set_status(
                "No monitor available for brightness control",
                is_error=True
            )

    def _on_monitor_changed(self, monitor_id: str) -> None:
        """
        Handle monitor selection change.

        Args:
            monitor_id: Selected monitor unique ID
        """
        logger.info(f"Monitor selected: {monitor_id}")
        self._current_monitor = monitor_id
        self.monitor_selected.emit(monitor_id)

        # Update popup with current values for the new monitor
        self._update_popup_values()

        # Clear any previous status messages
        self._brightness_popup.clear_status()

    def _on_vcp_changed(self, vcp_code: int, value: int) -> None:
        """
        Handle VCP value change from popup.

        Args:
            vcp_code: The VCP code that changed
            value: New value
        """
        monitor_id = self._current_monitor or ""
        logger.info(f"VCP 0x{vcp_code:02X} changed to {value} for monitor {monitor_id}")
        self.vcp_changed.emit(vcp_code, value, monitor_id)

        # Update DDC if manager is available
        if self._ddc_manager and self._current_monitor:
            # Check if monitor still exists
            if self._current_monitor not in self._monitors:
                logger.warning(f"Monitor {self._current_monitor} not available, refreshing monitors")
                self._refresh_monitors()
                # Try to set VCP value on the new monitor
                if self._current_monitor:
                    self._set_vcp(vcp_code, value, self._current_monitor)
            else:
                self._set_vcp(vcp_code, value, self._current_monitor)
        else:
            logger.warning("No DDC manager or monitor available")
            self._brightness_popup.set_status(
                "No monitor available for VCP control",
                is_error=True
            )

    def _on_settings_requested(self) -> None:
        """Handle settings menu action."""
        logger.info("Settings requested")
        self.show_settings()
        self.settings_requested.emit()

    def _on_quit_requested(self) -> None:
        """Handle quit menu action."""
        logger.info("Quit requested")
        self.quit_requested.emit()

    def _set_brightness(self, value: int, monitor_id: str) -> bool:
        """
        Set brightness for a monitor.

        Args:
            value: Brightness value (0-100)
            monitor_id: Monitor unique ID

        Returns:
            True if successful, False otherwise
        """
        if not self._ddc_manager:
            logger.error("DDC manager not available")
            self._brightness_popup.set_status(
                "DDC manager not available",
                is_error=True
            )
            return False

        try:
            self._ddc_manager.set_brightness(monitor_id, value)
            self._brightness_popup.clear_status()
            return True
        except Exception as e:
            logger.error(f"Failed to set brightness for monitor {monitor_id}: {e}")
            self._brightness_popup.set_status(
                f"Failed to set brightness: {e}",
                is_error=True
            )
            return False

    def _set_vcp(self, vcp_code: int, value: int, monitor_id: str) -> bool:
        """
        Set a VCP value for a monitor.

        Args:
            vcp_code: The VCP code to set
            value: The value to set
            monitor_id: Monitor unique ID

        Returns:
            True if successful, False otherwise
        """
        if not self._ddc_manager:
            logger.error("DDC manager not available")
            self._brightness_popup.set_status(
                "DDC manager not available",
                is_error=True
            )
            return False

        try:
            self._ddc_manager.set_vcp(monitor_id, vcp_code, value)
            self._brightness_popup.clear_status()
            return True
        except VCPNotSupportedError as e:
            logger.warning(f"VCP code 0x{vcp_code:02X} not supported by monitor {monitor_id}: {e}")
            self._brightness_popup.set_status(
                f"VCP 0x{vcp_code:02X} not supported by this monitor",
                is_error=False
            )
            return False
        except Exception as e:
            logger.error(f"Failed to set VCP 0x{vcp_code:02X} for monitor {monitor_id}: {e}")
            self._brightness_popup.set_status(
                f"Failed to set VCP value: {e}",
                is_error=True
            )
            return False

    def set_ddc_manager(self, ddc_manager) -> None:
        """
        Set the DDC manager.

        Args:
            ddc_manager: DDCManager instance
        """
        self._ddc_manager = ddc_manager
        self._refresh_monitors()

    def _refresh_monitors(self) -> None:
        """Refresh the list of available monitors."""
        if not self._ddc_manager:
            return

        try:
            monitors = self._ddc_manager.get_monitors()
            self._monitors = {
                m.unique_id: m.display_name for m in monitors
            }

            # Handle monitor disconnection: check if current monitor still exists
            if self._current_monitor and self._current_monitor not in self._monitors:
                logger.warning(f"Current monitor {self._current_monitor} disconnected")
                self._current_monitor = None
                self._brightness_popup.set_status(
                    "Monitor disconnected, selecting another monitor...",
                    is_error=False
                )

            # Set current monitor to first available if not set or if previous was disconnected
            if not self._current_monitor and self._monitors:
                self._current_monitor = next(iter(self._monitors))
                logger.info(f"Selected new monitor: {self._current_monitor}")

            # Update popup with monitors
            self._brightness_popup.set_monitors(self._monitors)

            # Set current monitor in popup
            if self._current_monitor:
                self._brightness_popup.set_current_monitor(self._current_monitor)

            # Show/hide monitor selector based on monitor count
            if len(self._monitors) > 1:
                self._brightness_popup.set_monitor_selector_visible(True)
            else:
                self._brightness_popup.set_monitor_selector_visible(False)

            logger.info(f"Refreshed monitors: {len(self._monitors)} found")

        except Exception as e:
            logger.error(f"Failed to refresh monitors: {e}")
            self._brightness_popup.set_status(
                "Failed to detect monitors", is_error=True
            )

    def refresh_monitors(self) -> None:
        """Refresh the list of available monitors (for hotplug support)."""
        logger.info("Manual monitor refresh requested")
        self._refresh_monitors()

    def show(self) -> None:
        """Show the tray icon."""
        self._tray_icon.show()
        logger.info("System tray icon shown")

    def hide(self) -> None:
        """Hide the tray icon."""
        self._tray_icon.hide()
        self._brightness_popup.hide()
        logger.info("System tray icon hidden")

    def is_visible(self) -> bool:
        """
        Check if the tray icon is visible.

        Returns:
            True if visible, False otherwise
        """
        return self._tray_icon.isVisible()

    def set_tooltip(self, tooltip: str) -> None:
        """
        Set the tray icon tooltip.

        Args:
            tooltip: Tooltip text
        """
        self._tray_icon.setToolTip(tooltip)

    def show_message(
        self,
        title: str,
        message: str,
        icon: QSystemTrayIcon.MessageIcon = QSystemTrayIcon.MessageIcon.Information,
        timeout: int = 3000,
    ) -> None:
        """
        Show a system tray notification message.

        Args:
            title: Message title
            message: Message content
            icon: Message icon type
            timeout: Timeout in milliseconds
        """
        self._tray_icon.showMessage(title, message, icon, timeout)
        logger.debug(f"Showed notification: {title} - {message}")

    def get_brightness_popup(self) -> BrightnessPopup:
        """
        Get the brightness popup widget.

        Returns:
            BrightnessPopup instance
        """
        return self._brightness_popup

    def show_settings(self) -> None:
        """Show the settings dialog."""
        if self._settings_dialog is None and self._config_manager is not None:
            # Import here to avoid circular imports
            from .widgets.settings_dialog import SettingsDialog
            
            self._settings_dialog = SettingsDialog(
                config_manager=self._config_manager,
                parent=None,
            )
            self._settings_dialog.set_monitors(self._monitors)
            self._settings_dialog.settings_applied.connect(self._on_settings_applied)
        
        if self._settings_dialog is not None:
            self._settings_dialog.set_monitors(self._monitors)
            self._settings_dialog.show()
            self._settings_dialog.raise_()
            self._settings_dialog.activateWindow()
            logger.info("Settings dialog shown")

    def _on_settings_applied(self) -> None:
        """Handle settings being applied."""
        logger.info("Settings applied, refreshing UI")
        
        # Reload configuration
        if self._config_manager:
            try:
                config = self._config_manager.get()
                
                # Update brightness popup settings
                self._brightness_popup.set_auto_hide_ms(config.behavior.response_timeout_ms)
                
                # Update enabled VCP codes
                if hasattr(config.ui, "enabled_vcp_codes"):
                    self._brightness_popup.set_enabled_vcp_codes(config.ui.enabled_vcp_codes)
                    logger.info(f"Updated enabled VCP codes: {[f'0x{c:02X}' for c in config.ui.enabled_vcp_codes]}")
                
                # Update brightness slider step size if needed
                # This would require updating the BrightnessSlider class
                # For now, the step is handled in the DDC manager
                
                logger.info("UI refreshed with new settings")
                
            except Exception as e:
                logger.error(f"Failed to apply settings: {e}")

    def set_config_manager(self, config_manager) -> None:
        """
        Set the configuration manager.

        Args:
            config_manager: ConfigurationManager instance
        """
        self._config_manager = config_manager
        
        # Apply initial settings
        if config_manager:
            try:
                config = config_manager.get()
                self._brightness_popup.set_auto_hide_ms(config.behavior.response_timeout_ms)
                # Apply enabled VCP codes from config
                if hasattr(config.ui, "enabled_vcp_codes"):
                    self._brightness_popup.set_enabled_vcp_codes(config.ui.enabled_vcp_codes)
            except Exception as e:
                logger.warning(f"Failed to apply initial settings: {e}")

    def cleanup(self) -> None:
        """Clean up UI resources."""
        if self._settings_dialog is not None:
            self._settings_dialog.close()
            self._settings_dialog = None
        
        self._brightness_popup.hide()
        self._tray_icon.hide()
        logger.info("UI cleaned up")
