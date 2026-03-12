//! Monitor detection and management for DDC/CI operations.

use crate::ddc::command::CommandExecutor;
use crate::ddc::error::{DDCError, DDCResult};
use crate::ddc::vcp_codes::get_vcp_info;
use chrono::{DateTime, Utc};
use regex::Regex;
use std::collections::HashSet;
use std::sync::Arc;

/// Capabilities of a monitor.
#[derive(Debug, Clone)]
pub struct MonitorCapabilities {
    /// Set of VCP codes supported by the monitor
    pub supported_vcp_codes: HashSet<u8>,
    /// Maximum brightness value
    pub max_brightness: u16,
    /// Maximum contrast value
    pub max_contrast: u16,
    /// Whether input source selection is supported
    pub supports_input_source: bool,
    /// Whether power control is supported
    pub supports_power_control: bool,
    /// Whether audio controls are supported
    pub supports_audio: bool,
}

impl Default for MonitorCapabilities {
    fn default() -> Self {
        Self {
            supported_vcp_codes: HashSet::new(),
            max_brightness: 100,
            max_contrast: 100,
            supports_input_source: false,
            supports_power_control: false,
            supports_audio: false,
        }
    }
}

impl MonitorCapabilities {
    /// Check if a VCP code is supported.
    pub fn supports_vcp(&self, vcp_code: u8) -> bool {
        self.supported_vcp_codes.contains(&vcp_code)
    }
}

/// Represents a physical monitor connected to the system.
#[derive(Debug, Clone)]
pub struct Monitor {
    /// I2C bus number for this monitor
    pub bus: i32,
    /// Monitor model name
    pub model: String,
    /// Monitor serial number
    pub serial: String,
    /// Monitor manufacturer name
    pub manufacturer: String,
    /// Raw EDID data from the monitor
    pub edid_data: String,
    /// Monitor capabilities
    pub capabilities: MonitorCapabilities,
    /// Timestamp when this monitor was last detected
    pub last_seen: DateTime<Utc>,
    /// Cached VCP values
    cached_values: std::collections::HashMap<u8, u16>,
}

impl Monitor {
    /// Create a new Monitor.
    pub fn new(bus: i32) -> Self {
        Self {
            bus,
            model: "Unknown Monitor".to_string(),
            serial: String::new(),
            manufacturer: String::new(),
            edid_data: String::new(),
            capabilities: MonitorCapabilities::default(),
            last_seen: Utc::now(),
            cached_values: std::collections::HashMap::new(),
        }
    }

    /// Validate monitor data.
    pub fn validate(&self) -> DDCResult<()> {
        if self.bus < 0 {
            return Err(DDCError::Other(format!("Invalid bus number: {}", self.bus)));
        }
        Ok(())
    }

    /// Get a human-readable display name for this monitor.
    pub fn display_name(&self) -> String {
        if !self.serial.is_empty() && self.serial != "Unknown" {
            return format!("{} ({})", self.model, self.serial);
        }
        self.model.clone()
    }

    /// Get a unique identifier for this monitor.
    pub fn unique_id(&self) -> String {
        if !self.serial.is_empty() && self.serial != "Unknown" {
            return self.serial.clone();
        }
        // Fallback to model + bus if serial is not available
        format!("{}_bus{}", self.model, self.bus)
    }

    /// Get a cached VCP value.
    pub fn get_cached_value(&self, vcp_code: u8) -> Option<u16> {
        self.cached_values.get(&vcp_code).copied()
    }

    /// Cache a VCP value.
    pub fn set_cached_value(&mut self, vcp_code: u8, value: u16) {
        self.cached_values.insert(vcp_code, value);
    }

    /// Clear all cached VCP values.
    pub fn clear_cache(&mut self) {
        self.cached_values.clear();
    }

    /// Invalidate a specific cached VCP value.
    pub fn invalidate_cache(&mut self, vcp_code: u8) {
        self.cached_values.remove(&vcp_code);
    }

    /// Convert monitor to a dictionary-like structure for serialization.
    pub fn to_dict(&self) -> serde_json::Value {
        serde_json::json!({
            "bus": self.bus,
            "model": self.model,
            "serial": self.serial,
            "manufacturer": self.manufacturer,
            "display_name": self.display_name(),
            "unique_id": self.unique_id(),
            "last_seen": self.last_seen.to_rfc3339(),
            "capabilities": {
                "supported_vcp_codes": self.capabilities.supported_vcp_codes.iter().copied().collect::<Vec<_>>(),
                "max_brightness": self.capabilities.max_brightness,
                "max_contrast": self.capabilities.max_contrast,
                "supports_input_source": self.capabilities.supports_input_source,
                "supports_power_control": self.capabilities.supports_power_control,
                "supports_audio": self.capabilities.supports_audio,
            },
        })
    }
}

/// Detector for finding monitors on the system.
pub struct MonitorDetector {
    executor: Arc<tokio::sync::Mutex<CommandExecutor>>,
}

impl MonitorDetector {
    /// Create a new MonitorDetector.
    pub fn new(executor: Arc<tokio::sync::Mutex<CommandExecutor>>) -> Self {
        Self { executor }
    }

    /// Detect all available monitors.
    pub async fn detect_monitors(&self) -> DDCResult<Vec<Monitor>> {
        tracing::info!("MonitorDetector::detect_monitors() - Acquiring executor lock");
        let mut executor = self.executor.lock().await;
        tracing::info!("MonitorDetector::detect_monitors() - Calling executor.detect_monitors()");
        let result = executor.detect_monitors().await?;
        
        // Release the lock before parsing to avoid deadlock
        drop(executor);
        tracing::info!("MonitorDetector::detect_monitors() - Released executor lock");

        tracing::info!("MonitorDetector::detect_monitors() - Command result: success={}, stdout_len={}",
            result.success, result.stdout.len());
        
        if !result.success {
            return Err(DDCError::CommandExecution(crate::ddc::error::CommandExecutionError {
                command: result.command,
                exit_code: result.return_code,
                stderr: result.stderr,
            }));
        }

        tracing::info!("MonitorDetector::detect_monitors() - Parsing output");
        self._parse_detect_output(&result.stdout).await
    }

    /// Parse the output from `ddcutil detect --brief`.
    async fn _parse_detect_output(&self, output: &str) -> DDCResult<Vec<Monitor>> {
        tracing::info!("_parse_detect_output() - Starting to parse {} lines", output.lines().count());
        let mut monitors = Vec::new();
        let bus_re = Regex::new(r"I2C bus:\s*/dev/i2c-(\d+)").unwrap();
        let model_re = Regex::new(r"Model:\s*(.+?)\s*\n").unwrap();
        let serial_re = Regex::new(r"Serial number:\s*(.+?)\s*\n").unwrap();
        let manufacturer_re = Regex::new(r"Manufacturing ID:\s*(.+?)\s*\n").unwrap();

        let lines: Vec<&str> = output.lines().collect();
        let mut i = 0;

        tracing::info!("_parse_detect_output() - Starting line-by-line parsing");
        while i < lines.len() {
            let line = lines[i];

            if let Some(caps) = bus_re.captures(line) {
                let bus: i32 = caps.get(1).unwrap().as_str().parse().unwrap_or(-1);
                tracing::info!("_parse_detect_output() - Found monitor on bus {}", bus);
                let mut monitor = Monitor::new(bus);

                // Look for monitor info in following lines
                for j in (i + 1)..std::cmp::min(i + 20, lines.len()) {
                    if let Some(caps) = model_re.captures(lines[j]) {
                        monitor.model = caps.get(1).unwrap().as_str().trim().to_string();
                    }
                    if let Some(caps) = serial_re.captures(lines[j]) {
                        monitor.serial = caps.get(1).unwrap().as_str().trim().to_string();
                    }
                    if let Some(caps) = manufacturer_re.captures(lines[j]) {
                        monitor.manufacturer = caps.get(1).unwrap().as_str().trim().to_string();
                    }
                }

                tracing::info!("_parse_detect_output() - Getting capabilities for bus {}", bus);
                // Get capabilities for this monitor
                if let Ok(capabilities) = self._get_monitor_capabilities(bus).await {
                    monitor.capabilities = capabilities;
                    tracing::info!("_parse_detect_output() - Successfully retrieved capabilities for bus {}", bus);
                } else {
                    tracing::warn!("_parse_detect_output() - Failed to get capabilities for bus {}", bus);
                }

                let display_name = monitor.display_name();
                monitors.push(monitor);
                tracing::info!("_parse_detect_output() - Added monitor {} to list (total: {})",
                    display_name, monitors.len());
            }

            i += 1;
        }

        tracing::info!("_parse_detect_output() - Parsing complete, found {} monitors", monitors.len());
        Ok(monitors)
    }

    /// Get capabilities for a specific monitor.
    async fn _get_monitor_capabilities(&self, bus: i32) -> DDCResult<MonitorCapabilities> {
        tracing::info!("_get_monitor_capabilities() - Acquiring executor lock for bus {}", bus);
        let mut executor = self.executor.lock().await;
        tracing::info!("_get_monitor_capabilities() - Calling get_capabilities for bus {}", bus);
        let result = executor.get_capabilities(bus).await?;
        tracing::info!("_get_monitor_capabilities() - get_capabilities completed for bus {}, success={}",
            bus, result.success);

        // Release the lock before parsing
        drop(executor);
        tracing::info!("_get_monitor_capabilities() - Released executor lock for bus {}", bus);

        if !result.success {
            tracing::warn!("_get_monitor_capabilities() - get_capabilities failed for bus {}, using defaults", bus);
            return Ok(MonitorCapabilities::default());
        }

        tracing::info!("_get_monitor_capabilities() - Parsing capabilities output for bus {}", bus);
        self._parse_capabilities_output(&result.stdout)
    }

    /// Parse the capabilities output.
    fn _parse_capabilities_output(&self, output: &str) -> DDCResult<MonitorCapabilities> {
        tracing::info!("_parse_capabilities_output() - Parsing capabilities from {} bytes", output.len());
        let mut capabilities = MonitorCapabilities::default();

        // Parse supported VCP codes
        let vcp_re = Regex::new(r"VCP code ([0-9A-Fa-f]{2})").unwrap();
        let mut vcp_count = 0;
        for caps in vcp_re.captures_iter(output) {
            if let Some(code_str) = caps.get(1) {
                if let Ok(code) = u8::from_str_radix(code_str.as_str(), 16) {
                    capabilities.supported_vcp_codes.insert(code);
                    vcp_count += 1;
                }
            }
        }
        tracing::info!("_parse_capabilities_output() - Found {} VCP codes", vcp_count);

        // Check for specific capabilities based on VCP codes
        capabilities.supports_input_source = capabilities.supports_vcp(0x60);
        capabilities.supports_power_control = capabilities.supports_vcp(0xD6);
        capabilities.supports_audio = capabilities.supports_vcp(0x62) || capabilities.supports_vcp(0x8D);

        // Get max brightness and contrast from VCP info if available
        if let Some(info) = get_vcp_info(0x10) {
            capabilities.max_brightness = info.max_value;
        }
        if let Some(info) = get_vcp_info(0x12) {
            capabilities.max_contrast = info.max_value;
        }

        tracing::info!("_parse_capabilities_output() - Capabilities parsed: input_source={}, power_control={}, audio={}, max_brightness={}, max_contrast={}",
            capabilities.supports_input_source, capabilities.supports_power_control,
            capabilities.supports_audio, capabilities.max_brightness, capabilities.max_contrast);
        Ok(capabilities)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monitor_new() {
        let monitor = Monitor::new(1);
        assert_eq!(monitor.bus, 1);
        assert_eq!(monitor.model, "Unknown Monitor");
    }

    #[test]
    fn test_monitor_display_name() {
        let mut monitor = Monitor::new(1);
        monitor.model = "Test Monitor".to_string();
        monitor.serial = "ABC123".to_string();
        assert_eq!(monitor.display_name(), "Test Monitor (ABC123)");
    }

    #[test]
    fn test_monitor_unique_id() {
        let mut monitor = Monitor::new(1);
        monitor.serial = "ABC123".to_string();
        assert_eq!(monitor.unique_id(), "ABC123");

        monitor.serial = String::new();
        monitor.model = "Test Monitor".to_string();
        assert_eq!(monitor.unique_id(), "Test Monitor_bus1");
    }

    #[test]
    fn test_monitor_cached_values() {
        let mut monitor = Monitor::new(1);
        assert!(monitor.get_cached_value(0x10).is_none());

        monitor.set_cached_value(0x10, 50);
        assert_eq!(monitor.get_cached_value(0x10), Some(50));

        monitor.invalidate_cache(0x10);
        assert!(monitor.get_cached_value(0x10).is_none());
    }

    #[test]
    fn test_monitor_capabilities_supports_vcp() {
        let mut capabilities = MonitorCapabilities::default();
        assert!(!capabilities.supports_vcp(0x10));

        capabilities.supported_vcp_codes.insert(0x10);
        assert!(capabilities.supports_vcp(0x10));
    }
}
