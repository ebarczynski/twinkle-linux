//! Utility functions for Twinkle Linux.

use std::sync::OnceLock;
use tokio::runtime::Handle;

/// Global tokio runtime handle, set once at startup.
/// Required because GTK signal handlers call glib::spawn_future_local
/// which runs on the GLib main context — outside the tokio runtime.
/// Every async closure spawned via glib must enter this handle first.
static TOKIO_HANDLE: OnceLock<Handle> = OnceLock::new();

/// Set the global tokio runtime handle. Called once from main().
pub fn set_tokio_handle(handle: Handle) {
    TOKIO_HANDLE.set(handle).expect("tokio handle already set");
}

/// Get the global tokio runtime handle.
pub fn tokio_handle() -> &'static Handle {
    TOKIO_HANDLE.get().expect("tokio handle not set — call set_tokio_handle() first")
}

/// Spawn an async future on the GLib main context with the tokio runtime entered.
/// This is the correct way to bridge GTK signal handlers to tokio async code.
///
/// Usage: `spawn_local(async { ... })` instead of `glib::spawn_future_local(async { ... })`
pub fn spawn_local<F>(future: F)
where
    F: std::future::Future<Output = ()> + 'static,
{
    let handle = tokio_handle().clone();
    gtk4::glib::spawn_future_local(async move {
        let _guard = handle.enter();
        future.await;
    });
}

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
