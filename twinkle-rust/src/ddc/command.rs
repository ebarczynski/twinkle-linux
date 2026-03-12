//! Command executor for ddcutil subprocess management.

use crate::ddc::error::{CommandExecutionError, DDCError, DDCResult};
use regex::Regex;
use std::process::Command;
use std::time::Duration;
use tokio::time::timeout as tokio_timeout;

/// Default timeout for command execution in seconds.
pub const DEFAULT_TIMEOUT_SECS: f64 = 5.0;

/// Default maximum number of retry attempts.
pub const DEFAULT_MAX_RETRIES: u32 = 3;

/// Default initial delay between retries in seconds.
pub const DEFAULT_RETRY_DELAY_SECS: f64 = 0.1;

/// Default multiplier for exponential backoff.
pub const DEFAULT_RETRY_BACKOFF: f64 = 2.0;

/// Result of a ddcutil command execution.
#[derive(Debug, Clone)]
pub struct CommandResult {
    /// Whether the command executed successfully
    pub success: bool,
    /// The exit code of the command
    pub return_code: i32,
    /// Standard output from the command
    pub stdout: String,
    /// Standard error from the command
    pub stderr: String,
    /// Extracted numeric value (for getvcp commands)
    pub value: Option<u16>,
    /// The command that was executed
    pub command: String,
}

impl CommandResult {
    /// Get a formatted error message.
    pub fn error_message(&self) -> String {
        if self.success {
            return String::new();
        }
        let mut msg = format!("Command failed with exit code {}", self.return_code);
        if !self.stderr.is_empty() {
            msg.push_str(&format!(": {}", self.stderr.trim()));
        }
        msg
    }
}

/// Executor for ddcutil commands with retry logic and error handling.
pub struct CommandExecutor {
    /// Default timeout for command execution in seconds
    timeout_secs: f64,
    /// Maximum number of retry attempts for transient failures
    max_retries: u32,
    /// Initial delay between retries in seconds
    retry_delay_secs: f64,
    /// Multiplier for exponential backoff
    retry_backoff: f64,
    /// Path to ddcutil executable
    ddcutil_path: Option<String>,
}

impl CommandExecutor {
    /// Create a new CommandExecutor with default settings.
    pub fn new() -> Self {
        Self {
            timeout_secs: DEFAULT_TIMEOUT_SECS,
            max_retries: DEFAULT_MAX_RETRIES,
            retry_delay_secs: DEFAULT_RETRY_DELAY_SECS,
            retry_backoff: DEFAULT_RETRY_BACKOFF,
            ddcutil_path: None,
        }
    }

    /// Create a new CommandExecutor with custom settings.
    pub fn with_config(
        timeout_secs: f64,
        max_retries: u32,
        retry_delay_secs: f64,
        retry_backoff: f64,
    ) -> Self {
        Self {
            timeout_secs,
            max_retries,
            retry_delay_secs,
            retry_backoff,
            ddcutil_path: None,
        }
    }

    /// Check if ddcutil is available on the system.
    /// This is an async function that uses timeout to prevent blocking.
    pub async fn check_ddcutil_available(&mut self) -> bool {
        tracing::info!("check_ddcutil_available() - Starting ddcutil --version command");
        
        let timeout_duration = Duration::from_secs_f64(self.timeout_secs);
        
        let result = tokio_timeout(timeout_duration, async {
            tokio::task::spawn_blocking(|| {
                Command::new("ddcutil")
                    .arg("--version")
                    .output()
            })
            .await
            .map_err(|e| DDCError::Other(format!("Task join error: {}", e)))?
            .map_err(|e| DDCError::Io(e))
        })
        .await;

        tracing::info!("check_ddcutil_available() - Command completed");
        match result {
            Ok(Ok(output)) => {
                if output.status.success() {
                    self.ddcutil_path = Some("ddcutil".to_string());
                    tracing::info!("ddcutil found: {}", String::from_utf8_lossy(&output.stdout).trim());
                    true
                } else {
                    tracing::warn!("ddcutil --version failed with exit code {:?}", output.status.code());
                    false
                }
            }
            Ok(Err(e)) => {
                tracing::warn!("ddcutil command failed: {}", e);
                false
            }
            Err(_) => {
                tracing::warn!("ddcutil --version timed out after {}s", self.timeout_secs);
                false
            }
        }
    }

    /// Get the path to the ddcutil executable.
    ///
    /// Returns an error if ddcutil is not available.
    async fn get_ddcutil_path(&mut self) -> DDCResult<String> {
        if self.ddcutil_path.is_none() && !self.check_ddcutil_available().await {
            return Err(DDCError::DDCNotAvailable);
        }
        Ok(self.ddcutil_path.clone().unwrap_or_else(|| "ddcutil".to_string()))
    }

    /// Execute a ddcutil command with timeout and retry logic.
    pub async fn execute(&mut self, args: &[&str]) -> DDCResult<CommandResult> {
        let ddcutil_path = self.get_ddcutil_path().await?;
        let command_str = format!("{} {}", ddcutil_path, args.join(" "));

        tracing::info!("execute() - Starting command: {} (timeout: {}s)", command_str, self.timeout_secs);
        let mut last_error = None;
        let mut delay = self.retry_delay_secs;

        for attempt in 0..=self.max_retries {
            if attempt > 0 {
                tracing::debug!("Retry attempt {} for command: {}", attempt, command_str);
                tokio::time::sleep(Duration::from_secs_f64(delay)).await;
                delay *= self.retry_backoff;
            }

            tracing::info!("execute() - Attempt {} for command: {}", attempt + 1, command_str);
            match self._execute_single(&ddcutil_path, args).await {
                Ok(result) => {
                    tracing::info!("execute() - Command completed with success={}", result.success);
                    if result.success {
                        return Ok(result);
                    }
                    // Check if this is a recoverable error
                    if self._is_recoverable(&result) {
                        last_error = Some(DDCError::CommandExecution(CommandExecutionError {
                            command: command_str.clone(),
                            exit_code: result.return_code,
                            stderr: result.stderr.clone(),
                        }));
                        continue;
                    }
                    return Err(DDCError::CommandExecution(CommandExecutionError {
                        command: command_str,
                        exit_code: result.return_code,
                        stderr: result.stderr,
                    }));
                }
                Err(e) => {
                    tracing::warn!("execute() - Command error: {}", e);
                    if is_timeout_error(&e) {
                        last_error = Some(e);
                        continue;
                    }
                    return Err(e);
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            DDCError::CommandExecution(CommandExecutionError {
                command: command_str,
                exit_code: -1,
                stderr: "Max retries exceeded".to_string(),
            })
        }))
    }

    /// Execute a single command attempt with timeout.
    async fn _execute_single(&self, ddcutil_path: &str, args: &[&str]) -> DDCResult<CommandResult> {
        let timeout_duration = Duration::from_secs_f64(self.timeout_secs);

        // Clone to owned types for moving to spawn_blocking
        let ddcutil_path = ddcutil_path.to_string();
        let args: Vec<String> = args.iter().map(|s| s.to_string()).collect();
        let command_str = format!("{} {}", ddcutil_path, args.join(" "));

        let output = tokio_timeout(timeout_duration, async {
            tokio::task::spawn_blocking(move || {
                Command::new(&ddcutil_path)
                    .args(&args)
                    .output()
            })
            .await
            .map_err(|e| DDCError::Other(format!("Task join error: {}", e)))?
            .map_err(|e| DDCError::Io(e))
        })
        .await
        .map_err(|_| DDCError::Timeout(self.timeout_secs))??;

        let success = output.status.success();
        let return_code = output.status.code().unwrap_or(-1);
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        Ok(CommandResult {
            success,
            return_code,
            stdout,
            stderr,
            value: None,
            command: command_str,
        })
    }

    /// Check if a command result indicates a recoverable error.
    fn _is_recoverable(&self, result: &CommandResult) -> bool {
        // Check for timeout-related errors
        if result.stderr.contains("timeout") || result.stderr.contains("timed out") {
            return true;
        }
        // Check for transient I2C errors
        if result.stderr.contains("I2C bus") || result.stderr.contains("DDC/CI") {
            return true;
        }
        false
    }

    /// Get a VCP value from a monitor.
    pub async fn get_vcp(&mut self, bus: i32, vcp_code: u8) -> DDCResult<CommandResult> {
        let args = &["getvcp", &format!("--bus={}", bus), &format!("0x{:02X}", vcp_code)];
        let mut result = self.execute(args).await?;

        // Parse the value from output
        if let Some(value) = self._parse_getvcp_output(&result.stdout) {
            result.value = Some(value);
        }

        Ok(result)
    }

    /// Set a VCP value on a monitor.
    pub async fn set_vcp(&mut self, bus: i32, vcp_code: u8, value: u16) -> DDCResult<CommandResult> {
        let args = &[
            "setvcp",
            &format!("--bus={}", bus),
            &format!("0x{:02X}", vcp_code),
            &value.to_string(),
        ];
        self.execute(args).await
    }

    /// Detect monitors on the system.
    pub async fn detect_monitors(&mut self) -> DDCResult<CommandResult> {
        tracing::info!("detect_monitors() - Starting ddcutil detect --brief command");
        let args = &["detect", "--brief"];
        let result = self.execute(args).await;
        tracing::info!("detect_monitors() - Command completed");
        result
    }

    /// Get EDID data for a monitor.
    pub async fn get_edid(&mut self, bus: i32) -> DDCResult<CommandResult> {
        let args = &["getedid", "--bus", &format!("{}", bus)];
        self.execute(args).await
    }

    /// Get capabilities for a monitor.
    pub async fn get_capabilities(&mut self, bus: i32) -> DDCResult<CommandResult> {
        let args = &["capabilities", "--bus", &format!("{}", bus)];
        self.execute(args).await
    }

    /// Parse the value from getvcp output.
    fn _parse_getvcp_output(&self, stdout: &str) -> Option<u16> {
        // Output format: "VCP 10 C 50 100" or "VCP 10 (Brightness): current value = 50, maximum = 100"
        let re = Regex::new(r"current value\s*=\s*(\d+)").ok()?;
        re.captures(stdout)?.get(1)?.as_str().parse().ok()
    }
}

impl Default for CommandExecutor {
    fn default() -> Self {
        Self::new()
    }
}

/// Check if an error is a timeout error.
fn is_timeout_error(err: &DDCError) -> bool {
    matches!(err, DDCError::Timeout(_))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_result_error_message() {
        let result = CommandResult {
            success: false,
            return_code: 1,
            stdout: String::new(),
            stderr: "Permission denied".to_string(),
            value: None,
            command: "ddcutil getvcp".to_string(),
        };
        assert_eq!(
            result.error_message(),
            "Command failed with exit code 1: Permission denied"
        );
    }

    #[test]
    fn test_parse_getvcp_output() {
        let executor = CommandExecutor::new();
        let output = "VCP 10 (Brightness): current value = 50, maximum = 100";
        assert_eq!(executor._parse_getvcp_output(output), Some(50));
    }
}
