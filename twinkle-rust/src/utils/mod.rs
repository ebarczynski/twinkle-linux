//! Utility functions for Twinkle Linux.

/// Get the application version.
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Get the application name.
pub fn app_name() -> &'static str {
    env!("CARGO_PKG_NAME")
}

/// Get the application authors.
pub fn authors() -> &'static str {
    env!("CARGO_PKG_AUTHORS")
}

/// Get the application description.
pub fn description() -> &'static str {
    env!("CARGO_PKG_DESCRIPTION")
}

/// Format a brightness value as a percentage.
pub fn format_brightness(value: u16) -> String {
    format!("{}%", value)
}

/// Clamp a value between min and max.
pub fn clamp<T: Ord>(value: T, min: T, max: T) -> T {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

/// Snap a slider value to the nearest configured preset value.
///
/// Ties resolve to the smaller preset. An empty preset list returns the
/// value unchanged (no snapping).
pub fn snap_to_preset(value: u16, presets: &[u16]) -> u16 {
    presets
        .iter()
        .copied()
        .min_by_key(|&p| (p.abs_diff(value), p))
        .unwrap_or(value)
}

/// Normalize configured preset values for slider snapping/tick marks:
/// sort, dedupe, drop duplicates, clamp to 0..=100 and drop values below
/// the minimum brightness (they would be overridden by the clamp anyway,
/// so their tick marks would be misleading).
pub fn normalized_presets(values: &[u16], min_brightness: u16) -> Vec<u16> {
    let mut presets: Vec<u16> = values
        .iter()
        .copied()
        .filter(|&v| v <= 100 && v >= min_brightness)
        .collect();
    presets.sort_unstable();
    presets.dedup();
    presets
}

/// Pair preset values with their display labels for the tray menu list:
/// sorted by value, deduplicated (first label wins), values clamped to
/// 0..=100 and above `min_brightness`. Missing/short `labels` fall back
/// to an empty label.
pub fn preset_menu_entries(
    values: &[u16],
    labels: &[String],
    min_brightness: u16,
) -> Vec<(u16, String)> {
    let mut seen = std::collections::HashSet::new();
    let mut entries: Vec<(u16, String)> = values
        .iter()
        .enumerate()
        .filter(|&(_, &v)| v <= 100 && v >= min_brightness)
        .map(|(i, &v)| (v, labels.get(i).cloned().unwrap_or_default()))
        .filter(|(v, _)| seen.insert(*v))
        .collect();
    entries.sort_by_key(|(v, _)| *v);
    entries
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!version().is_empty());
    }

    #[test]
    fn test_app_name() {
        assert_eq!(app_name(), "twinkle-linux");
    }

    #[test]
    fn test_format_brightness() {
        assert_eq!(format_brightness(50), "50%");
    }

    #[test]
    fn test_clamp() {
        assert_eq!(clamp(50, 0, 100), 50);
        assert_eq!(clamp(-10, 0, 100), 0);
        assert_eq!(clamp(150, 0, 100), 100);
    }

    #[test]
    fn test_snap_to_preset() {
        let presets = [10, 20, 40, 60, 80, 100];
        assert_eq!(snap_to_preset(0, &presets), 10);
        assert_eq!(snap_to_preset(12, &presets), 10);
        assert_eq!(snap_to_preset(15, &presets), 10); // exact tie → smaller preset
        assert_eq!(snap_to_preset(16, &presets), 20);
        assert_eq!(snap_to_preset(47, &presets), 40);
        assert_eq!(snap_to_preset(55, &presets), 60);
        assert_eq!(snap_to_preset(95, &presets), 100);
        assert_eq!(snap_to_preset(100, &presets), 100);
        // No presets → value passes through unchanged
        assert_eq!(snap_to_preset(47, &[]), 47);
    }

    #[test]
    fn test_normalized_presets() {
        // Unsorted, duplicated, out-of-range values are cleaned up
        assert_eq!(
            normalized_presets(&[60, 10, 100, 20, 40, 10, 150, 0], 1),
            vec![10, 20, 40, 60, 100]
        );
        // Values below min_brightness are dropped
        assert_eq!(normalized_presets(&[0, 10, 40], 10), vec![10, 40]);
        // Empty config → no snap points
        assert!(normalized_presets(&[], 1).is_empty());
    }

    #[test]
    fn test_preset_menu_entries() {
        let labels: Vec<String> = vec!["Night".into(), "Dusk".into(), "Cloudy".into()];
        // Labels pair by ORIGINAL config index (values[0]↔labels[0], ...),
        // then the result is sorted by value.
        assert_eq!(
            preset_menu_entries(&[40, 10, 20], &labels, 1),
            vec![
                (10, "Dusk".to_string()),
                (20, "Cloudy".to_string()),
                (40, "Night".to_string()),
            ]
        );
        // Duplicate value keeps the first label; below-min and >100 dropped
        assert_eq!(
            preset_menu_entries(&[20, 10, 20, 150, 0], &labels, 5),
            vec![(10, "Dusk".to_string()), (20, "Night".to_string())]
        );
        // Missing labels → empty label; values without any labels still listed
        assert_eq!(
            preset_menu_entries(&[30, 60], &[], 1),
            vec![(30, String::new()), (60, String::new())]
        );
        // Empty config → nothing
        assert!(preset_menu_entries(&[], &labels, 1).is_empty());
    }
}
