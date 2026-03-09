"""
Settings Dialog Widget.

This module provides the SettingsDialog class, a comprehensive dialog
for configuring Twinkle Linux application settings.
"""

from __future__ import annotations

import logging
from typing import TYPE_CHECKING, Optional

from PyQt6.QtCore import Qt, pyqtSignal
from PyQt6.QtWidgets import (
    QDialog,
    QVBoxLayout,
    QHBoxLayout,
    QTabWidget,
    QWidget,
    QCheckBox,
    QComboBox,
    QLabel,
    QSlider,
    QSpinBox,
    QPushButton,
    QGroupBox,
    QListWidget,
    QListWidgetItem,
    QMessageBox,
    QDialogButtonBox,
)

if TYPE_CHECKING:
    from collections.abc import Callable

from ...core.config import (
    ConfigurationManager,
    AppConfig,
    UIConfig,
    BehaviorConfig,
    MonitorConfig,
)

logger = logging.getLogger(__name__)


class GeneralSettingsTab(QWidget):
    """General settings tab for theme, language, and auto-start options."""

    def __init__(self, parent: QWidget | None = None) -> None:
        """Initialize the general settings tab."""
        super().__init__(parent)
        self._setup_ui()

    def _setup_ui(self) -> None:
        """Set up the general settings UI components."""
        layout = QVBoxLayout(self)
        layout.setSpacing(16)

        # Auto-start group
        autostart_group = QGroupBox("Startup")
        autostart_layout = QVBoxLayout()
        
        self._autostart_checkbox = QCheckBox("Start on system login")
        self._autostart_checkbox.setToolTip("Automatically start Twinkle Linux when you log in")
        autostart_layout.addWidget(self._autostart_checkbox)
        autostart_group.setLayout(autostart_layout)
        layout.addWidget(autostart_group)

        # Appearance group
        appearance_group = QGroupBox("Appearance")
        appearance_layout = QVBoxLayout()

        # Theme selection
        theme_layout = QHBoxLayout()
        theme_label = QLabel("Theme:")
        self._theme_combo = QComboBox()
        self._theme_combo.addItem("Auto (follow system)", "system")
        self._theme_combo.addItem("Light", "light")
        self._theme_combo.addItem("Dark", "dark")
        theme_layout.addWidget(theme_label)
        theme_layout.addWidget(self._theme_combo)
        theme_layout.addStretch()
        appearance_layout.addLayout(theme_layout)

        # Language selection (placeholder)
        language_layout = QHBoxLayout()
        language_label = QLabel("Language:")
        self._language_combo = QComboBox()
        self._language_combo.addItem("English (US)", "en_US")
        self._language_combo.addItem("Polish", "pl_PL")
        self._language_combo.setEnabled(False)  # Placeholder for future i18n
        self._language_combo.setToolTip("Language support coming soon")
        language_layout.addWidget(language_label)
        language_layout.addWidget(self._language_combo)
        language_layout.addStretch()
        appearance_layout.addLayout(language_layout)

        appearance_group.setLayout(appearance_layout)
        layout.addWidget(appearance_group)

        layout.addStretch()

    def load_from_config(self, config: AppConfig) -> None:
        """Load settings from configuration."""
        self._autostart_checkbox.setChecked(config.behavior.auto_start)
        
        # Set theme
        theme_index = self._theme_combo.findData(config.ui.theme)
        if theme_index >= 0:
            self._theme_combo.setCurrentIndex(theme_index)

    def apply_to_config(self, config: AppConfig) -> None:
        """Apply settings to configuration."""
        config.behavior.auto_start = self._autostart_checkbox.isChecked()
        config.ui.theme = self._theme_combo.currentData()


class MonitorSettingsTab(QWidget):
    """Monitor settings tab for default monitor and per-monitor settings."""

    def __init__(self, parent: QWidget | None = None) -> None:
        """Initialize the monitor settings tab."""
        super().__init__(parent)
        self._monitors: dict[str, str] = {}  # unique_id -> display_name
        self._setup_ui()

    def _setup_ui(self) -> None:
        """Set up the monitor settings UI components."""
        layout = QVBoxLayout(self)
        layout.setSpacing(16)

        # Default monitor group
        default_group = QGroupBox("Default Monitor")
        default_layout = QVBoxLayout()
        
        default_label = QLabel("Select the default monitor on startup:")
        self._default_monitor_combo = QComboBox()
        self._default_monitor_combo.setMinimumWidth(250)
        default_layout.addWidget(default_label)
        default_layout.addWidget(self._default_monitor_combo)
        default_group.setLayout(default_layout)
        layout.addWidget(default_group)

        # Monitor list group
        monitors_group = QGroupBox("Detected Monitors")
        monitors_layout = QVBoxLayout()
        
        self._monitor_list = QListWidget()
        self._monitor_list.setMinimumHeight(150)
        monitors_layout.addWidget(self._monitor_list)
        monitors_group.setLayout(monitors_layout)
        layout.addWidget(monitors_group)

        # Per-monitor settings group
        per_monitor_group = QGroupBox("Monitor Brightness Presets")
        per_monitor_layout = QVBoxLayout()
        
        preset_info = QLabel("Select a monitor above to configure its brightness presets:")
        preset_info.setWordWrap(True)
        per_monitor_layout.addWidget(preset_info)

        # Preset sliders
        self._preset_low_slider = self._create_preset_slider("Low", 0, 30)
        per_monitor_layout.addLayout(self._preset_low_slider[0])

        self._preset_medium_slider = self._create_preset_slider("Medium", 31, 70)
        per_monitor_layout.addLayout(self._preset_medium_slider[0])

        self._preset_high_slider = self._create_preset_slider("High", 71, 100)
        per_monitor_layout.addLayout(self._preset_high_slider[0])

        # Save presets button
        self._save_presets_btn = QPushButton("Save Presets for Selected Monitor")
        self._save_presets_btn.setEnabled(False)
        per_monitor_layout.addWidget(self._save_presets_btn)

        per_monitor_group.setLayout(per_monitor_layout)
        layout.addWidget(per_monitor_group)

        # Connect monitor list selection
        self._monitor_list.itemSelectionChanged.connect(self._on_monitor_selected)

        layout.addStretch()

    def _create_preset_slider(self, label: str, min_val: int, max_val: int) -> tuple[QHBoxLayout, QSlider]:
        """Create a preset slider with label."""
        layout = QHBoxLayout()
        
        lbl = QLabel(f"{label}:")
        slider = QSlider(Qt.Orientation.Horizontal)
        slider.setMinimum(min_val)
        slider.setMaximum(max_val)
        slider.setValue((min_val + max_val) // 2)
        slider.setTickPosition(QSlider.TickPosition.TicksBelow)
        slider.setTickInterval(10)
        
        value_label = QLabel(f"{slider.value()}%")
        value_label.setFixedWidth(40)
        
        slider.valueChanged.connect(lambda v, l=value_label: l.setText(f"{v}%"))
        
        layout.addWidget(lbl)
        layout.addWidget(slider)
        layout.addWidget(value_label)
        
        return (layout, slider, value_label)

    def _on_monitor_selected(self) -> None:
        """Handle monitor selection in the list."""
        selected_items = self._monitor_list.selectedItems()
        if selected_items:
            monitor_id = selected_items[0].data(Qt.ItemDataRole.UserRole)
            self._save_presets_btn.setEnabled(True)
            # Load presets for this monitor if available
            # This would be implemented when we have the config available
        else:
            self._save_presets_btn.setEnabled(False)

    def set_monitors(self, monitors: dict[str, str]) -> None:
        """Set the available monitors."""
        self._monitors = monitors.copy()
        
        # Update default monitor combo
        current_default = self._default_monitor_combo.currentData()
        self._default_monitor_combo.clear()
        for unique_id, display_name in self._monitors.items():
            self._default_monitor_combo.addItem(display_name, unique_id)
        
        # Restore selection
        if current_default:
            index = self._default_monitor_combo.findData(current_default)
            if index >= 0:
                self._default_monitor_combo.setCurrentIndex(index)
        
        # Update monitor list
        self._monitor_list.clear()
        for unique_id, display_name in self._monitors.items():
            item = QListWidgetItem(f"{display_name}")
            item.setData(Qt.ItemDataRole.UserRole, unique_id)
            self._monitor_list.addItem(item)

    def load_from_config(self, config: AppConfig) -> None:
        """Load settings from configuration."""
        # Default monitor would be loaded from config
        # This would require storing the default monitor ID in the config
        pass

    def apply_to_config(self, config: AppConfig) -> None:
        """Apply settings to configuration."""
        # Save default monitor to config
        default_monitor = self._default_monitor_combo.currentData()
        if default_monitor:
            # Store in config - would need to add this to AppConfig
            pass
        
        # Save presets for selected monitor
        selected_items = self._monitor_list.selectedItems()
        if selected_items:
            monitor_id = selected_items[0].data(Qt.ItemDataRole.UserRole)
            if monitor_id in config.monitors:
                monitor_config = config.monitors[monitor_id]
                monitor_config.brightness_presets = {
                    "low": self._preset_low_slider[1].value(),
                    "medium": self._preset_medium_slider[1].value(),
                    "high": self._preset_high_slider[1].value(),
                }


class BehaviorSettingsTab(QWidget):
    """Behavior settings tab for popup behavior and notifications."""

    def __init__(self, parent: QWidget | None = None) -> None:
        """Initialize the behavior settings tab."""
        super().__init__(parent)
        self._setup_ui()

    def _setup_ui(self) -> None:
        """Set up the behavior settings UI components."""
        layout = QVBoxLayout(self)
        layout.setSpacing(16)

        # Popup behavior group
        popup_group = QGroupBox("Brightness Popup")
        popup_layout = QVBoxLayout()

        # Auto-hide time
        autohide_layout = QHBoxLayout()
        autohide_label = QLabel("Auto-hide time:")
        self._autohide_slider = QSlider(Qt.Orientation.Horizontal)
        self._autohide_slider.setMinimum(1)
        self._autohide_slider.setMaximum(10)
        self._autohide_slider.setValue(3)
        self._autohide_slider.setTickPosition(QSlider.TickPosition.TicksBelow)
        self._autohide_slider.setTickInterval(1)
        self._autohide_label = QLabel("3 seconds")
        self._autohide_label.setFixedWidth(80)
        
        self._autohide_slider.valueChanged.connect(self._on_autohide_changed)
        
        autohide_layout.addWidget(autohide_label)
        autohide_layout.addWidget(self._autohide_slider)
        autohide_layout.addWidget(self._autohide_label)
        popup_layout.addLayout(autohide_layout)

        # Brightness step size
        step_layout = QHBoxLayout()
        step_label = QLabel("Brightness step size:")
        self._step_slider = QSlider(Qt.Orientation.Horizontal)
        self._step_slider.setMinimum(1)
        self._step_slider.setMaximum(10)
        self._step_slider.setValue(5)
        self._step_slider.setTickPosition(QSlider.TickPosition.TicksBelow)
        self._step_slider.setTickInterval(1)
        self._step_label = QLabel("5")
        self._step_label.setFixedWidth(40)
        
        self._step_slider.valueChanged.connect(self._on_step_changed)
        
        step_layout.addWidget(step_label)
        step_layout.addWidget(self._step_slider)
        step_layout.addWidget(self._step_label)
        popup_layout.addLayout(step_layout)

        popup_group.setLayout(popup_layout)
        layout.addWidget(popup_group)

        # Notifications group
        notifications_group = QGroupBox("Notifications")
        notifications_layout = QVBoxLayout()

        self._notify_errors_checkbox = QCheckBox("Show notifications for errors")
        self._notify_errors_checkbox.setChecked(True)
        notifications_layout.addWidget(self._notify_errors_checkbox)

        self._notify_brightness_checkbox = QCheckBox("Show notifications for brightness changes")
        self._notify_brightness_checkbox.setChecked(False)
        notifications_layout.addWidget(self._notify_brightness_checkbox)

        notifications_group.setLayout(notifications_layout)
        layout.addWidget(notifications_group)

        layout.addStretch()

    def _on_autohide_changed(self, value: int) -> None:
        """Handle auto-hide slider change."""
        self._autohide_label.setText(f"{value} seconds")

    def _on_step_changed(self, value: int) -> None:
        """Handle step slider change."""
        self._step_label.setText(str(value))

    def load_from_config(self, config: AppConfig) -> None:
        """Load settings from configuration."""
        # Auto-hide time - convert from ms to seconds
        autohide_seconds = config.behavior.response_timeout_ms // 1000
        autohide_seconds = max(1, min(10, autohide_seconds))
        self._autohide_slider.setValue(autohide_seconds)
        
        # Brightness step
        step = config.behavior.brightness_step
        step = max(1, min(10, step))
        self._step_slider.setValue(step)

    def apply_to_config(self, config: AppConfig) -> None:
        """Apply settings to configuration."""
        # Auto-hide time - convert to ms
        config.behavior.response_timeout_ms = self._autohide_slider.value() * 1000
        
        # Brightness step
        config.behavior.brightness_step = self._step_slider.value()


class AdvancedSettingsTab(QWidget):
    """Advanced settings tab for logging, debug mode, and timeouts."""

    # Signal emitted when reset to defaults is requested
    reset_requested = pyqtSignal()

    def __init__(self, parent: QWidget | None = None) -> None:
        """Initialize the advanced settings tab."""
        super().__init__(parent)
        self._setup_ui()

    def _setup_ui(self) -> None:
        """Set up the advanced settings UI components."""
        layout = QVBoxLayout(self)
        layout.setSpacing(16)

        # Logging group
        logging_group = QGroupBox("Logging")
        logging_layout = QVBoxLayout()

        # Log level
        loglevel_layout = QHBoxLayout()
        loglevel_label = QLabel("Log level:")
        self._loglevel_combo = QComboBox()
        self._loglevel_combo.addItem("DEBUG", "DEBUG")
        self._loglevel_combo.addItem("INFO", "INFO")
        self._loglevel_combo.addItem("WARNING", "WARNING")
        self._loglevel_combo.addItem("ERROR", "ERROR")
        self._loglevel_combo.addItem("CRITICAL", "CRITICAL")
        self._loglevel_combo.setCurrentIndex(1)  # Default to INFO
        loglevel_layout.addWidget(loglevel_label)
        loglevel_layout.addWidget(self._loglevel_combo)
        loglevel_layout.addStretch()
        logging_layout.addLayout(loglevel_layout)

        # Debug mode
        self._debug_checkbox = QCheckBox("Enable debug mode")
        self._debug_checkbox.setToolTip("Enable verbose logging and additional debug information")
        logging_layout.addWidget(self._debug_checkbox)

        logging_group.setLayout(logging_layout)
        layout.addWidget(logging_group)

        # Performance group
        performance_group = QGroupBox("Performance")
        performance_layout = QVBoxLayout()

        # Command timeout
        timeout_layout = QHBoxLayout()
        timeout_label = QLabel("Command timeout (seconds):")
        self._timeout_spinner = QSpinBox()
        self._timeout_spinner.setMinimum(1)
        self._timeout_spinner.setMaximum(10)
        self._timeout_spinner.setValue(1)
        timeout_layout.addWidget(timeout_label)
        timeout_layout.addWidget(self._timeout_spinner)
        timeout_layout.addStretch()
        performance_layout.addLayout(timeout_layout)

        # Retry count
        retry_layout = QHBoxLayout()
        retry_label = QLabel("Retry count:")
        self._retry_spinner = QSpinBox()
        self._retry_spinner.setMinimum(1)
        self._retry_spinner.setMaximum(5)
        self._retry_spinner.setValue(3)
        retry_layout.addWidget(retry_label)
        retry_layout.addWidget(self._retry_spinner)
        retry_layout.addStretch()
        performance_layout.addLayout(retry_layout)

        performance_group.setLayout(performance_layout)
        layout.addWidget(performance_group)

        # Reset button
        reset_layout = QHBoxLayout()
        reset_layout.addStretch()
        self._reset_button = QPushButton("Reset to Defaults")
        self._reset_button.setStyleSheet("color: #e74c3c;")
        self._reset_button.clicked.connect(self._on_reset_clicked)
        reset_layout.addWidget(self._reset_button)
        reset_layout.addStretch()
        layout.addLayout(reset_layout)

        layout.addStretch()

    def _on_reset_clicked(self) -> None:
        """Handle reset to defaults button click."""
        self.reset_requested.emit()

    def load_from_config(self, config: AppConfig) -> None:
        """Load settings from configuration."""
        # Log level - would need to add this to AppConfig
        self._loglevel_combo.setCurrentIndex(1)  # Default to INFO
        
        # Debug mode - would need to add this to AppConfig
        self._debug_checkbox.setChecked(False)
        
        # Command timeout - convert from ms to seconds
        timeout_seconds = config.behavior.response_timeout_ms // 1000
        timeout_seconds = max(1, min(10, timeout_seconds))
        self._timeout_spinner.setValue(timeout_seconds)
        
        # Retry count
        retry = config.behavior.max_retries
        retry = max(1, min(5, retry))
        self._retry_spinner.setValue(retry)

    def apply_to_config(self, config: AppConfig) -> None:
        """Apply settings to configuration."""
        # Command timeout - convert to ms
        config.behavior.response_timeout_ms = self._timeout_spinner.value() * 1000
        
        # Retry count
        config.behavior.max_retries = self._retry_spinner.value()


class VCPSettingsTab(QWidget):
    """VCP settings tab for enabling/disabling VCP controls in the popup."""

    # Available VCP codes that can be controlled
    AVAILABLE_VCP_CODES = {
        0x10: ("Brightness", "Display brightness level", True),
        0x12: ("Contrast", "Display contrast level", True),
        0x14: ("Color Temperature", "Color temperature preset", True),
        0x60: ("Input Source", "Video input source selection", True),
        0x62: ("Volume", "Monitor speaker volume", True),
    }

    def __init__(self, parent: QWidget | None = None) -> None:
        """Initialize the VCP settings tab."""
        super().__init__(parent)
        self._enabled_vcp_codes: list[int] = [0x10, 0x12, 0x14, 0x60, 0x62]
        self._vcp_checkboxes: dict[int, QCheckBox] = {}
        self._setup_ui()

    def _setup_ui(self) -> None:
        """Set up the VCP settings UI components."""
        layout = QVBoxLayout(self)
        layout.setSpacing(16)

        # VCP controls group
        vcp_group = QGroupBox("VCP Controls in Popup")
        vcp_layout = QVBoxLayout()

        # Description
        description = QLabel(
            "Select which VCP controls should appear in the brightness popup. "
            "Not all monitors support all VCP codes."
        )
        description.setWordWrap(True)
        vcp_layout.addWidget(description)

        # Add checkboxes for each VCP code
        for vcp_code, (name, description, default) in sorted(self.AVAILABLE_VCP_CODES.items()):
            checkbox = QCheckBox(f"{name} (0x{vcp_code:02X})")
            checkbox.setToolTip(description)
            checkbox.setChecked(vcp_code in self._enabled_vcp_codes)
            vcp_layout.addWidget(checkbox)
            self._vcp_checkboxes[vcp_code] = checkbox

        vcp_group.setLayout(vcp_layout)
        layout.addWidget(vcp_group)

        # Preset configurations group
        preset_group = QGroupBox("Quick Presets")
        preset_layout = QVBoxLayout()

        preset_description = QLabel(
            "Select a preset to quickly enable common VCP control combinations:"
        )
        preset_description.setWordWrap(True)
        preset_layout.addWidget(preset_description)

        # Preset buttons
        preset_buttons_layout = QHBoxLayout()

        minimal_btn = QPushButton("Minimal")
        minimal_btn.setToolTip("Only show brightness control")
        minimal_btn.clicked.connect(lambda: self._apply_preset([0x10]))
        preset_buttons_layout.addWidget(minimal_btn)

        standard_btn = QPushButton("Standard")
        standard_btn.setToolTip("Show display controls (Brightness, Contrast, Color Temperature)")
        standard_btn.clicked.connect(lambda: self._apply_preset([0x10, 0x12, 0x14]))
        preset_buttons_layout.addWidget(standard_btn)

        full_btn = QPushButton("Full")
        full_btn.setToolTip("Show all available VCP controls")
        full_btn.clicked.connect(lambda: self._apply_preset([0x10, 0x12, 0x14, 0x60, 0x62]))
        preset_buttons_layout.addWidget(full_btn)

        preset_layout.addLayout(preset_buttons_layout)
        preset_group.setLayout(preset_layout)
        layout.addWidget(preset_group)

        layout.addStretch()

    def _apply_preset(self, vcp_codes: list[int]) -> None:
        """
        Apply a preset configuration of VCP codes.

        Args:
            vcp_codes: List of VCP codes to enable
        """
        for vcp_code, checkbox in self._vcp_checkboxes.items():
            checkbox.setChecked(vcp_code in vcp_codes)
        self._update_enabled_vcp_codes()

    def _update_enabled_vcp_codes(self) -> None:
        """Update the enabled VCP codes list based on checkbox states."""
        self._enabled_vcp_codes = [
            vcp_code
            for vcp_code, checkbox in self._vcp_checkboxes.items()
            if checkbox.isChecked()
        ]

    def load_from_config(self, config: AppConfig) -> None:
        """Load settings from configuration."""
        # Get enabled VCP codes from config or use default
        if hasattr(config, "ui") and hasattr(config.ui, "enabled_vcp_codes"):
            self._enabled_vcp_codes = config.ui.enabled_vcp_codes
        else:
            self._enabled_vcp_codes = [0x10, 0x12, 0x14, 0x60, 0x62]

        # Update checkboxes
        for vcp_code, checkbox in self._vcp_checkboxes.items():
            checkbox.setChecked(vcp_code in self._enabled_vcp_codes)

    def apply_to_config(self, config: AppConfig) -> None:
        """Apply settings to configuration."""
        self._update_enabled_vcp_codes()
        # Store enabled VCP codes in config
        if not hasattr(config.ui, "enabled_vcp_codes"):
            config.ui.model_extra["enabled_vcp_codes"] = []
        config.ui.enabled_vcp_codes = self._enabled_vcp_codes

    def get_enabled_vcp_codes(self) -> list[int]:
        """
        Get the list of enabled VCP codes.

        Returns:
            List of enabled VCP codes
        """
        self._update_enabled_vcp_codes()
        return self._enabled_vcp_codes.copy()


class SettingsDialog(QDialog):
    """
    Main settings dialog for Twinkle Linux.

    This dialog provides a tabbed interface for configuring all aspects
    of the application, including general settings, monitor settings,
    behavior settings, and advanced settings.
    """

    # Signal emitted when settings are applied
    settings_applied = pyqtSignal()

    def __init__(
        self,
        config_manager: ConfigurationManager,
        parent: QWidget | None = None,
    ) -> None:
        """
        Initialize the settings dialog.

        Args:
            config_manager: The configuration manager instance
            parent: Parent widget
        """
        super().__init__(parent)
        
        self._config_manager = config_manager
        self._config: Optional[AppConfig] = None
        self._monitors: dict[str, str] = {}
        
        self._setup_ui()
        self._load_settings()

    def _setup_ui(self) -> None:
        """Set up the dialog UI components."""
        self.setWindowTitle("Twinkle Linux Settings")
        self.setMinimumSize(600, 500)
        
        layout = QVBoxLayout(self)
        
        # Create tab widget
        self._tab_widget = QTabWidget()
        
        # Create tabs
        self._general_tab = GeneralSettingsTab()
        self._monitor_tab = MonitorSettingsTab()
        self._behavior_tab = BehaviorSettingsTab()
        self._advanced_tab = AdvancedSettingsTab()
        self._vcp_tab = VCPSettingsTab()
        
        # Connect reset signal
        self._advanced_tab.reset_requested.connect(self._reset_to_defaults)
        
        # Add tabs
        self._tab_widget.addTab(self._general_tab, "General")
        self._tab_widget.addTab(self._monitor_tab, "Monitors")
        self._tab_widget.addTab(self._behavior_tab, "Behavior")
        self._tab_widget.addTab(self._vcp_tab, "VCP Controls")
        self._tab_widget.addTab(self._advanced_tab, "Advanced")
        
        layout.addWidget(self._tab_widget)
        
        # Create button box
        self._button_box = QDialogButtonBox(
            QDialogButtonBox.StandardButton.Save |
            QDialogButtonBox.StandardButton.Apply |
            QDialogButtonBox.StandardButton.Cancel
        )
        
        # Connect buttons
        self._button_box.button(QDialogButtonBox.StandardButton.Save).clicked.connect(self._on_save)
        self._button_box.button(QDialogButtonBox.StandardButton.Apply).clicked.connect(self._on_apply)
        self._button_box.button(QDialogButtonBox.StandardButton.Cancel).clicked.connect(self._on_cancel)
        
        layout.addWidget(self._button_box)

    def _load_settings(self) -> None:
        """Load settings from configuration manager."""
        try:
            self._config = self._config_manager.get()
            
            # Load settings into each tab
            self._general_tab.load_from_config(self._config)
            self._monitor_tab.load_from_config(self._config)
            self._monitor_tab.set_monitors(self._monitors)
            self._behavior_tab.load_from_config(self._config)
            self._vcp_tab.load_from_config(self._config)
            self._advanced_tab.load_from_config(self._config)
            
            logger.debug("Settings loaded into dialog")
            
        except Exception as e:
            logger.error(f"Failed to load settings: {e}")
            QMessageBox.critical(
                self,
                "Error",
                f"Failed to load settings: {e}"
            )

    def _save_settings(self) -> bool:
        """
        Save settings to configuration manager.

        Returns:
            True if successful, False otherwise
        """
        try:
            if self._config is None:
                self._config = self._config_manager.get()
            
            # Apply settings from each tab
            self._general_tab.apply_to_config(self._config)
            self._monitor_tab.apply_to_config(self._config)
            self._behavior_tab.apply_to_config(self._config)
            self._vcp_tab.apply_to_config(self._config)
            self._advanced_tab.apply_to_config(self._config)
            
            # Save to file
            self._config_manager.update(
                ui=self._config.ui.model_dump(mode="json"),
                behavior=self._config.behavior.model_dump(mode="json"),
            )
            self._config_manager.save()
            
            logger.info("Settings saved successfully")
            return True
            
        except Exception as e:
            logger.error(f"Failed to save settings: {e}")
            QMessageBox.critical(
                self,
                "Error",
                f"Failed to save settings: {e}"
            )
            return False

    def _on_save(self) -> None:
        """Handle save button click."""
        if self._save_settings():
            self.settings_applied.emit()
            QMessageBox.information(
                self,
                "Settings Saved",
                "Settings have been saved successfully."
            )
            self.accept()

    def _on_apply(self) -> None:
        """Handle apply button click."""
        if self._save_settings():
            self.settings_applied.emit()
            # Reload settings to reflect changes
            self._load_settings()

    def _on_cancel(self) -> None:
        """Handle cancel button click."""
        self.reject()

    def _reset_to_defaults(self) -> None:
        """Reset all settings to defaults."""
        reply = QMessageBox.question(
            self,
            "Reset to Defaults",
            "Are you sure you want to reset all settings to their default values?",
            QMessageBox.StandardButton.Yes | QMessageBox.StandardButton.No,
            QMessageBox.StandardButton.No
        )
        
        if reply == QMessageBox.StandardButton.Yes:
            try:
                self._config_manager.reset_to_defaults()
                self._load_settings()
                QMessageBox.information(
                    self,
                    "Reset Complete",
                    "All settings have been reset to their default values."
                )
            except Exception as e:
                logger.error(f"Failed to reset to defaults: {e}")
                QMessageBox.critical(
                    self,
                    "Error",
                    f"Failed to reset to defaults: {e}"
                )

    def set_monitors(self, monitors: dict[str, str]) -> None:
        """
        Set the available monitors.

        Args:
            monitors: Dictionary mapping unique_id to display_name
        """
        self._monitors = monitors.copy()
        self._monitor_tab.set_monitors(self._monitors)

    def get_monitors(self) -> dict[str, str]:
        """
        Get the available monitors.

        Returns:
            Dictionary mapping unique_id to display_name
        """
        return self._monitors.copy()

    def showEvent(self, event) -> None:
        """Handle show event."""
        super().showEvent(event)
        # Reload settings when dialog is shown
        self._load_settings()
