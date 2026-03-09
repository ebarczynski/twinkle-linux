"""
Brightness Slider Widget.

This module provides the BrightnessSlider class, a custom QSlider widget
for controlling monitor brightness with debouncing and visual feedback.
"""

from __future__ import annotations

import logging
from typing import TYPE_CHECKING

from PyQt6.QtCore import Qt, QTimer, pyqtSignal
from PyQt6.QtWidgets import QSlider, QLabel, QVBoxLayout, QWidget

if TYPE_CHECKING:
    from collections.abc import Callable

logger = logging.getLogger(__name__)


class BrightnessSlider(QWidget):
    """
    A brightness slider widget with debouncing and visual feedback.

    This widget provides a slider for controlling monitor brightness
    with debouncing to prevent rapid DDC/CI calls and visual feedback
    for the current brightness level.
    """

    # Signal emitted when brightness changes (after debounce)
    brightness_changed = pyqtSignal(int)

    # Signal emitted when brightness changes immediately (no debounce)
    brightness_changed_immediate = pyqtSignal(int)

    def __init__(
        self,
        parent: QWidget | None = None,
        debounce_ms: int = 200,
        show_value_label: bool = True,
    ) -> None:
        """
        Initialize the brightness slider.

        Args:
            parent: Parent widget
            debounce_ms: Debounce delay in milliseconds
            show_value_label: Whether to show the current value label
        """
        super().__init__(parent)

        self._debounce_ms = debounce_ms
        self._show_value_label = show_value_label
        self._pending_brightness: int | None = None

        # Create debounce timer
        self._debounce_timer = QTimer(self)
        self._debounce_timer.setSingleShot(True)
        self._debounce_timer.timeout.connect(self._on_debounce_timeout)

        # Set up UI
        self._setup_ui()

    def _setup_ui(self) -> None:
        """Set up the slider UI components."""
        layout = QVBoxLayout(self)
        layout.setContentsMargins(0, 0, 0, 0)
        layout.setSpacing(4)

        # Create value label
        if self._show_value_label:
            self._value_label = QLabel("Brightness: 50%")
            self._value_label.setAlignment(Qt.AlignmentFlag.AlignCenter)
            layout.addWidget(self._value_label)
        else:
            self._value_label = None

        # Create slider
        self._slider = QSlider(Qt.Orientation.Horizontal)
        self._slider.setMinimum(0)
        self._slider.setMaximum(100)
        self._slider.setValue(50)
        self._slider.setSingleStep(1)
        self._slider.setPageStep(10)
        self._slider.setTickPosition(QSlider.TickPosition.TicksBelow)
        self._slider.setTickInterval(25)

        # Connect slider signals
        self._slider.valueChanged.connect(self._on_slider_value_changed)
        self._slider.sliderPressed.connect(self._on_slider_pressed)
        self._slider.sliderReleased.connect(self._on_slider_released)

        layout.addWidget(self._slider)

    def _on_slider_value_changed(self, value: int) -> None:
        """
        Handle slider value changes.

        Args:
            value: New slider value
        """
        # Update label
        if self._value_label is not None:
            self._value_label.setText(f"Brightness: {value}%")

        # Store pending value
        self._pending_brightness = value

        # Emit immediate signal for UI updates
        self.brightness_changed_immediate.emit(value)

        # Restart debounce timer
        self._debounce_timer.start(self._debounce_ms)

    def _on_debounce_timeout(self) -> None:
        """Handle debounce timer timeout."""
        if self._pending_brightness is not None:
            brightness = self._pending_brightness
            self._pending_brightness = None
            logger.debug(f"Debounced brightness change: {brightness}")
            self.brightness_changed.emit(brightness)

    def _on_slider_pressed(self) -> None:
        """Handle slider press event."""
        # Stop any pending debounce timer
        self._debounce_timer.stop()

    def _on_slider_released(self) -> None:
        """Handle slider release event."""
        # Emit final value immediately
        if self._pending_brightness is not None:
            brightness = self._pending_brightness
            self._pending_brightness = None
            self._debounce_timer.stop()
            logger.debug(f"Slider released, emitting brightness: {brightness}")
            self.brightness_changed.emit(brightness)

    def set_brightness(self, value: int) -> None:
        """
        Set the brightness value programmatically.

        Args:
            value: Brightness value (0-100)
        """
        value = max(0, min(100, value))
        self._slider.blockSignals(True)
        self._slider.setValue(value)
        if self._value_label is not None:
            self._value_label.setText(f"Brightness: {value}%")
        self._slider.blockSignals(False)

    def get_brightness(self) -> int:
        """
        Get the current brightness value.

        Returns:
            Current brightness value (0-100)
        """
        return self._slider.value()

    def set_debounce_ms(self, ms: int) -> None:
        """
        Set the debounce delay.

        Args:
            ms: Debounce delay in milliseconds
        """
        self._debounce_ms = ms

    def get_debounce_ms(self) -> int:
        """
        Get the debounce delay.

        Returns:
            Debounce delay in milliseconds
        """
        return self._debounce_ms

    def set_show_value_label(self, show: bool) -> None:
        """
        Set whether to show the value label.

        Args:
            show: True to show the label, False to hide it
        """
        if self._show_value_label != show:
            self._show_value_label = show
            if show and self._value_label is None:
                # Create label
                self._value_label = QLabel(f"Brightness: {self._slider.value()}%")
                self._value_label.setAlignment(Qt.AlignmentFlag.AlignCenter)
                self.layout().insertWidget(0, self._value_label)
            elif not show and self._value_label is not None:
                # Remove label
                self.layout().removeWidget(self._value_label)
                self._value_label.deleteLater()
                self._value_label = None

    def get_show_value_label(self) -> bool:
        """
        Get whether the value label is shown.

        Returns:
            True if the label is shown, False otherwise
        """
        return self._show_value_label

    def set_enabled(self, enabled: bool) -> None:
        """
        Enable or disable the slider.

        Args:
            enabled: True to enable, False to disable
        """
        self._slider.setEnabled(enabled)
        if self._value_label is not None:
            self._value_label.setEnabled(enabled)

    def is_enabled(self) -> bool:
        """
        Check if the slider is enabled.

        Returns:
            True if enabled, False otherwise
        """
        return self._slider.isEnabled()
