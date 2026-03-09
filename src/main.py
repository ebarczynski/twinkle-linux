"""
Main entry point for Twinkle Linux.

This module provides the command-line interface and application initialization
for Twinkle Linux, a GUI application for controlling external monitor brightness
via DDC/CI on Linux/Ubuntu.
"""

import argparse
import logging
import os
import sys

from PyQt6.QtWidgets import QApplication

from src.core.app import TwinkleApp
from src.core.logging import setup_logging, get_logger


# Package version
__version__ = "0.1.0"
__author__ = "Twinkle Linux Contributors"
__license__ = "MIT"


logger = get_logger(__name__)


def parse_arguments() -> argparse.Namespace:
    """
    Parse command-line arguments.

    Returns:
        Parsed command-line arguments.
    """
    parser = argparse.ArgumentParser(
        prog="twinkle-linux",
        description="GUI application for controlling external monitor brightness via DDC/CI on Linux/Ubuntu.",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  %(prog)s                    Start the application
  %(prog)s --verbose          Enable verbose logging
  %(prog)s --version          Show version information
  %(prog)s --debug            Enable debug mode
        """,
    )

    parser.add_argument(
        "--version",
        action="version",
        version=f"%(prog)s {__version__}",
        help="Show version information and exit",
    )

    parser.add_argument(
        "-v",
        "--verbose",
        action="store_true",
        help="Enable verbose logging (INFO level)",
    )

    parser.add_argument(
        "--debug",
        action="store_true",
        help="Enable debug mode (DEBUG level logging)",
    )

    parser.add_argument(
        "--quiet",
        action="store_true",
        help="Suppress console output (only log to file)",
    )

    parser.add_argument(
        "--config",
        type=str,
        metavar="PATH",
        help="Path to configuration file (default: XDG config location)",
    )

    parser.add_argument(
        "--log-file",
        type=str,
        metavar="PATH",
        help="Path to log file (default: XDG state location)",
    )

    parser.add_argument(
        "--no-auto-start",
        action="store_true",
        help="Disable auto-start on login (overrides config)",
    )

    return parser.parse_args()


def setup_application_logging(args: argparse.Namespace) -> None:
    """
    Configure application logging based on command-line arguments.

    Args:
        args: Parsed command-line arguments.
    """
    if args.debug:
        log_level = "DEBUG"
    elif args.verbose:
        log_level = "INFO"
    else:
        log_level = "WARNING"

    log_file = None
    if args.log_file:
        from pathlib import Path

        log_file = Path(args.log_file)

    setup_logging(
        log_level=log_level,
        log_file=log_file,
        enable_console=not args.quiet,
    )

    logger.info(f"Twinkle Linux v{__version__} starting")
    logger.debug(f"Command-line arguments: {vars(args)}")


def main() -> int:
    """
    Main entry point for the application.

    Returns:
        Exit code (0 for success, non-zero for error).
    """
    # Parse command-line arguments
    args = parse_arguments()

    # Set up logging
    setup_application_logging(args)

    # Create Qt application
    qt_app = QApplication(sys.argv)
    qt_app.setApplicationName("Twinkle Linux")
    qt_app.setApplicationVersion(__version__)
    qt_app.setOrganizationName("Twinkle Linux")

    # Create Twinkle application controller
    app = TwinkleApp(qt_app)

    # Initialize the application
    try:
        app.initialize()
    except Exception as e:
        logger.error(f"Failed to initialize application: {e}")
        return 1

    # Handle application quit
    qt_app.aboutToQuit.connect(lambda: logger.info("Qt application about to quit"))

    # Run the application
    exit_code = 0
    try:
        exit_code = app.run()
    except KeyboardInterrupt:
        logger.info("Application interrupted by user")
        exit_code = 0
    except Exception as e:
        logger.error(f"Application error: {e}", exc_info=True)
        exit_code = 1
    finally:
        app.quit(exit_code)

    return exit_code


if __name__ == "__main__":
    sys.exit(main())
