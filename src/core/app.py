"""
Application foundation module for Twinkle Linux.

This module provides the TwinkleApp class which serves as the main application
controller, managing the application lifecycle and coordinating all components.
"""

import logging
import signal
import sys
from pathlib import Path
from typing import Optional

from PyQt6.QtWidgets import QApplication
from PyQt6.QtCore import QObject, pyqtSignal

from .config import ConfigurationManager, AppConfig
from .logging import setup_logging, get_logger
from ..ddc.ddc_manager import DDCManager
from ..ui.ui_main import UIMain


logger = get_logger(__name__)


class TwinkleApp(QObject):
    """
    Main application controller for Twinkle Linux.

    This class manages the application lifecycle, coordinates between
    different components, and handles graceful shutdown.
    """

    # Signals for application lifecycle events
    about_to_quit = pyqtSignal()
    initialized = pyqtSignal()
    shutdown_started = pyqtSignal()

    def __init__(self, qt_app: QApplication) -> None:
        """
        Initialize the Twinkle application.

        Args:
            qt_app: The Qt QApplication instance.
        """
        super().__init__()

        self._qt_app: QApplication = qt_app
        self._config_manager: Optional[ConfigurationManager] = None
        self._ddc_manager: Optional[DDCManager] = None
        self._ui_main: Optional[UIMain] = None
        self._is_initialized: bool = False
        self._is_shutting_down: bool = False

        # Set up signal handlers for graceful shutdown
        self._setup_signal_handlers()

        logger.info("TwinkleApp instance created")

    @property
    def qt_app(self) -> QApplication:
        """Get the Qt QApplication instance."""
        return self._qt_app

    @property
    def config_manager(self) -> ConfigurationManager:
        """
        Get the configuration manager.

        Returns:
            The ConfigurationManager instance.

        Raises:
            RuntimeError: If the application has not been initialized.
        """
        if self._config_manager is None:
            raise RuntimeError("Application not initialized. Call initialize() first.")
        return self._config_manager

    @property
    def is_initialized(self) -> bool:
        """Check if the application has been initialized."""
        return self._is_initialized

    @property
    def is_shutting_down(self) -> bool:
        """Check if the application is shutting down."""
        return self._is_shutting_down

    @property
    def ddc_manager(self) -> Optional[DDCManager]:
        """
        Get the DDC manager.

        Returns:
            The DDCManager instance, or None if not initialized.
        """
        return self._ddc_manager

    @property
    def ui_main(self) -> Optional[UIMain]:
        """
        Get the UI main coordinator.

        Returns:
            The UIMain instance, or None if not initialized.
        """
        return self._ui_main

    def initialize(self) -> None:
        """
        Initialize the application and all its dependencies.

        This method sets up the configuration manager, DDC manager,
        and UI components for use.
        """
        if self._is_initialized:
            logger.warning("Application already initialized")
            return

        logger.info("Initializing Twinkle Linux application...")

        try:
            # Initialize configuration manager
            self._config_manager = ConfigurationManager()
            config = self._config_manager.load()
            logger.info(f"Configuration loaded (version {config.version})")

            # Initialize DDC Manager (Phase 2)
            self._ddc_manager = DDCManager()
            if self._ddc_manager.initialize():
                logger.info("DDC Manager initialized successfully")
            else:
                logger.warning("DDC Manager initialization failed, continuing without DDC support")

            # Initialize UI components (Phase 3)
            self._initialize_ui()

            self._is_initialized = True
            self.initialized.emit()
            logger.info("Application initialized successfully")

        except Exception as e:
            logger.error(f"Failed to initialize application: {e}")
            raise

    def _initialize_ui(self) -> None:
        """Initialize the UI components."""
        # Get resource paths
        icon_path = self._get_resource_path("icons/twinkle.svg")
        stylesheet_path = self._get_resource_path("styles/default.qss")

        # Create UI main coordinator
        self._ui_main = UIMain(
            parent=self,
            icon_path=icon_path,
            stylesheet_path=stylesheet_path,
            config_manager=self._config_manager,
        )

        # Connect UI signals
        self._ui_main.brightness_changed.connect(self._on_brightness_changed)
        self._ui_main.monitor_selected.connect(self._on_monitor_selected)
        self._ui_main.settings_requested.connect(self._on_settings_requested)
        self._ui_main.quit_requested.connect(self.quit)

        # Set DDC manager for UI
        if self._ddc_manager:
            self._ui_main.set_ddc_manager(self._ddc_manager)

        # Show tray icon
        self._ui_main.show()

        logger.info("UI initialized successfully")

    def _get_resource_path(self, relative_path: str) -> Path | None:
        """
        Get the absolute path to a resource file.

        Args:
            relative_path: Relative path from src/ui/resources/

        Returns:
            Absolute path to the resource, or None if not found
        """
        # Try to find the resource in the package
        base_path = Path(__file__).parent.parent / "ui" / "resources"
        resource_path = base_path / relative_path

        if resource_path.exists():
            return resource_path

        logger.debug(f"Resource not found: {resource_path}")
        return None

    def _on_brightness_changed(self, value: int, monitor_id: str) -> None:
        """
        Handle brightness change from UI.

        Args:
            value: New brightness value
            monitor_id: Monitor unique ID
        """
        logger.debug(f"Brightness changed to {value} for monitor {monitor_id}")
        # DDC operations are handled by UIMain directly

    def _on_monitor_selected(self, monitor_id: str) -> None:
        """
        Handle monitor selection from UI.

        Args:
            monitor_id: Selected monitor unique ID
        """
        logger.debug(f"Monitor selected: {monitor_id}")

    def _on_settings_requested(self) -> None:
        """Handle settings request from UI."""
        logger.info("Settings requested (not yet implemented)")

    def run(self) -> int:
        """
        Run the application main loop.

        Returns:
            The exit code from the Qt application.
        """
        if not self._is_initialized:
            raise RuntimeError("Application not initialized. Call initialize() first.")

        logger.info("Starting application main loop")
        return self._qt_app.exec()

    def quit(self, exit_code: int = 0) -> None:
        """
        Quit the application gracefully.

        Args:
            exit_code: The exit code to return.
        """
        if self._is_shutting_down:
            logger.debug("Already shutting down")
            return

        logger.info("Initiating graceful shutdown...")
        self._is_shutting_down = True
        self.shutdown_started.emit()

        # Clean up UI
        if self._ui_main is not None:
            self._ui_main.cleanup()

        # Save configuration before quitting
        if self._config_manager is not None:
            try:
                self._config_manager.save()
                logger.info("Configuration saved")
            except Exception as e:
                logger.error(f"Failed to save configuration: {e}")

        # Emit about_to_quit signal
        self.about_to_quit.emit()

        # Quit the Qt application
        self._qt_app.quit()

        logger.info(f"Application shutting down with exit code {exit_code}")

    def _setup_signal_handlers(self) -> None:
        """Set up signal handlers for graceful shutdown."""
        # Handle SIGINT (Ctrl+C)
        signal.signal(signal.SIGINT, self._signal_handler)
        # Handle SIGTERM
        signal.signal(signal.SIGTERM, self._signal_handler)

    def _signal_handler(self, signum: int, frame) -> None:
        """
        Handle system signals for graceful shutdown.

        Args:
            signum: The signal number.
            frame: The current stack frame.
        """
        signal_name = signal.Signals(signum).name
        logger.info(f"Received signal {signal_name} ({signum})")
        self.quit()

    def restart(self) -> None:
        """
        Restart the application.

        This method saves the current state and restarts the application.
        """
        logger.info("Restarting application...")

        # Save configuration
        if self._config_manager is not None:
            try:
                self._config_manager.save()
            except Exception as e:
                logger.error(f"Failed to save configuration: {e}")

        # Quit and let the main.py handle restart
        self.quit()

    def get_config(self) -> AppConfig:
        """
        Get the current application configuration.

        Returns:
            The current AppConfig instance.
        """
        return self.config_manager.get()

    def reload_config(self) -> AppConfig:
        """
        Reload configuration from file.

        Returns:
            The reloaded AppConfig instance.
        """
        return self.config_manager.reload()

    def save_config(self) -> None:
        """Save the current configuration to file."""
        self.config_manager.save()

    def update_config(self, **kwargs) -> None:
        """
        Update configuration values.

        Args:
            **kwargs: Configuration key-value pairs to update.
        """
        self.config_manager.update(**kwargs)
        self.config_manager.save()
