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
}
