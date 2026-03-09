"""
Brightness Popup Widget.

This module provides the BrightnessPopup class, a popup window for
quick brightness adjustments with additional VCP controls.
"""

from __future__ import annotations

import logging
from typing import TYPE_CHECKING

from PyQt6.QtCore import Qt, QPoint, QTimer, pyqtSignal
from PyQt6.QtWidgets import (
    QWidget,
    QVBoxLayout,
    QHBoxLayout,
    QPushButton,
    QLabel,
    QFrame,
    QComboBox,
    QTabWidget,
    QScrollArea,
)

if TYPE_CHECKING:
    from collections.abc import Callable

from .widgets.brightness_slider import BrightnessSlider
from .widgets.vcp_controls import VCPControlSection

logger = logging.getLogger(__name__)


class BrightnessPopup(QWidget):
    """
    A popup window for quick brightness adjustments with additional VCP controls.

    This widget provides a brightness slider, preset buttons, and additional
    VCP controls (contrast, volume, input source, color temperature) organized
    in tabs. It auto-hides after inactivity.
    """

    # Signal emitted when brightness changes
    brightness_changed = pyqtSignal(int)

    # Signal emitted when monitor selection changes
    monitor_changed = pyqtSignal(str)

    # Signal emitted when VCP value changes
    vcp_changed = pyqtSignal(int, int)  # vcp_code, value

    def __init__(
        self,
        parent: QWidget | None = None,
        auto_hide_ms: int = 3000,
        show_monitor_selector: bool = False,
    ) -> None:
        """
        Initialize the brightness popup.

        Args:
            parent: Parent widget
            auto_hide_ms: Auto-hide delay in milliseconds (0 to disable)
            show_monitor_selector: Whether to show monitor selector
        """
        super().__init__(parent)

        self._auto_hide_ms = auto_hide_ms
        self._show_monitor_selector = show_monitor_selector
        self._monitors: dict[str, str] = {}  # unique_id -> display_name
        self._current_monitor: str | None = None
        self._enabled_vcp_codes: list[int] = [0x10, 0x12, 0x14, 0x60, 0x62]  # Default enabled VCP codes
        self._vcp_sections: dict[str, VCPControlSection] = {}

        # Auto-hide timer
        self._auto_hide_timer = QTimer(self)
        self._auto_hide_timer.setSingleShot(True)
        self._auto_hide_timer.timeout.connect(self._on_auto_hide_timeout)

        # Set up UI
        self._setup_ui()

        # Set window flags for popup behavior
        self.setWindowFlags(
            Qt.WindowType.Popup
            | Qt.WindowType.FramelessWindowHint
            | Qt.WindowType.NoDropShadowWindowHint
        )
        self.setAttribute(Qt.WidgetAttribute.WA_TranslucentBackground)
        self.setAttribute(Qt.WidgetAttribute.WA_ShowWithoutActivating)

    def _setup_ui(self) -> None:
        """Set up the popup UI components."""
        # Main container with rounded corners and shadow
        self._container = QFrame(self)
        self._container.setObjectName("brightnessPopupContainer")

        container_layout = QVBoxLayout(self._container)
        container_layout.setContentsMargins(12, 12, 12, 12)
        container_layout.setSpacing(12)

        # Header with title and monitor selector
        header_layout = QHBoxLayout()
        header_layout.setSpacing(8)

        self._title_label = QLabel("Brightness")
        self._title_label.setObjectName("brightnessPopupTitle")
        header_layout.addWidget(self._title_label)

        header_layout.addStretch()

        # Monitor selector (optional)
        if self._show_monitor_selector:
            self._monitor_selector = QComboBox()
            self._monitor_selector.setObjectName("brightnessPopupMonitorSelector")
            self._monitor_selector.setMinimumWidth(150)
            self._monitor_selector.currentIndexChanged.connect(
                self._on_monitor_selection_changed
            )
            header_layout.addWidget(self._monitor_selector)
        else:
            self._monitor_selector = None

        container_layout.addLayout(header_layout)

        # Create tab widget for different control groups
        self._tab_widget = QTabWidget()
        self._tab_widget.setObjectName("brightnessPopupTabWidget")

        # Brightness tab (main tab)
        brightness_tab = self._create_brightness_tab()
        self._tab_widget.addTab(brightness_tab, "Brightness")

        # Display tab (Contrast, Color Temperature)
        display_tab = self._create_display_tab()
        self._tab_widget.addTab(display_tab, "Display")

        # Audio tab (Volume)
        audio_tab = self._create_audio_tab()
        self._tab_widget.addTab(audio_tab, "Audio")

        # Input tab (Input Source)
        input_tab = self._create_input_tab()
        self._tab_widget.addTab(input_tab, "Input")

        container_layout.addWidget(self._tab_widget)

        # Status label for errors/info
        self._status_label = QLabel("")
        self._status_label.setObjectName("brightnessPopupStatus")
        self._status_label.setAlignment(Qt.AlignmentFlag.AlignCenter)
        self._status_label.setWordWrap(True)
        container_layout.addWidget(self._status_label)

        # Main layout
        main_layout = QVBoxLayout(self)
        main_layout.setContentsMargins(0, 0, 0, 0)
        main_layout.addWidget(self._container)

    def _create_brightness_tab(self) -> QWidget:
        """Create the brightness tab with brightness slider and presets."""
        tab = QWidget()
        layout = QVBoxLayout(tab)
        layout.setContentsMargins(0, 0, 0, 0)
        layout.setSpacing(12)

        # Brightness slider
        self._brightness_slider = BrightnessSlider(
            parent=tab,
            debounce_ms=200,
            show_value_label=True,
        )
        self._brightness_slider.brightness_changed.connect(self._on_brightness_changed)
        layout.addWidget(self._brightness_slider)

        # Preset buttons
        preset_layout = QHBoxLayout()
        preset_layout.setSpacing(8)

        preset_values = [25, 50, 75, 100]
        self._preset_buttons: list[QPushButton] = []

        for value in preset_values:
            btn = QPushButton(f"{value}%")
            btn.setObjectName("brightnessPresetButton")
            btn.setProperty("presetValue", value)
            btn.clicked.connect(lambda checked, v=value: self._on_preset_clicked(v))
            self._preset_buttons.append(btn)
            preset_layout.addWidget(btn)

        layout.addLayout(preset_layout)
        layout.addStretch()

        return tab

    def _create_display_tab(self) -> QWidget:
        """Create the display tab with contrast and color temperature controls."""
        tab = QWidget()
        layout = QVBoxLayout(tab)
        layout.setContentsMargins(0, 0, 0, 0)
        layout.setSpacing(12)

        # Scroll area for controls
        scroll = QScrollArea()
        scroll.setWidgetResizable(True)
        scroll.setFrameShape(QFrame.Shape.NoFrame)

        scroll_content = QWidget()
        scroll_layout = QVBoxLayout(scroll_content)
        scroll_layout.setContentsMargins(0, 0, 0, 0)
        scroll_layout.setSpacing(12)

        # Contrast control section
        if 0x12 in self._enabled_vcp_codes:
            self._contrast_section = VCPControlSection("Contrast", scroll_content)
            self._contrast_slider = self._contrast_section.add_slider(0x12, debounce_ms=200)
            self._contrast_slider.value_changed.connect(
                lambda v: self._on_vcp_changed(0x12, v)
            )
            scroll_layout.addWidget(self._contrast_section)
            self._vcp_sections["display"] = self._contrast_section

        # Color Temperature control section
        if 0x14 in self._enabled_vcp_codes:
            self._color_temp_section = VCPControlSection("Color Temperature", scroll_content)
            self._color_temp_combo = self._color_temp_section.add_combo_box(0x14)
            self._color_temp_combo.value_changed.connect(
                lambda v: self._on_vcp_changed(0x14, v)
            )
            scroll_layout.addWidget(self._color_temp_section)

        scroll_layout.addStretch()
        scroll.setWidget(scroll_content)
        layout.addWidget(scroll)

        return tab

    def _create_audio_tab(self) -> QWidget:
        """Create the audio tab with volume control."""
        tab = QWidget()
        layout = QVBoxLayout(tab)
        layout.setContentsMargins(0, 0, 0, 0)
        layout.setSpacing(12)

        # Scroll area for controls
        scroll = QScrollArea()
        scroll.setWidgetResizable(True)
        scroll.setFrameShape(QFrame.Shape.NoFrame)

        scroll_content = QWidget()
        scroll_layout = QVBoxLayout(scroll_content)
        scroll_layout.setContentsMargins(0, 0, 0, 0)
        scroll_layout.setSpacing(12)

        # Volume control section
        if 0x62 in self._enabled_vcp_codes:
            self._volume_section = VCPControlSection("Volume", scroll_content)
            self._volume_slider = self._volume_section.add_slider(0x62, debounce_ms=200)
            self._volume_slider.value_changed.connect(
                lambda v: self._on_vcp_changed(0x62, v)
            )
            scroll_layout.addWidget(self._volume_section)
            self._vcp_sections["audio"] = self._volume_section

        scroll_layout.addStretch()
        scroll.setWidget(scroll_content)
        layout.addWidget(scroll)

        return tab

    def _create_input_tab(self) -> QWidget:
        """Create the input tab with input source control."""
        tab = QWidget()
        layout = QVBoxLayout(tab)
        layout.setContentsMargins(0, 0, 0, 0)
        layout.setSpacing(12)

        # Scroll area for controls
        scroll = QScrollArea()
        scroll.setWidgetResizable(True)
        scroll.setFrameShape(QFrame.Shape.NoFrame)

        scroll_content = QWidget()
        scroll_layout = QVBoxLayout(scroll_content)
        scroll_layout.setContentsMargins(0, 0, 0, 0)
        scroll_layout.setSpacing(12)

        # Input Source control section
        if 0x60 in self._enabled_vcp_codes:
            self._input_section = VCPControlSection("Input Source", scroll_content)
            self._input_combo = self._input_section.add_combo_box(0x60)
            self._input_combo.value_changed.connect(
                lambda v: self._on_vcp_changed(0x60, v)
            )
            scroll_layout.addWidget(self._input_section)
            self._vcp_sections["input"] = self._input_section

        scroll_layout.addStretch()
        scroll.setWidget(scroll_content)
        layout.addWidget(scroll)

        return tab

    def _on_brightness_changed(self, value: int) -> None:
        """
        Handle brightness change from slider.

        Args:
            value: New brightness value
        """
        logger.debug(f"Brightness popup: brightness changed to {value}")
        self.brightness_changed.emit(value)

    def _on_vcp_changed(self, vcp_code: int, value: int) -> None:
        """
        Handle VCP value change.

        Args:
            vcp_code: The VCP code that changed
            value: New value
        """
        logger.debug(f"Brightness popup: VCP 0x{vcp_code:02X} changed to {value}")
        self.vcp_changed.emit(vcp_code, value)

    def _on_preset_clicked(self, value: int) -> None:
        """
        Handle preset button click.

        Args:
            value: Preset brightness value
        """
        logger.debug(f"Brightness popup: preset clicked: {value}")
        self._brightness_slider.set_brightness(value)
        self.brightness_changed.emit(value)

    def _on_monitor_selection_changed(self, index: int) -> None:
        """
        Handle monitor selection change from combo box.

        Args:
            index: Selected index in the combo box
        """
        if index >= 0 and self._monitor_selector is not None:
            monitor_id = self._monitor_selector.itemData(index)
            if monitor_id:
                self._on_monitor_changed(monitor_id)

    def _on_monitor_changed(self, monitor_id: str) -> None:
        """
        Handle monitor selection change.

        Args:
            monitor_id: Selected monitor unique ID
        """
        logger.debug(f"Brightness popup: monitor changed to {monitor_id}")
        self._current_monitor = monitor_id
        self.monitor_changed.emit(monitor_id)

    def _on_auto_hide_timeout(self) -> None:
        """Handle auto-hide timer timeout."""
        logger.debug("Brightness popup: auto-hide timeout")
        self.hide()

    def show_at(self, pos: QPoint) -> None:
        """
        Show the popup at the specified position.

        Args:
            pos: Position to show the popup at
        """
        # Move to position
        self.move(pos)

        # Show popup
        super().show()

        # Start auto-hide timer if enabled
        if self._auto_hide_ms > 0:
            self._auto_hide_timer.start(self._auto_hide_ms)

    def showEvent(self, event) -> None:
        """Handle show event."""
        super().showEvent(event)
        logger.debug("Brightness popup shown")

    def hideEvent(self, event) -> None:
        """Handle hide event."""
        super().hideEvent(event)
        self._auto_hide_timer.stop()
        logger.debug("Brightness popup hidden")

    def mousePressEvent(self, event) -> None:
        """Handle mouse press event."""
        # Close popup when clicking outside
        if not self._container.geometry().contains(event.pos()):
            self.hide()
        super().mousePressEvent(event)

    def enterEvent(self, event) -> None:
        """Handle mouse enter event."""
        # Pause auto-hide timer when mouse is inside
        if self._auto_hide_timer.isActive():
            self._auto_hide_timer.stop()
        super().enterEvent(event)

    def leaveEvent(self, event) -> None:
        """Handle mouse leave event."""
        # Restart auto-hide timer when mouse leaves
        if self._auto_hide_ms > 0:
            self._auto_hide_timer.start(self._auto_hide_ms)
        super().leaveEvent(event)

    def set_brightness(self, value: int) -> None:
        """
        Set the brightness value.

        Args:
            value: Brightness value (0-100)
        """
        self._brightness_slider.set_brightness(value)

    def get_brightness(self) -> int:
        """
        Get the current brightness value.

        Returns:
            Current brightness value (0-100)
        """
        return self._brightness_slider.get_brightness()

    def set_monitors(self, monitors: dict[str, str]) -> None:
        """
        Set the available monitors.

        Args:
            monitors: Dictionary mapping unique_id to display_name
        """
        self._monitors = monitors.copy()

        if self._monitor_selector is not None:
            current_selection = self._monitor_selector.currentText()
            self._monitor_selector.blockSignals(True)
            self._monitor_selector.clear()

            for unique_id, display_name in self._monitors.items():
                self._monitor_selector.addItem(display_name, unique_id)

            # Restore selection if possible
            if current_selection:
                index = self._monitor_selector.findText(current_selection)
                if index >= 0:
                    self._monitor_selector.setCurrentIndex(index)
                elif self._monitors:
                    self._monitor_selector.setCurrentIndex(0)
            elif self._monitors:
                self._monitor_selector.setCurrentIndex(0)

            self._monitor_selector.blockSignals(False)

            # Emit monitor changed signal for the current selection
            if self._monitor_selector.currentData():
                self._current_monitor = self._monitor_selector.currentData()
                self.monitor_changed.emit(self._current_monitor)

    def set_current_monitor(self, monitor_id: str) -> None:
        """
        Set the current monitor.

        Args:
            monitor_id: Monitor unique ID
        """
        if self._monitor_selector is not None:
            index = self._monitor_selector.findData(monitor_id)
            if index >= 0:
                self._monitor_selector.setCurrentIndex(index)
        self._current_monitor = monitor_id

    def get_current_monitor(self) -> str | None:
        """
        Get the current monitor ID.

        Returns:
            Current monitor unique ID, or None if no monitor selected
        """
        if self._monitor_selector is not None:
            return self._monitor_selector.currentData()
        return self._current_monitor

    def set_status(self, message: str, is_error: bool = False) -> None:
        """
        Set the status message.

        Args:
            message: Status message to display
            is_error: True if this is an error message
        """
        self._status_label.setText(message)
        if is_error:
            self._status_label.setProperty("error", True)
        else:
            self._status_label.setProperty("error", False)
        self._status_label.style().unpolish(self._status_label)
        self._status_label.style().polish(self._status_label)

    def clear_status(self) -> None:
        """Clear the status message."""
        self._status_label.setText("")
        self._status_label.setProperty("error", False)
        self._status_label.style().unpolish(self._status_label)
        self._status_label.style().polish(self._status_label)

    def set_enabled(self, enabled: bool) -> None:
        """
        Enable or disable the popup controls.

        Args:
            enabled: True to enable, False to disable
        """
        self._brightness_slider.set_enabled(enabled)
        for btn in self._preset_buttons:
            btn.setEnabled(enabled)
        if self._monitor_selector is not None:
            self._monitor_selector.setEnabled(enabled)
        # Enable/disable VCP controls
        for section in self._vcp_sections.values():
            section.set_enabled(enabled)

    def set_monitor_selector_visible(self, visible: bool) -> None:
        """
        Show or hide the monitor selector.

        Args:
            visible: True to show, False to hide
        """
        if self._monitor_selector is not None:
            self._monitor_selector.setVisible(visible)

    def set_auto_hide_ms(self, ms: int) -> None:
        """
        Set the auto-hide delay.

        Args:
            ms: Auto-hide delay in milliseconds (0 to disable)
        """
        self._auto_hide_ms = ms
        if self._auto_hide_ms == 0:
            self._auto_hide_timer.stop()

    def get_auto_hide_ms(self) -> int:
        """
        Get the auto-hide delay.

        Returns:
            Auto-hide delay in milliseconds
        """
        return self._auto_hide_ms

    def resizeEvent(self, event) -> None:
        """Handle resize event."""
        super().resizeEvent(event)
        # Update container size
        if self._container:
            self._container.resize(self.size())

    def set_vcp_value(self, vcp_code: int, value: int) -> None:
        """
        Set a VCP value programmatically.

        Args:
            vcp_code: The VCP code
            value: The value to set
        """
        if vcp_code == 0x10:
            self._brightness_slider.set_brightness(value)
        elif vcp_code == 0x12 and hasattr(self, "_contrast_slider"):
            self._contrast_slider.set_value(value)
        elif vcp_code == 0x14 and hasattr(self, "_color_temp_combo"):
            self._color_temp_combo.set_value(value)
        elif vcp_code == 0x60 and hasattr(self, "_input_combo"):
            self._input_combo.set_value(value)
        elif vcp_code == 0x62 and hasattr(self, "_volume_slider"):
            self._volume_slider.set_value(value)

    def get_vcp_value(self, vcp_code: int) -> int | None:
        """
        Get a VCP value.

        Args:
            vcp_code: The VCP code

        Returns:
            The current value, or None if the control doesn't exist
        """
        if vcp_code == 0x10:
            return self._brightness_slider.get_brightness()
        elif vcp_code == 0x12 and hasattr(self, "_contrast_slider"):
            return self._contrast_slider.get_value()
        elif vcp_code == 0x14 and hasattr(self, "_color_temp_combo"):
            return self._color_temp_combo.get_value()
        elif vcp_code == 0x60 and hasattr(self, "_input_combo"):
            return self._input_combo.get_value()
        elif vcp_code == 0x62 and hasattr(self, "_volume_slider"):
            return self._volume_slider.get_value()
        return None

    def set_enabled_vcp_codes(self, vcp_codes: list[int]) -> None:
        """
        Set which VCP codes are enabled in the popup.

        Args:
            vcp_codes: List of VCP codes to enable
        """
        self._enabled_vcp_codes = vcp_codes
        # Rebuild tabs to show/hide controls
        # Note: This would require rebuilding the UI, which is complex
        # For now, we'll just update the internal state
        logger.debug(f"Enabled VCP codes: {[f'0x{c:02X}' for c in vcp_codes]}")

    def get_enabled_vcp_codes(self) -> list[int]:
        """
        Get the list of enabled VCP codes.

        Returns:
            List of enabled VCP codes
        """
        return self._enabled_vcp_codes.copy()
