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

/// Whether this is a DDC/CI external monitor or an internal backlight display.
#[derive(Debug, Clone, PartialEq)]
pub enum MonitorType {
    /// External monitor controlled via DDC/CI over I2C
    External,
    /// Internal laptop display controlled via kernel backlight sysfs
    Internal,
}

/// Represents a physical monitor connected to the system.
#[derive(Debug, Clone)]
pub struct Monitor {
    /// I2C bus number for this monitor (external only)
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
    /// Monitor type: external DDC/CI or internal backlight
    pub monitor_type: MonitorType,
    /// Sysfs backlight path (internal only, e.g. /sys/class/backlight/intel_backlight)
    pub backlight_path: Option<String>,
}

impl Monitor {
    /// Create a new external DDC/CI Monitor.
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
            monitor_type: MonitorType::External,
            backlight_path: None,
        }
    }

    /// Create a new internal backlight Monitor.
    pub fn new_internal(name: &str, path: &str) -> Self {
        Self {
            bus: -1,
            model: name.to_string(),
            serial: String::new(),
            manufacturer: "Internal".to_string(),
            edid_data: String::new(),
            capabilities: MonitorCapabilities {
                supported_vcp_codes: HashSet::new(),
                max_brightness: 100,
                max_contrast: 100,
                supports_input_source: false,
                supports_power_control: false,
                supports_audio: false,
            },
            last_seen: Utc::now(),
            cached_values: std::collections::HashMap::new(),
            monitor_type: MonitorType::Internal,
            backlight_path: Some(path.to_string()),
        }
    }

    /// Validate monitor data.
    pub fn validate(&self) -> DDCResult<()> {
        if self.monitor_type == MonitorType::External && self.bus < 0 {
            return Err(DDCError::Other(format!("Invalid bus number: {}", self.bus)));
        }
        Ok(())
    }

    /// Get a human-readable display name for this monitor.
    pub fn display_name(&self) -> String {
        if self.monitor_type == MonitorType::Internal {
            return format!("{} (Internal)", self.model);
        }
        if !self.manufacturer.is_empty() && self.model != "Unknown Monitor" {
            if !self.serial.is_empty() && self.serial != "Unknown" {
                return format!("{} {} ({})", self.manufacturer, self.model, self.serial);
            }
            return format!("{} {}", self.manufacturer, self.model);
        }
        if self.model != "Unknown Monitor" {
            if !self.serial.is_empty() && self.serial != "Unknown" {
                return format!("{} ({})", self.model, self.serial);
            }
            return self.model.clone();
        }
        // Worst case: bus number
        format!("Monitor (bus {})", self.bus)
    }

    /// Get a unique identifier for this monitor.
    pub fn unique_id(&self) -> String {
        if self.monitor_type == MonitorType::Internal {
            return format!("internal_{}", self.model);
        }
        if !self.serial.is_empty() && self.serial != "Unknown" {
            return self.serial.clone();
        }
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
            "monitor_type": match self.monitor_type {
                MonitorType::External => "external",
                MonitorType::Internal => "internal",
            },
            "backlight_path": self.backlight_path,
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

    /// Detect all available monitors (both external DDC/CI and internal backlight).
    pub async fn detect_monitors(&self) -> DDCResult<Vec<Monitor>> {
        let mut monitors = Vec::new();

        // Detect internal backlight displays
        self._detect_internal_backlights(&mut monitors);

        // Detect external DDC/CI monitors
        tracing::info!("MonitorDetector::detect_monitors() - Acquiring executor lock");
        let mut executor = self.executor.lock().await;
        tracing::info!("MonitorDetector::detect_monitors() - Calling executor.detect_monitors()");
        // Use non-brief output for full monitor info
        let result = executor.detect_monitors().await?;

        drop(executor);
        tracing::info!("MonitorDetector::detect_monitors() - Released executor lock");

        tracing::info!("MonitorDetector::detect_monitors() - Command result: success={}, stdout_len={}",
            result.success, result.stdout.len());

        if !result.success {
            return Err(DDCError::CommandExecution(crate::ddc::error::CommandExecutionError {
                command: result.command,
                exit_code: result.return_code,
                stderr: result.stderr,
                stdout: result.stdout,
            }));
        }

        tracing::info!("MonitorDetector::detect_monitors() - Parsing output");
        let mut external = self._parse_detect_output(&result.stdout).await?;
        monitors.append(&mut external);

        Ok(monitors)
    }

    /// Detect internal backlight displays via /sys/class/backlight/.
    fn _detect_internal_backlights(&self, monitors: &mut Vec<Monitor>) {
        let backlight_dir = std::path::Path::new("/sys/class/backlight");
        if !backlight_dir.exists() {
            tracing::info!("No /sys/class/backlight directory — no internal displays");
            return;
        }

        let entries = match std::fs::read_dir(backlight_dir) {
            Ok(e) => e,
            Err(err) => {
                tracing::warn!("Cannot read /sys/class/backlight: {}", err);
                return;
            }
        };

        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            let path = entry.path().to_string_lossy().to_string();

            // Verify we can read brightness
            let brightness_file = format!("{}/brightness", path);
            let max_brightness_file = format!("{}/max_brightness", path);

            if !std::path::Path::new(&brightness_file).exists() {
                continue;
            }

            let max_brightness: u16 = std::fs::read_to_string(&max_brightness_file)
                .ok()
                .and_then(|s| s.trim().parse().ok())
                .unwrap_or(100);

            tracing::info!("Found internal backlight: {} (max={})", name, max_brightness);

            let mut monitor = Monitor::new_internal(&name, &path);
            monitor.capabilities.max_brightness = max_brightness;
            monitors.push(monitor);
        }
    }

    /// Parse the output from `ddcutil detect` (full output, not --brief).
    async fn _parse_detect_output(&self, output: &str) -> DDCResult<Vec<Monitor>> {
        tracing::info!("_parse_detect_output() - Starting to parse {} lines", output.lines().count());
        let mut monitors = Vec::new();

        // Regex patterns for full ddcutil detect output (no --brief).
        // Lines are already split by \n so no trailing \n in each line.
        let bus_re = Regex::new(r"I2C bus:\s*/dev/i2c-(\d+)").unwrap();
        // Full output: "Model:             ThinkVision T24i-20"
        let model_re = Regex::new(r"Model:\s*(.+)").unwrap();
        let serial_re = Regex::new(r"Serial number:\s*(.+)").unwrap();
        let mfg_re = Regex::new(r"Mfg id:\s*(.+)").unwrap();
        // Brief output fallback: "Monitor:           LEN T24i-20"
        let monitor_brief_re = Regex::new(r"Monitor:\s*(.+)").unwrap();

        let lines: Vec<&str> = output.lines().collect();
        let mut i = 0;

        tracing::info!("_parse_detect_output() - Starting line-by-line parsing");
        while i < lines.len() {
            let line = lines[i];

            if let Some(caps) = bus_re.captures(line) {
                let bus: i32 = caps.get(1).unwrap().as_str().parse().unwrap_or(-1);
                tracing::info!("_parse_detect_output() - Found monitor on bus {}", bus);
                let mut monitor = Monitor::new(bus);

                // Look for monitor info in following lines (up to 20 lines ahead)
                for j in (i + 1)..std::cmp::min(i + 20, lines.len()) {
                    let subj = lines[j];

                    // Stop if we hit the next display entry
                    if subj.starts_with("Display ") {
                        break;
                    }

                    // Try full format fields first
                    if let Some(caps) = model_re.captures(subj) {
                        let val = caps.get(1).unwrap().as_str().trim();
                        if !val.is_empty() {
                            monitor.model = val.to_string();
                        }
                    }
                    if let Some(caps) = serial_re.captures(subj) {
                        let val = caps.get(1).unwrap().as_str().trim();
                        if !val.is_empty() {
                            monitor.serial = val.to_string();
                        }
                    }
                    if let Some(caps) = mfg_re.captures(subj) {
                        let val = caps.get(1).unwrap().as_str().trim();
                        if !val.is_empty() {
                            monitor.manufacturer = val.to_string();
                        }
                    }
                    // Brief format fallback: "Monitor: LEN T24i-20"
                    if monitor.model == "Unknown Monitor" {
                        if let Some(caps) = monitor_brief_re.captures(subj) {
                            let val = caps.get(1).unwrap().as_str().trim();
                            if !val.is_empty() {
                                // Brief format is "MFG Model" — split on first space
                                if let Some(space_pos) = val.find(' ') {
                                    monitor.manufacturer = val[..space_pos].to_string();
                                    monitor.model = val[space_pos + 1..].trim().to_string();
                                } else {
                                    monitor.model = val.to_string();
                                }
                            }
                        }
                    }
                }

                tracing::info!("_parse_detect_output() - Getting capabilities for bus {}", bus);
                // Get capabilities for this monitor.
                // If capabilities fail, the monitor likely doesn't support DDC/CI
                // (e.g. internal laptop panels detected on I2C). Skip it — internal
                // displays are handled via /sys/class/backlight/ instead.
                match self._get_monitor_capabilities(bus).await {
                    Ok(capabilities) => {
                        monitor.capabilities = capabilities;
                        tracing::info!("_parse_detect_output() - Successfully retrieved capabilities for bus {}", bus);
                    }
                    Err(e) => {
                        tracing::warn!(
                            "_parse_detect_output() - Skipping monitor on bus {}: \
                             capabilities query failed ({}) — likely internal panel, \
                             use backlight instead",
                            bus, e
                        );
                        i += 1;
                        continue;
                    }
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

        // Parse supported VCP codes.
        // ddcutil capabilities output uses "Feature: XX" format, e.g.:
        //   Feature: 10 (Brightness)
        //   Feature: 12 (Contrast)
        let vcp_re = Regex::new(r"Feature:\s*([0-9A-Fa-f]{2})").unwrap();
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
        assert_eq!(monitor.monitor_type, MonitorType::External);
    }

    #[test]
    fn test_monitor_new_internal() {
        let monitor = Monitor::new_internal("intel_backlight", "/sys/class/backlight/intel_backlight");
        assert_eq!(monitor.model, "intel_backlight");
        assert_eq!(monitor.monitor_type, MonitorType::Internal);
        assert_eq!(monitor.backlight_path, Some("/sys/class/backlight/intel_backlight".to_string()));
    }

    #[test]
    fn test_monitor_display_name() {
        let mut monitor = Monitor::new(1);
        monitor.model = "T24i-20".to_string();
        monitor.manufacturer = "LEN".to_string();
        assert_eq!(monitor.display_name(), "LEN T24i-20");

        monitor.serial = "ABC123".to_string();
        assert_eq!(monitor.display_name(), "LEN T24i-20 (ABC123)");
    }

    #[test]
    fn test_monitor_display_name_internal() {
        let monitor = Monitor::new_internal("intel_backlight", "/sys/class/backlight/intel_backlight");
        assert_eq!(monitor.display_name(), "intel_backlight (Internal)");
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
    fn test_monitor_unique_id_internal() {
        let monitor = Monitor::new_internal("intel_backlight", "/sys/class/backlight/intel_backlight");
        assert_eq!(monitor.unique_id(), "internal_intel_backlight");
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

    #[test]
    fn test_parse_detect_output_regex() {
        // Verify the regex works on lines without trailing \n
        let model_re = Regex::new(r"Model:\s*(.+)").unwrap();
        let line = "      Model:             ThinkVision T24i-20";
        let caps = model_re.captures(line).unwrap();
        assert_eq!(caps.get(1).unwrap().as_str().trim(), "ThinkVision T24i-20");

        let mfg_re = Regex::new(r"Mfg id:\s*(.+)").unwrap();
        let line = "      Mfg id:            LEN";
        let caps = mfg_re.captures(line).unwrap();
        assert_eq!(caps.get(1).unwrap().as_str().trim(), "LEN");

        // Brief format
        let brief_re = Regex::new(r"Monitor:\s*(.+)").unwrap();
        let line = "   Monitor:             LEN T24i-20";
        let caps = brief_re.captures(line).unwrap();
        assert_eq!(caps.get(1).unwrap().as_str().trim(), "LEN T24i-20");
    }
}
