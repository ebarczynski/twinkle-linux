//! DDC/CI Manager - Main interface for DDC/CI operations.

use crate::ddc::command::CommandExecutor;
use crate::ddc::error::{is_permission_error, DDCError, DDCResult};
use crate::ddc::monitor::{Monitor, MonitorDetector};
use crate::ddc::vcp_codes::{get_common_vcp_codes, get_vcp_info};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Default cache TTL in seconds.
const DEFAULT_CACHE_TTL_SECS: f64 = 5.0;

/// Main interface for DDC/CI operations.
///
/// This struct provides a high-level API for interacting with monitors
/// via the DDC/CI protocol. It handles monitor detection, VCP value
/// reading and writing, and provides caching for improved performance.
pub struct DDCManager {
    /// Command executor for ddcutil commands
    executor: Arc<tokio::sync::Mutex<CommandExecutor>>,
    /// Monitor detector
    detector: MonitorDetector,
    /// Cache TTL in seconds
    cache_ttl: f64,
    /// Detected monitors, indexed by unique ID
    monitors: Arc<RwLock<HashMap<String, Monitor>>>,
    /// Whether the manager has been initialized
    initialized: Arc<RwLock<bool>>,
    /// Cache timestamps for VCP values
    #[allow(clippy::type_complexity)]
    cache_timestamps: Arc<RwLock<HashMap<String, HashMap<u8, chrono::DateTime<chrono::Utc>>>>>,
}

impl DDCManager {
    /// Create a new DDCManager with default settings.
    pub async fn new() -> DDCResult<Self> {
        tracing::info!("DDCManager::new() called");

        let executor = Arc::new(tokio::sync::Mutex::new(CommandExecutor::new()));
        tracing::info!("CommandExecutor created");

        let detector = MonitorDetector::new(executor.clone());
        tracing::info!("MonitorDetector created");

        Ok(Self {
            executor,
            detector,
            cache_ttl: DEFAULT_CACHE_TTL_SECS,
            monitors: Arc::new(RwLock::new(HashMap::new())),
            initialized: Arc::new(RwLock::new(false)),
            cache_timestamps: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Create a new DDCManager with custom settings.
    pub async fn with_config(cache_ttl_secs: f64) -> DDCResult<Self> {
        let executor = Arc::new(tokio::sync::Mutex::new(CommandExecutor::new()));
        let detector = MonitorDetector::new(executor.clone());

        Ok(Self {
            executor,
            detector,
            cache_ttl: cache_ttl_secs,
            monitors: Arc::new(RwLock::new(HashMap::new())),
            initialized: Arc::new(RwLock::new(false)),
            cache_timestamps: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Initialize the DDCManager and detect available monitors.
    ///
    /// Returns true if initialization succeeded, false otherwise.
    pub async fn initialize(&self) -> DDCResult<bool> {
        tracing::info!("Initializing DDCManager...");

        // Check if ddcutil is available
        tracing::info!("Checking if ddcutil is available...");
        {
            let mut executor = self.executor.lock().await;
            tracing::info!("Acquired executor lock, calling check_ddcutil_available()...");
            let available = executor.check_ddcutil_available().await;
            drop(executor); // Release lock before potential error handling
            if !available {
                tracing::error!("ddcutil is not available on this system");
                return Err(DDCError::DDCNotAvailable);
            }
            tracing::info!("ddcutil is available");
        }
        tracing::info!("Released executor lock");

        // Detect monitors
        tracing::info!("Detecting monitors...");
        match self.detector.detect_monitors().await {
            Ok(detected_monitors) => {
                tracing::info!("Found {} monitor(s)", detected_monitors.len());
                let mut monitors = self.monitors.write().await;
                for monitor in detected_monitors {
                    tracing::info!("Adding monitor: {}", monitor.display_name());
                    monitors.insert(monitor.unique_id(), monitor);
                }

                let mut initialized = self.initialized.write().await;
                *initialized = true;

                tracing::info!("DDCManager initialized with {} monitor(s)", monitors.len());
                Ok(true)
            }
            Err(e) => {
                tracing::error!("Failed to initialize DDCManager: {}", e);
                Err(e)
            }
        }
    }

    /// Check if DDC/CI is available on the system.
    pub async fn is_available(&self) -> bool {
        let mut executor = self.executor.lock().await;
        executor.check_ddcutil_available().await
    }

    /// Check if the user has permission to access I2C devices.
    ///
    /// Returns true if permissions are OK, false otherwise.
    pub async fn check_permissions(&self, bus: Option<i32>) -> bool {
        let mut executor = self.executor.lock().await;

        // Try to query a common VCP code to check permissions
        let bus_to_check = bus.unwrap_or(1);
        match executor.get_vcp(bus_to_check, 0x10).await {
            Ok(result) => result.success || !result.stderr.contains("Permission denied"),
            Err(e) => !is_permission_error(&e),
        }
    }

    /// Get all detected monitors.
    pub async fn get_monitors(&self) -> Vec<Monitor> {
        let initialized = self.initialized.read().await;
        if !*initialized {
            drop(initialized);
            let _ = self.initialize().await;
        }

        let monitors = self.monitors.read().await;
        monitors.values().cloned().collect()
    }

    /// Get a monitor by its I2C bus number.
    pub async fn get_monitor_by_bus(&self, bus: i32) -> DDCResult<Monitor> {
        let monitors = self.monitors.read().await;
        for monitor in monitors.values() {
            if monitor.bus == bus {
                return Ok(monitor.clone());
            }
        }
        Err(DDCError::MonitorNotFound { bus })
    }

    /// Get a monitor by its serial number.
    pub async fn get_monitor_by_serial(&self, serial: &str) -> DDCResult<Monitor> {
        let monitors = self.monitors.read().await;
        for monitor in monitors.values() {
            if monitor.serial == serial {
                return Ok(monitor.clone());
            }
        }
        Err(DDCError::Other(format!(
            "Monitor with serial {} not found",
            serial
        )))
    }

    /// Get a monitor by its unique ID.
    pub async fn get_monitor_by_id(&self, unique_id: &str) -> DDCResult<Monitor> {
        let monitors = self.monitors.read().await;
        monitors
            .get(unique_id)
            .cloned()
            .ok_or_else(|| DDCError::Other(format!("Monitor with ID {} not found", unique_id)))
    }

    /// Get a VCP value from a monitor.
    pub async fn get_vcp(&self, monitor_id: &str, vcp_code: u8) -> DDCResult<u16> {
        let monitor = self.get_monitor_by_id(monitor_id).await?;

        // Check cache first
        let cache_timestamps = self.cache_timestamps.read().await;
        if let Some(timestamps) = cache_timestamps.get(monitor_id) {
            if let Some(&timestamp) = timestamps.get(&vcp_code) {
                let elapsed = (chrono::Utc::now() - timestamp).num_seconds() as f64;
                if elapsed < self.cache_ttl {
                    let monitors = self.monitors.read().await;
                    if let Some(monitor) = monitors.get(monitor_id) {
                        if let Some(cached) = monitor.get_cached_value(vcp_code) {
                            drop(cache_timestamps);
                            return Ok(cached);
                        }
                    }
                }
            }
        }
        drop(cache_timestamps);

        // Query the monitor
        let mut executor = self.executor.lock().await;
        let result = executor.get_vcp(monitor.bus, vcp_code).await?;

        if let Some(value) = result.value {
            // Update cache
            let mut monitors = self.monitors.write().await;
            if let Some(monitor) = monitors.get_mut(monitor_id) {
                monitor.set_cached_value(vcp_code, value);
            }

            let mut cache_timestamps = self.cache_timestamps.write().await;
            let timestamps = cache_timestamps
                .entry(monitor_id.to_string())
                .or_insert_with(HashMap::new);
            timestamps.insert(vcp_code, chrono::Utc::now());

            Ok(value)
        } else {
            Err(DDCError::ParseError(
                "Failed to parse VCP value".to_string(),
            ))
        }
    }

    /// Set a VCP value on a monitor.
    pub async fn set_vcp(&self, monitor_id: &str, vcp_code: u8, value: u16) -> DDCResult<()> {
        let monitor = self.get_monitor_by_id(monitor_id).await?;

        // Warn if VCP code is not in reported capabilities, but don't block —
        // capabilities parsing can be incomplete (e.g. monitors that don't
        // respond to capabilities queries). Try the command anyway.
        if !monitor.capabilities.supports_vcp(vcp_code) {
            tracing::warn!(
                "VCP code 0x{:02X} not reported in capabilities for {}, attempting anyway",
                vcp_code,
                monitor_id
            );
        }

        // Validate value
        if let Some(info) = get_vcp_info(vcp_code) {
            if !info.validate_value(value) {
                return Err(DDCError::InvalidVCPValue {
                    vcp_code,
                    value,
                    min: info.min_value,
                    max: info.max_value,
                });
            }
        }

        // Set the value
        let mut executor = self.executor.lock().await;
        executor.set_vcp(monitor.bus, vcp_code, value).await?;

        // Update cache
        let mut monitors = self.monitors.write().await;
        if let Some(monitor) = monitors.get_mut(monitor_id) {
            monitor.set_cached_value(vcp_code, value);
        }

        let mut cache_timestamps = self.cache_timestamps.write().await;
        let timestamps = cache_timestamps
            .entry(monitor_id.to_string())
            .or_insert_with(HashMap::new);
        timestamps.insert(vcp_code, chrono::Utc::now());

        Ok(())
    }

    /// Get the brightness of a monitor.
    /// For internal displays, reads from /sys/class/backlight/.
    /// For external monitors, uses DDC/CI VCP code 0x10.
    pub async fn get_brightness(&self, monitor_id: &str) -> DDCResult<u16> {
        let monitor = self.get_monitor_by_id(monitor_id).await?;

        if monitor.monitor_type == crate::ddc::monitor::MonitorType::Internal {
            return self._get_backlight_brightness(&monitor).await;
        }

        self.get_vcp(monitor_id, 0x10).await
    }

    /// Set the brightness of a monitor.
    /// For internal displays, writes to /sys/class/backlight/.
    /// For external monitors, uses DDC/CI VCP code 0x10.
    /// Clamps to minimum_brightness to prevent backlight shutoff.
    pub async fn set_brightness(&self, monitor_id: &str, value: u16) -> DDCResult<()> {
        let monitor = self.get_monitor_by_id(monitor_id).await?;

        if monitor.monitor_type == crate::ddc::monitor::MonitorType::Internal {
            return self._set_backlight_brightness(&monitor, value).await;
        }

        self.set_vcp(monitor_id, 0x10, value).await
    }

    /// Set brightness of all monitors.
    /// Clamps to min to prevent backlight shutoff.
    pub async fn set_all_brightness(&self, value: u16, min_brightness: u16) -> DDCResult<()> {
        let monitors = self.get_monitors().await;
        let clamped = value.max(min_brightness);
        for m in &monitors {
            if let Err(e) = self.set_brightness(&m.unique_id(), clamped).await {
                tracing::warn!("set_all_brightness failed on {}: {}", m.display_name(), e);
            }
        }
        Ok(())
    }

    /// Read brightness from kernel backlight sysfs.
    async fn _get_backlight_brightness(&self, monitor: &Monitor) -> DDCResult<u16> {
        let path = monitor
            .backlight_path
            .as_ref()
            .ok_or_else(|| DDCError::Other("No backlight path".to_string()))?;

        let max_brightness: u16 = tokio::task::spawn_blocking({
            let path = path.clone();
            move || {
                std::fs::read_to_string(format!("{}/max_brightness", path))
                    .ok()
                    .and_then(|s| s.trim().parse().ok())
                    .unwrap_or(100)
            }
        })
        .await
        .map_err(|e| DDCError::Other(e.to_string()))?;

        let current: u16 = tokio::task::spawn_blocking({
            let path = path.clone();
            move || {
                std::fs::read_to_string(format!("{}/brightness", path))
                    .ok()
                    .and_then(|s| s.trim().parse().ok())
                    .unwrap_or(0)
            }
        })
        .await
        .map_err(|e| DDCError::Other(e.to_string()))?;

        // Convert to 0-100 percentage
        let pct = if max_brightness > 0 {
            (current as u32 * 100 / max_brightness as u32) as u16
        } else {
            0
        };

        tracing::debug!(
            "Backlight {} read: {}/{} = {}%",
            path,
            current,
            max_brightness,
            pct
        );
        Ok(pct)
    }

    /// Write brightness to internal display via systemd-logind D-Bus.
    ///
    /// Uses org.freedesktop.login1.Session.SetBrightness which works
    /// without special group membership — it checks the active session
    /// instead of file permissions. Falls back to direct sysfs write.
    async fn _set_backlight_brightness(&self, monitor: &Monitor, value: u16) -> DDCResult<()> {
        let path = monitor
            .backlight_path
            .as_ref()
            .ok_or_else(|| DDCError::Other("No backlight path".to_string()))?;

        let value = value.clamp(0, 100);

        // Extract backlight name from sysfs path (e.g. "intel_backlight" from
        // "/sys/class/backlight/intel_backlight")
        let backlight_name = path
            .rsplit('/')
            .next()
            .ok_or_else(|| DDCError::Other("Invalid backlight path".to_string()))?
            .to_string();

        // Read max_brightness (world-readable) and compute raw value
        let (raw_value, max) = tokio::task::spawn_blocking({
            let path = path.clone();
            move || -> (u32, u32) {
                let max_str = std::fs::read_to_string(format!("{}/max_brightness", &path))
                    .unwrap_or_else(|_| "100".to_string());
                let max: u32 = max_str.trim().parse().unwrap_or(100);
                let raw = (value as u32 * max / 100) as u32;
                (raw, max)
            }
        })
        .await
        .map_err(|e| DDCError::Other(e.to_string()))?;

        // Try systemd-logind D-Bus SetBrightness first.
        // This works without special permissions for any user with an active session.
        let gdbus_result = tokio::task::spawn_blocking({
            let backlight_name = backlight_name.clone();
            move || -> std::io::Result<()> {
                let output = std::process::Command::new("gdbus")
                    .args([
                        "call",
                        "--system",
                        "--dest",
                        "org.freedesktop.login1",
                        "--object-path",
                        "/org/freedesktop/login1/session/auto",
                        "--method",
                        "org.freedesktop.login1.Session.SetBrightness",
                        "backlight",
                        &backlight_name,
                        &raw_value.to_string(),
                    ])
                    .output()?;

                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    return Err(std::io::Error::other(format!(
                        "gdbus SetBrightness failed: {}",
                        stderr.trim()
                    )));
                }
                Ok(())
            }
        })
        .await
        .map_err(|e| DDCError::Other(e.to_string()))?;

        match gdbus_result {
            Ok(()) => {
                tracing::info!(
                    "Backlight {} set to {}% (raw {}/{}) via systemd-logind",
                    backlight_name,
                    value,
                    raw_value,
                    max
                );
                return Ok(());
            }
            Err(e) => {
                tracing::warn!(
                    "systemd-logind SetBrightness failed ({}), falling back to direct sysfs write",
                    e
                );
            }
        }

        // Fallback: direct sysfs write (needs video group or root)
        let result: std::io::Result<()> = tokio::task::spawn_blocking({
            let path = path.clone();
            move || std::fs::write(format!("{}/brightness", path), raw_value.to_string())
        })
        .await
        .map_err(|e| DDCError::Other(e.to_string()))?;

        result.map_err(|e| {
            if e.kind() == std::io::ErrorKind::PermissionDenied {
                DDCError::PermissionDenied(format!(
                    "Cannot write to {}/brightness. \
                     Install brightnessctl or ensure systemd-logind is running.",
                    path
                ))
            } else {
                DDCError::Other(format!("Backlight write failed: {}", e))
            }
        })?;

        tracing::info!(
            "Backlight {} set to {}% (raw {}/{}) via sysfs",
            backlight_name,
            value,
            raw_value,
            max
        );
        Ok(())
    }

    /// Get the contrast of a monitor.
    pub async fn get_contrast(&self, monitor_id: &str) -> DDCResult<u16> {
        self.get_vcp(monitor_id, 0x12).await
    }

    /// Set the contrast of a monitor.
    pub async fn set_contrast(&self, monitor_id: &str, value: u16) -> DDCResult<()> {
        self.set_vcp(monitor_id, 0x12, value).await
    }

    /// Get the volume of a monitor.
    pub async fn get_volume(&self, monitor_id: &str) -> DDCResult<u16> {
        self.get_vcp(monitor_id, 0x62).await
    }

    /// Set the volume of a monitor.
    pub async fn set_volume(&self, monitor_id: &str, value: u16) -> DDCResult<()> {
        self.set_vcp(monitor_id, 0x62, value).await
    }

    /// Get the input source of a monitor.
    pub async fn get_input_source(&self, monitor_id: &str) -> DDCResult<u16> {
        self.get_vcp(monitor_id, 0x60).await
    }

    /// Set the input source of a monitor.
    pub async fn set_input_source(&self, monitor_id: &str, value: u16) -> DDCResult<()> {
        self.set_vcp(monitor_id, 0x60, value).await
    }

    /// Get the color temperature of a monitor.
    pub async fn get_color_temperature(&self, monitor_id: &str) -> DDCResult<u16> {
        self.get_vcp(monitor_id, 0x14).await
    }

    /// Set the color temperature of a monitor.
    pub async fn set_color_temperature(&self, monitor_id: &str, value: u16) -> DDCResult<()> {
        self.set_vcp(monitor_id, 0x14, value).await
    }

    /// Get information about all supported VCP codes.
    pub fn get_vcp_codes_info(&self) -> Vec<crate::ddc::vcp_codes::VCPCodeInfo> {
        get_common_vcp_codes()
    }

    /// Clear the cache for a specific monitor.
    pub async fn clear_cache(&self, monitor_id: &str) {
        let mut monitors = self.monitors.write().await;
        if let Some(monitor) = monitors.get_mut(monitor_id) {
            monitor.clear_cache();
        }

        let mut cache_timestamps = self.cache_timestamps.write().await;
        cache_timestamps.remove(monitor_id);
    }

    /// Clear all caches.
    pub async fn clear_all_caches(&self) {
        let mut monitors = self.monitors.write().await;
        for monitor in monitors.values_mut() {
            monitor.clear_cache();
        }

        let mut cache_timestamps = self.cache_timestamps.write().await;
        cache_timestamps.clear();
    }

    /// Refresh monitor detection.
    pub async fn refresh_monitors(&self) -> DDCResult<Vec<Monitor>> {
        let detected_monitors = self.detector.detect_monitors().await?;

        let mut monitors = self.monitors.write().await;
        monitors.clear();

        for monitor in detected_monitors {
            monitors.insert(monitor.unique_id(), monitor);
        }

        tracing::info!("Refreshed monitors: {} detected", monitors.len());

        Ok(monitors.values().cloned().collect())
    }
}

impl Default for DDCManager {
    fn default() -> Self {
        // Note: This is a blocking default implementation
        // In async context, use DDCManager::new() instead
        Self {
            executor: Arc::new(tokio::sync::Mutex::new(CommandExecutor::default())),
            detector: MonitorDetector::new(Arc::new(tokio::sync::Mutex::new(
                CommandExecutor::default(),
            ))),
            cache_ttl: DEFAULT_CACHE_TTL_SECS,
            monitors: Arc::new(RwLock::new(HashMap::new())),
            initialized: Arc::new(RwLock::new(false)),
            cache_timestamps: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}
