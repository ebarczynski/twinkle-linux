//! VCP control widgets for additional monitor settings.

use crate::ddc::vcp_codes::{get_vcp_info, ValueType};
use gtk4::prelude::*;
use gtk4::{Box, ComboBoxText, Label, Orientation, Scale};
use std::collections::HashMap;

/// VCP control section for a specific VCP code.
pub struct VCPControlSection {
    /// Container widget
    container: Box,
    /// VCP code
    vcp_code: u8,
    /// VCP info
    vcp_info: Option<crate::ddc::vcp_codes::VCPCodeInfo>,
    /// Current value
    current_value: u16,
}

impl VCPControlSection {
    /// Create a new VCP control section.
    pub fn new(vcp_code: u8) -> Self {
        let vcp_info = get_vcp_info(vcp_code);

        let container = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(4)
            .margin_top(4)
            .margin_bottom(4)
            .margin_start(4)
            .margin_end(4)
            .build();

        let mut section = Self {
            container,
            vcp_code,
            vcp_info,
            current_value: 0,
        };

        section.build_ui();
        section
    }

    /// Build the UI for this VCP control section.
    fn build_ui(&mut self) {
        let name = self
            .vcp_info
            .as_ref()
            .map(|info| info.name)
            .unwrap_or("Unknown");

        // Create label
        let label = Label::builder()
            .label(name)
            .halign(gtk4::Align::Start)
            .build();
        self.container.append(&label);

        // Create appropriate control based on value type
        if let Some(ref info) = self.vcp_info {
            match info.value_type {
                ValueType::Continuous => {
                    let adjustment = gtk4::Adjustment::new(
                        50.0,
                        info.min_value as f64,
                        info.max_value as f64,
                        1.0,
                        5.0,
                        0.0,
                    );

                    let scale = Scale::builder()
                        .orientation(Orientation::Horizontal)
                        .adjustment(&adjustment)
                        .hexpand(true)
                        .draw_value(true)
                        .build();

                    self.container.append(&scale);
                }
                ValueType::NonContinuous => {
                    let combo = ComboBoxText::new();

                    if let Some(ref values) = info.values {
                        for (&value, name) in values {
                            combo.append(Some(&format!("{:#04x}", value)), name);
                        }
                    }

                    combo.set_active(Some(0));
                    self.container.append(&combo);
                }
                _ => {
                    // Read-only or write-only, just show a label
                    let value_label = Label::builder()
                        .label(format!("Value: {}", self.current_value))
                        .halign(gtk4::Align::Start)
                        .build();
                    self.container.append(&value_label);
                }
            }
        }
    }

    /// Get the container widget.
    pub fn widget(&self) -> &Box {
        &self.container
    }

    /// Set the current value.
    pub fn set_value(&mut self, value: u16) {
        self.current_value = value;
        // Update the UI widget based on value
    }

    /// Get the current value.
    pub fn get_value(&self) -> u16 {
        self.current_value
    }

    /// Get the VCP code.
    pub fn vcp_code(&self) -> u8 {
        self.vcp_code
    }
}

/// Container for multiple VCP control sections.
pub struct VCPControlsContainer {
    /// Container widget
    container: Box,
    /// VCP sections by code
    sections: HashMap<u8, VCPControlSection>,
}

impl VCPControlsContainer {
    /// Create a new VCP controls container.
    pub fn new() -> Self {
        let container = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(8)
            .margin_top(8)
            .margin_bottom(8)
            .margin_start(8)
            .margin_end(8)
            .build();

        Self {
            container,
            sections: HashMap::new(),
        }
    }

    /// Add a VCP control section.
    pub fn add_section(&mut self, vcp_code: u8) {
        let section = VCPControlSection::new(vcp_code);
        self.container.append(section.widget());
        self.sections.insert(vcp_code, section);
    }

    /// Add multiple VCP control sections.
    pub fn add_sections(&mut self, vcp_codes: &[u8]) {
        for &code in vcp_codes {
            self.add_section(code);
        }
    }

    /// Set the value for a specific VCP code.
    pub fn set_value(&mut self, vcp_code: u8, value: u16) {
        if let Some(section) = self.sections.get_mut(&vcp_code) {
            section.set_value(value);
        }
    }

    /// Get the value for a specific VCP code.
    pub fn get_value(&self, vcp_code: u8) -> Option<u16> {
        self.sections.get(&vcp_code).map(|s| s.get_value())
    }

    /// Get the container widget.
    pub fn widget(&self) -> &Box {
        &self.container
    }

    /// Clear all sections.
    pub fn clear(&mut self) {
        while let Some(child) = self.container.first_child() {
            self.container.remove(&child);
        }
        self.sections.clear();
    }
}

impl Default for VCPControlsContainer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vcp_control_section_new() {
        let section = VCPControlSection::new(0x10);
        assert_eq!(section.vcp_code(), 0x10);
    }

    #[test]
    fn test_vcp_controls_container_new() {
        let container = VCPControlsContainer::new();
        assert_eq!(container.sections.len(), 0);
    }

    #[test]
    fn test_vcp_controls_container_add_section() {
        let mut container = VCPControlsContainer::new();
        container.add_section(0x10);
        assert_eq!(container.sections.len(), 1);
    }
}
