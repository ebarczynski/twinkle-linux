"""
Logging configuration module for Twinkle Linux.

This module provides functions to configure application logging with
support for both file and console handlers, configurable log levels,
and structured log formatting.
"""

import logging
import sys
from logging.handlers import RotatingFileHandler
from pathlib import Path
from typing import Optional


def setup_logging(
    log_level: str = "INFO",
    log_file: Optional[Path] = None,
    enable_console: bool = True,
    max_file_size: int = 10 * 1024 * 1024,  # 10 MB
    backup_count: int = 5,
) -> logging.Logger:
    """
    Configure application logging with file and console handlers.

    Args:
        log_level: Logging level (DEBUG, INFO, WARNING, ERROR, CRITICAL).
        log_file: Path to log file. If None, uses default location.
        enable_console: Whether to enable console logging.
        max_file_size: Maximum size of log file in bytes before rotation.
        backup_count: Number of backup log files to keep.

    Returns:
        The configured root logger.
    """
    # Get the root logger
    root_logger = logging.getLogger()
    root_logger.setLevel(getattr(logging, log_level.upper(), logging.INFO))

    # Remove any existing handlers
    root_logger.handlers.clear()

    # Create formatter
    formatter = logging.Formatter(
        fmt="%(asctime)s - %(name)s - %(levelname)s - %(message)s",
        datefmt="%Y-%m-%d %H:%M:%S",
    )

    # Add file handler if log_file is specified or default location exists
    if log_file is None:
        log_file = _get_default_log_path()

    try:
        log_file.parent.mkdir(parents=True, exist_ok=True)

        file_handler = RotatingFileHandler(
            log_file,
            maxBytes=max_file_size,
            backupCount=backup_count,
            encoding="utf-8",
        )
        file_handler.setLevel(logging.DEBUG)  # File gets all logs
        file_handler.setFormatter(formatter)
        root_logger.addHandler(file_handler)

    except Exception as e:
        # If file handler fails, log to stderr
        print(f"Warning: Failed to set up file logging: {e}", file=sys.stderr)

    # Add console handler if enabled
    if enable_console:
        console_handler = logging.StreamHandler(sys.stdout)
        console_handler.setLevel(getattr(logging, log_level.upper(), logging.INFO))
        console_handler.setFormatter(formatter)
        root_logger.addHandler(console_handler)

    return root_logger


def _get_default_log_path() -> Path:
    """
    Get the default log file path.

    Returns:
        Path to the default log file.
    """
    try:
        from xdg_base_dirs import xdg_state_home

        log_dir = xdg_state_home() / "twinkle-linux" / "logs"
    except ImportError:
        # Fallback to ~/.local/state if pyxdg is not available
        log_dir = Path.home() / ".local" / "state" / "twinkle-linux" / "logs"

    log_dir.mkdir(parents=True, exist_ok=True)
    return log_dir / "twinkle.log"


def get_logger(name: str) -> logging.Logger:
    """
    Get a logger with the specified name.

    Args:
        name: Name of the logger (typically __name__ of the calling module).

    Returns:
        A logger instance with the specified name.
    """
    return logging.getLogger(name)


def set_log_level(level: str) -> None:
    """
    Set the logging level for all handlers.

    Args:
        level: Logging level (DEBUG, INFO, WARNING, ERROR, CRITICAL).
    """
    log_level = getattr(logging, level.upper(), logging.INFO)
    root_logger = logging.getLogger()
    root_logger.setLevel(log_level)

    for handler in root_logger.handlers:
        # Don't change file handler level (keep at DEBUG)
        if not isinstance(handler, RotatingFileHandler):
            handler.setLevel(log_level)


def disable_console_logging() -> None:
    """Disable console logging by removing console handlers."""
    root_logger = logging.getLogger()
    root_logger.handlers = [
        h for h in root_logger.handlers if isinstance(h, RotatingFileHandler)
    ]


def enable_console_logging(level: str = "INFO") -> None:
    """
    Enable console logging.

    Args:
        level: Logging level for console output.
    """
    root_logger = logging.getLogger()

    # Check if console handler already exists
    for handler in root_logger.handlers:
        if not isinstance(handler, RotatingFileHandler):
            return  # Console handler already exists

    # Add console handler
    console_handler = logging.StreamHandler(sys.stdout)
    console_handler.setLevel(getattr(logging, level.upper(), logging.INFO))
    formatter = logging.Formatter(
        fmt="%(asctime)s - %(name)s - %(levelname)s - %(message)s",
        datefmt="%Y-%m-%d %H:%M:%S",
    )
    console_handler.setFormatter(formatter)
    root_logger.addHandler(console_handler)
