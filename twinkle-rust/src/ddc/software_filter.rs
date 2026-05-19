//! Software brightness filter using XRandR gamma adjustment.
//!
//! When hardware brightness (DDC/CI VCP 0x10) is at its minimum but still too bright
//! (e.g. at night), this module applies a software gamma filter via `xrandr` to
//! further dim the display below the hardware level.
//!
//! The gamma value ranges from 0.0 (black) to 1.0 (normal). A value of 0.5 means
//! the display appears at roughly half the hardware brightness.

use std::process::Command;

/// Software brightness filter state.
pub struct SoftwareFilter {
    /// Current filter level: 0.0 = no filter (100% passthrough), 1.0 = maximum dimming.
    /// Maps to gamma: gamma = 1.0 - (filter_level * 0.9) so we never go fully black.
    filter_level: f64,
    /// XRandR output names for all connected displays
    output_names: Vec<String>,
}

impl SoftwareFilter {
    /// Create a new software filter.
    pub fn new() -> Self {
        Self {
            filter_level: 0.0,
            output_names: Vec::new(),
        }
    }

    /// Detect XRandR outputs and store all connected ones.
    pub fn detect_outputs(&mut self) -> Vec<String> {
        let outputs = Self::get_xrandr_outputs();
        if self.output_names.is_empty() {
            self.output_names = outputs.clone();
        }
        outputs
    }

    /// Get list of connected XRandR output names.
    pub fn get_xrandr_outputs() -> Vec<String> {
        let output = match Command::new("xrandr").arg("--query").output() {
            Ok(o) => o,
            Err(e) => {
                tracing::warn!("Failed to run xrandr: {}", e);
                return Vec::new();
            }
        };

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut outputs = Vec::new();

        for line in stdout.lines() {
            // xrandr output lines: "DP-1 connected primary 2560x1440+0+0 (normal left..."
            if line.contains(" connected") {
                if let Some(name) = line.split_whitespace().next() {
                    outputs.push(name.to_string());
                }
            }
        }

        tracing::info!("Detected XRandR outputs: {:?}", outputs);
        outputs
    }

    /// Get the current filter level (0.0 to 1.0).
    pub fn filter_level(&self) -> f64 {
        self.filter_level
    }

    /// Set the filter level and apply immediately.
    /// 0.0 = no filter (hardware brightness unchanged)
    /// 1.0 = maximum dimming (gamma ≈ 0.1, very dark but not black)
    ///
    /// Returns Ok(()) if the gamma was applied successfully.
    pub fn set_filter_level(&mut self, level: f64) -> Result<(), String> {
        let level = level.clamp(0.0, 1.0);

        if level == 0.0 {
            // Reset gamma to normal
            return self.reset_gamma();
        }

        // Map filter level to gamma value.
        // At level=0.0: gamma=1.0 (normal)
        // At level=1.0: gamma=0.1 (very dim, but not black)
        // This prevents the screen from going completely black.
        let gamma = 1.0 - (level * 0.9);

        self.apply_gamma(gamma)?;
        self.filter_level = level;
        Ok(())
    }

    /// Apply a gamma value to ALL connected outputs via xrandr.
    fn apply_gamma(&self, gamma: f64) -> Result<(), String> {
        if self.output_names.is_empty() {
            return Err("No XRandR outputs configured".to_string());
        }

        let gamma_str = format!("{:.4}", gamma);
        let gamma_arg = format!("{}:{}:{}", gamma_str, gamma_str, gamma_str);

        for output_name in &self.output_names {
            tracing::info!("Applying gamma {} to output {}", gamma_str, output_name);

            let output = Command::new("xrandr")
                .args(["--output", output_name, "--gamma", &gamma_arg])
                .output()
                .map_err(|e| format!("Failed to run xrandr: {}", e))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                tracing::warn!(
                    "xrandr gamma failed on {}: {}",
                    output_name,
                    stderr.trim()
                );
            }
        }

        Ok(())
    }

    /// Reset gamma to normal (1.0:1.0:1.0) on ALL outputs.
    pub fn reset_gamma(&self) -> Result<(), String> {
        if self.output_names.is_empty() {
            return Ok(());
        }

        for output_name in &self.output_names {
            tracing::info!("Resetting gamma on output {}", output_name);

            let output = Command::new("xrandr")
                .args(["--output", output_name, "--gamma", "1.0:1.0:1.0"])
                .output()
                .map_err(|e| format!("Failed to run xrandr: {}", e))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                tracing::warn!(
                    "xrandr gamma reset failed on {}: {}",
                    output_name,
                    stderr.trim()
                );
            }
        }

        Ok(())
    }

    /// Reset gamma on ALL connected outputs.
    pub fn reset_all_gamma() -> Result<(), String> {
        let outputs = Self::get_xrandr_outputs();
        for output_name in &outputs {
            tracing::info!("Resetting gamma on output {}", output_name);
            let result = Command::new("xrandr")
                .args(["--output", output_name, "--gamma", "1.0:1.0:1.0"])
                .output()
                .map_err(|e| format!("Failed to run xrandr: {}", e))?;

            if !result.status.success() {
                let stderr = String::from_utf8_lossy(&result.stderr);
                tracing::warn!(
                    "Failed to reset gamma on {}: {}",
                    output_name,
                    stderr.trim()
                );
            }
        }
        Ok(())
    }

    /// Convert a filter level (0.0-1.0) to a human-readable percentage.
    /// 0% = no filter, 100% = maximum dimming.
    pub fn level_to_percent(level: f64) -> u8 {
        (level * 100.0).round() as u8
    }

    /// Convert a percentage (0-100) to a filter level (0.0-1.0).
    pub fn percent_to_level(percent: u8) -> f64 {
        percent as f64 / 100.0
    }
}

impl Default for SoftwareFilter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_level_to_percent() {
        assert_eq!(SoftwareFilter::level_to_percent(0.0), 0);
        assert_eq!(SoftwareFilter::level_to_percent(0.5), 50);
        assert_eq!(SoftwareFilter::level_to_percent(1.0), 100);
    }

    #[test]
    fn test_percent_to_level() {
        assert!((SoftwareFilter::percent_to_level(0) - 0.0).abs() < 0.01);
        assert!((SoftwareFilter::percent_to_level(50) - 0.5).abs() < 0.01);
        assert!((SoftwareFilter::percent_to_level(100) - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_filter_level_clamp() {
        let mut filter = SoftwareFilter::new();
        // Should not panic with out-of-range values
        assert!(filter.set_filter_level(-1.0).is_ok());
        assert!((filter.filter_level() - 0.0).abs() < 0.01);

        assert!(filter.set_filter_level(2.0).is_ok());
        // filter_level is clamped but won't apply xrandr in test env
    }
}
