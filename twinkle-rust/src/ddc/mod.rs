//! DDC/CI (Display Data Channel Command Interface) module.
//!
//! This module provides functionality for controlling external monitors
//! via the DDC/CI protocol using the ddcutil command-line tool.

pub mod command;
pub mod ddc_manager;
pub mod error;
pub mod monitor;
pub mod vcp_codes;

// Re-exports for convenience
pub use command::{CommandExecutor, CommandResult};
pub use ddc_manager::DDCManager;
pub use error::{DDCError, DDCResult, is_permission_error, is_recoverable, is_timeout_error};
pub use monitor::{Monitor, MonitorCapabilities, MonitorDetector};
pub use vcp_codes::{VCPCodeInfo, ValueType, get_common_vcp_codes, get_vcp_info};
