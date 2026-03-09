"""
VCP Control Widgets.

This module provides reusable widgets for controlling VCP codes,
including sliders for continuous values and combo boxes for discrete values.
"""

from __future__ import annotations

import logging
from typing import TYPE_CHECKING

from PyQt6.QtCore import Qt, QTimer, pyqtSignal
from PyQt6.QtWidgets import (
    QWidget,
    QVBoxLayout,
    QHBoxLayout,
    QSlider,
    QLabel,
    QComboBox,
    QFrame,
)

if TYPE_CHECKING:
    from collections.abc import Callable

from src.ddc.vcp_codes import VCPCodeInfo, get_vcp_info, get_value_name

logger = logging.getLogger(__name__)


class VCPSlider(QWidget):
    """
    A slider widget for controlling continuous VCP values.

    This widget provides a slider for controlling VCP codes with continuous
    values (e.g., brightness, contrast, volume) with debouncing to prevent
    rapid DDC/CI calls.
    """

    # Signal emitted when value changes (after debounce)
    value_changed = pyqtSignal(int)

    # Signal emitted when value changes immediately (no debounce)
    value_changed_immediate = pyqtSignal(int)

    def __init__(
        self,
        vcp_code: int,
        parent: QWidget | None = None,
        debounce_ms: int = 200,
        show_value_label: bool = True,
    ) -> None:
        """
        Initialize the VCP slider.

        Args:
            vcp_code: The VCP code this slider controls
            parent: Parent widget
            debounce_ms: Debounce delay in milliseconds
            show_value_label: Whether to show the current value label
        """
        super().__init__(parent)

        self._vcp_code = vcp_code
        self._debounce_ms = debounce_ms
        self._show_value_label = show_value_label
        self._pending_value: int | None = None

        # Get VCP info
        self._vcp_info = get_vcp_info(vcp_code)

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

        # Create header with label and value
        header_layout = QHBoxLayout()
        header_layout.setSpacing(8)

        # Label
        self._label = QLabel(self._vcp_info.name if self._vcp_info else f"VCP 0x{self._vcp_code:02X}")
        self._label.setObjectName("vcpControlLabel")
        header_layout.addWidget(self._label)

        # Value label
        if self._show_value_label:
            self._value_label = QLabel("50%")
            self._value_label.setObjectName("vcpControlValueLabel")
            self._value_label.setAlignment(Qt.AlignmentFlag.AlignRight)
            header_layout.addWidget(self._value_label)
        else:
            self._value_label = None

        header_layout.addStretch()
        layout.addLayout(header_layout)

        # Create slider
        self._slider = QSlider(Qt.Orientation.Horizontal)
        self._slider.setObjectName("vcpSlider")

        # Set range based on VCP info
        if self._vcp_info:
            self._slider.setMinimum(self._vcp_info.min_value)
            self._slider.setMaximum(self._vcp_info.max_value)
        else:
            self._slider.setMinimum(0)
            self._slider.setMaximum(100)

        self._slider.setValue((self._slider.minimum() + self._slider.maximum()) // 2)
        self._slider.setSingleStep(1)
        self._slider.setPageStep(10)
        self._slider.setTickPosition(QSlider.TickPosition.TicksBelow)
        self._slider.setTickInterval((self._slider.maximum() - self._slider.minimum()) // 4)

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
        # Update value label
        if self._value_label is not None:
            self._value_label.setText(f"{value}%")

        # Store pending value
        self._pending_value = value

        # Emit immediate signal for UI updates
        self.value_changed_immediate.emit(value)

        # Restart debounce timer
        self._debounce_timer.start(self._debounce_ms)

    def _on_debounce_timeout(self) -> None:
        """Handle debounce timer timeout."""
        if self._pending_value is not None:
            value = self._pending_value
            self._pending_value = None
            logger.debug(f"Debounced VCP 0x{self._vcp_code:02X} change: {value}")
            self.value_changed.emit(value)

    def _on_slider_pressed(self) -> None:
        """Handle slider press event."""
        # Stop any pending debounce timer
        self._debounce_timer.stop()

    def _on_slider_released(self) -> None:
        """Handle slider release event."""
        # Emit final value immediately
        if self._pending_value is not None:
            value = self._pending_value
            self._pending_value = None
            self._debounce_timer.stop()
            logger.debug(f"Slider released, emitting VCP 0x{self._vcp_code:02X}: {value}")
            self.value_changed.emit(value)

    def set_value(self, value: int) -> None:
        """
        Set the value programmatically.

        Args:
            value: Value to set
        """
        # Clamp to valid range
        min_val = self._slider.minimum()
        max_val = self._slider.maximum()
        value = max(min_val, min(max_val, value))

        self._slider.blockSignals(True)
        self._slider.setValue(value)
        if self._value_label is not None:
            self._value_label.setText(f"{value}%")
        self._slider.blockSignals(False)

    def get_value(self) -> int:
        """
        Get the current value.

        Returns:
            Current value
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

    def set_enabled(self, enabled: bool) -> None:
        """
        Enable or disable the slider.

        Args:
            enabled: True to enable, False to disable
        """
        self._slider.setEnabled(enabled)
        if self._label:
            self._label.setEnabled(enabled)
        if self._value_label:
            self._value_label.setEnabled(enabled)


class VCPComboBox(QWidget):
    """
    A combo box widget for controlling discrete VCP values.

    This widget provides a combo box for controlling VCP codes with discrete
    values (e.g., input source, color temperature).
    """

    # Signal emitted when value changes
    value_changed = pyqtSignal(int)

    def __init__(
        self,
        vcp_code: int,
        parent: QWidget | None = None,
    ) -> None:
        """
        Initialize the VCP combo box.

        Args:
            vcp_code: The VCP code this combo box controls
            parent: Parent widget
        """
        super().__init__(parent)

        self._vcp_code = vcp_code

        # Get VCP info
        self._vcp_info = get_vcp_info(vcp_code)

        # Set up UI
        self._setup_ui()

    def _setup_ui(self) -> None:
        """Set up the combo box UI components."""
        layout = QVBoxLayout(self)
        layout.setContentsMargins(0, 0, 0, 0)
        layout.setSpacing(4)

        # Create header with label
        header_layout = QHBoxLayout()
        header_layout.setSpacing(8)

        # Label
        self._label = QLabel(self._vcp_info.name if self._vcp_info else f"VCP 0x{self._vcp_code:02X}")
        self._label.setObjectName("vcpControlLabel")
        header_layout.addWidget(self._label)

        header_layout.addStretch()
        layout.addLayout(header_layout)

        # Create combo box
        self._combo_box = QComboBox()
        self._combo_box.setObjectName("vcpComboBox")

        # Populate with values from VCP info
        if self._vcp_info and self._vcp_info.values:
            for value, name in sorted(self._vcp_info.values.items()):
                self._combo_box.addItem(name, value)
        else:
            # Fallback: add numeric values
            self._combo_box.addItem("0", 0)
            self._combo_box.addItem("1", 1)

        # Connect combo box signal
        self._combo_box.currentIndexChanged.connect(self._on_current_index_changed)

        layout.addWidget(self._combo_box)

    def _on_current_index_changed(self, index: int) -> None:
        """
        Handle combo box index change.

        Args:
            index: New index
        """
        if index >= 0:
            value = self._combo_box.currentData()
            if value is not None:
                logger.debug(f"VCP 0x{self._vcp_code:02X} combo box changed to: {value}")
                self.value_changed.emit(value)

    def set_value(self, value: int) -> None:
        """
        Set the value programmatically.

        Args:
            value: Value to set
        """
        index = self._combo_box.findData(value)
        if index >= 0:
            self._combo_box.blockSignals(True)
            self._combo_box.setCurrentIndex(index)
            self._combo_box.blockSignals(False)

    def get_value(self) -> int | None:
        """
        Get the current value.

        Returns:
            Current value, or None if no selection
        """
        return self._combo_box.currentData()

    def set_enabled(self, enabled: bool) -> None:
        """
        Enable or disable the combo box.

        Args:
            enabled: True to enable, False to disable
        """
        self._combo_box.setEnabled(enabled)
        if self._label:
            self._label.setEnabled(enabled)


class VCPControlSection(QFrame):
    """
    A section widget containing VCP controls for a specific category.

    This widget groups related VCP controls together (e.g., Display, Audio, Input).
    """

    def __init__(
        self,
        title: str,
        parent: QWidget | None = None,
    ) -> None:
        """
        Initialize the VCP control section.

        Args:
            title: Section title
            parent: Parent widget
        """
        super().__init__(parent)

        self._title = title
        self._controls: dict[int, QWidget] = {}

        # Set up UI
        self._setup_ui()

    def _setup_ui(self) -> None:
        """Set up the section UI components."""
        self.setObjectName("vcpControlSection")

        layout = QVBoxLayout(self)
        layout.setContentsMargins(0, 0, 0, 0)
        layout.setSpacing(8)

        # Section title
        title_label = QLabel(self._title)
        title_label.setObjectName("vcpSectionTitle")
        layout.addWidget(title_label)

        # Controls container
        self._controls_container = QWidget()
        self._controls_layout = QVBoxLayout(self._controls_container)
        self._controls_layout.setContentsMargins(0, 0, 0, 0)
        self._controls_layout.setSpacing(8)
        layout.addWidget(self._controls_container)

    def add_slider(
        self,
        vcp_code: int,
        debounce_ms: int = 200,
        show_value_label: bool = True,
    ) -> VCPSlider:
        """
        Add a slider control for a VCP code.

        Args:
            vcp_code: The VCP code
            debounce_ms: Debounce delay in milliseconds
            show_value_label: Whether to show the current value label

        Returns:
            The created VCPSlider widget
        """
        slider = VCPSlider(
            vcp_code=vcp_code,
            parent=self._controls_container,
            debounce_ms=debounce_ms,
            show_value_label=show_value_label,
        )
        self._controls[vcp_code] = slider
        self._controls_layout.addWidget(slider)
        return slider

    def add_combo_box(self, vcp_code: int) -> VCPComboBox:
        """
        Add a combo box control for a VCP code.

        Args:
            vcp_code: The VCP code

        Returns:
            The created VCPComboBox widget
        """
        combo_box = VCPComboBox(
            vcp_code=vcp_code,
            parent=self._controls_container,
        )
        self._controls[vcp_code] = combo_box
        self._controls_layout.addWidget(combo_box)
        return combo_box

    def get_control(self, vcp_code: int) -> QWidget | None:
        """
        Get a control widget by VCP code.

        Args:
            vcp_code: The VCP code

        Returns:
            The control widget, or None if not found
        """
        return self._controls.get(vcp_code)

    def remove_control(self, vcp_code: int) -> None:
        """
        Remove a control widget by VCP code.

        Args:
            vcp_code: The VCP code
        """
        control = self._controls.pop(vcp_code, None)
        if control:
            self._controls_layout.removeWidget(control)
            control.deleteLater()

    def set_enabled(self, enabled: bool) -> None:
        """
        Enable or disable all controls in this section.

        Args:
            enabled: True to enable, False to disable
        """
        for control in self._controls.values():
            if hasattr(control, "set_enabled"):
                control.set_enabled(enabled)
