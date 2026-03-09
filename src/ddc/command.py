"""
Command executor for ddcutil subprocess management.

This module provides the CommandExecutor class that handles execution of
ddcutil commands with proper timeout handling, retry logic, and output parsing.
"""

from __future__ import annotations

import logging
import re
import shlex
import subprocess
import time
from dataclasses import dataclass
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from collections.abc import Callable

from src.ddc.exceptions import (
    CommandExecutionError,
    DDCNotAvailableError,
    PermissionError,
    TimeoutError,
)

logger = logging.getLogger(__name__)


@dataclass
class CommandResult:
    """
    Result of a ddcutil command execution.

    Attributes:
        success: Whether the command executed successfully
        return_code: The exit code of the command
        stdout: Standard output from the command
        stderr: Standard error from the command
        value: Extracted numeric value (for getvcp commands)
        command: The command that was executed
    """

    success: bool
    return_code: int
    stdout: str
    stderr: str
    value: int | None = None
    command: str = ""

    @property
    def error_message(self) -> str:
        """Get a formatted error message."""
        if self.success:
            return ""
        msg = f"Command failed with exit code {self.return_code}"
        if self.stderr:
            msg += f": {self.stderr.strip()}"
        return msg


class CommandExecutor:
    """
    Executor for ddcutil commands with retry logic and error handling.

    This class provides methods to execute ddcutil commands with proper
    timeout handling, retry logic for transient failures, and output parsing.
    """

    # Default configuration
    DEFAULT_TIMEOUT = 5.0  # seconds
    DEFAULT_MAX_RETRIES = 3
    DEFAULT_RETRY_DELAY = 0.1  # seconds
    DEFAULT_RETRY_BACKOFF = 2.0  # multiplier for exponential backoff

    # Patterns for parsing ddcutil output
    GETVCP_PATTERN = re.compile(r"VCP\s+(0x[0-9A-Fa-f]{2})\s+.*\s+([0-9]+)")
    DETECT_PATTERN = re.compile(r"I2C bus:\s*/dev/i2c-(\d+)")
    EDID_PATTERN = re.compile(
        r"Model:\s*(.+?)\s*\nSerial number:\s*(.+?)\s*\n"
        r"Manufactured:\s*(.+?)\s*\n"
        r"EDID version:\s*(.+?)\s*\n"
        r"Manufacturing ID:\s*(.+?)\s*\n"
        r"Display type:\s*(.+?)\s*\n"
    )

    def __init__(
        self,
        timeout: float = DEFAULT_TIMEOUT,
        max_retries: int = DEFAULT_MAX_RETRIES,
        retry_delay: float = DEFAULT_RETRY_DELAY,
        retry_backoff: float = DEFAULT_RETRY_BACKOFF,
    ) -> None:
        """
        Initialize the CommandExecutor.

        Args:
            timeout: Default timeout for command execution in seconds
            max_retries: Maximum number of retry attempts for transient failures
            retry_delay: Initial delay between retries in seconds
            retry_backoff: Multiplier for exponential backoff
        """
        self.timeout = timeout
        self.max_retries = max_retries
        self.retry_delay = retry_delay
        self.retry_backoff = retry_backoff
        self._ddcutil_path: str | None = None

    def check_ddcutil_available(self) -> bool:
        """
        Check if ddcutil is available on the system.

        Returns:
            True if ddcutil is available, False otherwise
        """
        try:
            result = subprocess.run(
                ["ddcutil", "--version"],
                capture_output=True,
                text=True,
                timeout=2.0,
            )
            if result.returncode == 0:
                # Parse version to store path
                self._ddcutil_path = "ddcutil"
                logger.info(f"ddcutil found: {result.stdout.strip()}")
                return True
        except (subprocess.TimeoutExpired, FileNotFoundError):
            pass

        logger.warning("ddcutil not found on system")
        return False

    def _get_ddcutil_path(self) -> str:
        """
        Get the path to the ddcutil executable.

        Returns:
            Path to ddcutil

        Raises:
            DDCNotAvailableError: If ddcutil is not available
        """
        if self._ddcutil_path is None and not self.check_ddcutil_available():
            raise DDCNotAvailableError()
        return self._ddcutil_path or "ddcutil"

    def _execute(
        self,
        args: list[str],
        timeout: float | None = None,
    ) -> CommandResult:
        """
        Execute a ddcutil command without retry logic.

        Args:
            args: Command arguments (excluding ddcutil executable)
            timeout: Timeout in seconds, or None to use default

        Returns:
            CommandResult with execution details

        Raises:
            TimeoutError: If the command times out
        """
        command = [self._get_ddcutil_path()] + args
        timeout = timeout if timeout is not None else self.timeout
        command_str = " ".join(shlex.quote(arg) for arg in command)

        logger.debug(f"Executing: {command_str}")

        try:
            result = subprocess.run(
                command,
                capture_output=True,
                text=True,
                timeout=timeout,
            )

            return CommandResult(
                success=result.returncode == 0,
                return_code=result.returncode,
                stdout=result.stdout,
                stderr=result.stderr,
                command=command_str,
            )

        except subprocess.TimeoutExpired as e:
            logger.error(f"Command timed out after {timeout}s: {command_str}")
            raise TimeoutError(
                message=f"ddcutil command timed out after {timeout} seconds",
                timeout_seconds=timeout,
            ) from e
        except FileNotFoundError as e:
            logger.error("ddcutil executable not found")
            raise DDCNotAvailableError() from e
        except Exception as e:
            logger.error(f"Unexpected error executing command: {e}")
            raise CommandExecutionError(
                message=f"Unexpected error executing ddcutil command: {e}",
                command=command_str,
            ) from e

    def _should_retry(self, result: CommandResult) -> bool:
        """
        Determine if a failed command should be retried.

        Args:
            result: The command result to check

        Returns:
            True if the command should be retried, False otherwise
        """
        # Don't retry if it was a success
        if result.success:
            return False

        # Check for permission errors (don't retry)
        if "Permission denied" in result.stderr or "Permission" in result.stderr:
            return False

        # Check for monitor not found errors (don't retry)
        if "monitor not found" in result.stderr.lower():
            return False

        # Check for I2C bus errors (transient, should retry)
        if "i2c" in result.stderr.lower() or "ddc" in result.stderr.lower():
            return True

        # Check for timeout-related errors (should retry)
        if "timeout" in result.stderr.lower():
            return True

        # Default to not retry for unknown errors
        return False

    def execute(
        self,
        args: list[str],
        timeout: float | None = None,
        max_retries: int | None = None,
    ) -> CommandResult:
        """
        Execute a ddcutil command with retry logic.

        Args:
            args: Command arguments (excluding ddcutil executable)
            timeout: Timeout in seconds, or None to use default
            max_retries: Maximum number of retry attempts, or None to use default

        Returns:
            CommandResult with execution details

        Raises:
            DDCNotAvailableError: If ddcutil is not available
            PermissionError: If permission errors occur
            CommandExecutionError: If the command fails after all retries
        """
        max_retries = max_retries if max_retries is not None else self.max_retries
        retry_delay = self.retry_delay

        last_result: CommandResult | None = None

        for attempt in range(max_retries + 1):
            try:
                result = self._execute(args, timeout)

                # Check for permission errors
                if not result.success and "Permission denied" in result.stderr:
                    raise PermissionError(
                        message="Permission denied accessing I2C device",
                        stderr=result.stderr,
                    )

                # If successful or should not retry, return result
                if result.success or not self._should_retry(result):
                    return result

                last_result = result

            except TimeoutError:
                # Timeout errors are retryable
                if attempt < max_retries:
                    logger.warning(
                        f"Command timed out, retrying ({attempt + 1}/{max_retries})"
                    )
                    time.sleep(retry_delay)
                    retry_delay *= self.retry_backoff
                    continue
                raise

            except (PermissionError, DDCNotAvailableError):
                # These errors should not be retried
                raise

            except CommandExecutionError:
                # Unexpected errors should not be retried
                raise

        # If we get here, all retries failed
        if last_result:
            raise CommandExecutionError(
                message=f"Command failed after {max_retries} retries",
                command=last_result.command,
                exit_code=last_result.return_code,
                stderr=last_result.stderr,
            )

        raise CommandExecutionError(
            message="Command failed after retries (no result available)"
        )

    def parse_getvcp_output(self, output: str) -> tuple[int, int] | None:
        """
        Parse the output of a ddcutil getvcp command.

        Args:
            output: The stdout from getvcp command

        Returns:
            Tuple of (vcp_code, value) if parsing succeeds, None otherwise
        """
        # Try to match the standard ddcutil output format
        # Example: "VCP 10 (Brightness): current value = 75, maximum value = 100"
        match = re.search(r"VCP\s+(0x[0-9A-Fa-f]{2}|\d+)\s+.*current value\s*=\s*(\d+)", output)
        if match:
            code_str, value_str = match.groups()
            code = int(code_str, 0)  # Handles both hex and decimal
            value = int(value_str)
            return (code, value)

        # Alternative format: "0x10: 75 (0x4B)"
        match = re.search(r"(0x[0-9A-Fa-f]{2}):\s*(\d+)", output)
        if match:
            code = int(match.group(1), 0)
            value = int(match.group(2))
            return (code, value)

        return None

    def get_vcp(
        self,
        bus: int | None = None,
        vcp_code: int = 0x10,
        timeout: float | None = None,
    ) -> CommandResult:
        """
        Execute a ddcutil getvcp command.

        Args:
            bus: I2C bus number, or None for default monitor
            vcp_code: VCP code to query
            timeout: Timeout in seconds, or None to use default

        Returns:
            CommandResult with the parsed value in the 'value' field
        """
        args = ["getvcp", f"0x{vcp_code:02X}"]
        if bus is not None:
            args = [f"--bus={bus}"] + args

        result = self.execute(args, timeout)

        # Parse the output to extract the value
        if result.success:
            parsed = self.parse_getvcp_output(result.stdout)
            if parsed:
                result.value = parsed[1]

        return result

    def set_vcp(
        self,
        bus: int | None = None,
        vcp_code: int = 0x10,
        value: int = 50,
        timeout: float | None = None,
    ) -> CommandResult:
        """
        Execute a ddcutil setvcp command.

        Args:
            bus: I2C bus number, or None for default monitor
            vcp_code: VCP code to set
            value: Value to set
            timeout: Timeout in seconds, or None to use default

        Returns:
            CommandResult
        """
        args = ["setvcp", f"0x{vcp_code:02X}", str(value)]
        if bus is not None:
            args = [f"--bus={bus}"] + args

        return self.execute(args, timeout)

    def detect_monitors(self, timeout: float | None = None) -> CommandResult:
        """
        Execute a ddcutil detect command.

        Args:
            timeout: Timeout in seconds, or None to use default

        Returns:
            CommandResult with monitor detection output
        """
        args = ["detect"]
        return self.execute(args, timeout)

    def vcp_info(
        self,
        bus: int | None = None,
        timeout: float | None = None,
    ) -> CommandResult:
        """
        Execute a ddcutil vcpinfo command.

        Args:
            bus: I2C bus number, or None for default monitor
            timeout: Timeout in seconds, or None to use default

        Returns:
            CommandResult with VCP capability information
        """
        args = ["vcpinfo"]
        if bus is not None:
            args = [f"--bus={bus}"] + args

        return self.execute(args, timeout)

    def query_capabilities(
        self,
        bus: int | None = None,
        timeout: float | None = None,
    ) -> CommandResult:
        """
        Query monitor capabilities using ddcutil capabilities.

        Args:
            bus: I2C bus number, or None for default monitor
            timeout: Timeout in seconds, or None to use default

        Returns:
            CommandResult with capabilities output
        """
        args = ["capabilities"]
        if bus is not None:
            args = [f"--bus={bus}"] + args

        return self.execute(args, timeout)
