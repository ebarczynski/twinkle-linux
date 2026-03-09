# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 2026-03-06

### Added

#### Core Features
- Initial release of Twinkle Linux
- DDC/CI backend for monitor communication
- System tray integration with brightness control
- Multi-monitor support with independent control
- Brightness presets (day, night, custom)
- Keyboard shortcuts for brightness adjustment
- Auto-start on desktop session login
- Configuration persistence across sessions

#### User Interface
- System tray icon with context menu
- Brightness slider popup for quick adjustments
- Settings dialog for configuration
- VCP controls panel for additional monitor settings
- Monitor selection interface
- Dark/Light theme support

#### Monitor Support
- DDC/CI protocol implementation
- Support for VCP codes:
  - Brightness (0x10)
  - Contrast (0x12)
  - Color temperature (0x14)
  - Audio volume (0x62)
  - Input source (0x60)
  - Additional monitor-specific VCP codes
- Multi-monitor detection and management
- Monitor identification by serial number
- Bus assignment and management

#### Installation & Packaging
- Installation scripts for system-wide and user-level installation
- Udev rules for non-root I2C access
- Desktop entry file for application menu integration
- Icon files for various sizes
- Flatpak packaging support
- Snap packaging support
- Uninstallation scripts

#### Configuration
- JSON-based configuration file
- Per-monitor settings
- Brightness presets configuration
- Keyboard shortcut customization
- UI preferences
- Behavior settings (timeout, retries, etc.)

#### Development
- Comprehensive project structure
- Unit and integration test framework
- Logging system with multiple levels
- Type hints throughout the codebase
- Configuration management with validation
- Exception handling for DDC/CI operations

### Fixed
- Initial implementation - no known issues

### Security
- Udev rules for secure I2C device access
- Proper permission handling for I2C devices

### Documentation
- Comprehensive README with installation instructions
- Architecture design document
- User guide
- Contributing guidelines
- Code documentation

## [Unreleased]

### Planned
- Additional monitor features
- Profile management
- Scheduled brightness adjustments
- More VCP code support

[1.0.0]: https://github.com/twinkle-linux/twinkle-linux/releases/tag/v1.0.0
