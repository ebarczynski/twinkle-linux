//! Error types for DDC/CI operations.

use thiserror::Error;

/// Result type alias for DDC operations.
pub type DDCResult<T> = Result<T, DDCError>;

/// Errors that can occur during DDC/CI operations.
#[derive(Error, Debug)]
pub enum DDCError {
    /// ddcutil is not available on the system.
    #[error("ddcutil is not available on this system")]
    DDCNotAvailable,

    /// Command execution failed.
    #[error("Command execution failed: {0}")]
    CommandExecution(#[from] CommandExecutionError),

    /// Monitor not found.
    #[error("Monitor not found: bus {bus}")]
    MonitorNotFound { bus: i32 },

    /// Permission denied when accessing I2C device.
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// Timeout occurred during operation.
    #[error("Operation timed out after {0}s")]
    Timeout(f64),

    /// VCP code not supported by the monitor.
    #[error("VCP code 0x{vcp_code:02X} not supported by monitor")]
    VCPNotSupported { vcp_code: u8 },

    /// Invalid VCP value.
    #[error("Invalid VCP value {value} for code 0x{vcp_code:02X}: expected {min}-{max}")]
    InvalidVCPValue {
        vcp_code: u8,
        value: u16,
        min: u16,
        max: u16,
    },

    /// Monitor disconnected.
    #[error("Monitor disconnected: {monitor_id}")]
    MonitorDisconnected { monitor_id: String },

    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Parse error.
    #[error("Failed to parse output: {0}")]
    ParseError(String),

    /// Other error.
    #[error("{0}")]
    Other(String),
}

/// Error from command execution.
#[derive(Error, Debug)]
pub struct CommandExecutionError {
    /// The command that was executed.
    pub command: String,
    /// The exit code.
    pub exit_code: i32,
    /// Standard error output.
    pub stderr: String,
}

impl std::fmt::Display for CommandExecutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Command '{}' failed with exit code {}: {}",
            self.command, self.exit_code, self.stderr
        )
    }
}

/// Check if an error is a permission error.
pub fn is_permission_error(err: &DDCError) -> bool {
    matches!(err, DDCError::PermissionDenied(_))
}

/// Check if an error is a timeout error.
pub fn is_timeout_error(err: &DDCError) -> bool {
    matches!(err, DDCError::Timeout(_))
}

/// Check if an error is recoverable (can be retried).
pub fn is_recoverable(err: &DDCError) -> bool {
    is_timeout_error(err) || matches!(err, DDCError::CommandExecution(_) | DDCError::Io(_))
}
