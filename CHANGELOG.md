# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2025-05-09

### Added
- Rust implementation (twinkle-rust) as the primary application
- System tray icon via ksni (D-Bus StatusNotifierItem)
- Per-monitor brightness sliders with card-based dark UI
- "All Monitors" slider for controlling all displays at once
- Quick presets from tray: 10%, 20%, 40%, 60%, 80%, 100%
- DDC/CI support for external monitors (HDMI, DisplayPort, USB-C)
- Internal laptop panel support via systemd-logind D-Bus
- Intel (i915), AMD (amdgpu), NVIDIA, and ACPI backlight support
- Settings dialog with General, UI, Behavior, and Advanced tabs
- 300ms debounced sliders to prevent I2C flooding
- JSON config file at ~/.config/twinkle-linux/config.json
- GTK4 CSS dark theme with card-based layout
- C++23/26 implementation scaffold (twinkle-cpp)

### Removed
- Python implementation (src/, tests/, pyproject.toml, requirements.txt)
- Python-based install/uninstall scripts
- Dockerfiles and cross-compile tooling

### Changed
- twinkle-rust is now the primary and recommended implementation
- Updated GitHub Actions CI for Rust (fmt, clippy, test, build)
