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
#[allow(unused_imports)]
pub use ddc_manager::DDCManager;
