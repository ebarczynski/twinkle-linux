# Twinkle Linux (Rust Version)

A GUI application for controlling external monitor brightness via DDC/CI on Linux.

This is a Rust rewrite of the Python version of Twinkle Linux, providing the same functionality with improved performance and type safety.

## Features

- **DDC/CI Backend**: Wrapper around ddcutil subprocess calls for monitor control
- **System Tray Integration**: GTK4-based system tray icon with context menu
- **Brightness Control**: Brightness slider with real-time adjustment and debouncing
- **Multi-Monitor Support**: Monitor selector dropdown with per-monitor brightness tracking
- **Additional VCP Controls**: Contrast, Volume, Input Source, Color Temperature
- **Settings Dialog**: Comprehensive settings with General, UI, Behavior, and Advanced tabs

## Requirements

- Linux operating system
- ddcutil installed and configured
- GTK4 development libraries
- Rust 1.70 or later

### Installing ddcutil

```bash
# Ubuntu/Debian
sudo apt install ddcutil

# Fedora
sudo dnf install ddcutil

# Arch Linux
sudo pacman -S ddcutil
```

### Installing GTK4 development libraries

```bash
# Ubuntu/Debian
sudo apt install libgtk-4-dev

# Fedora
sudo dnf install gtk4-devel

# Arch Linux
sudo pacman -S gtk4
```

## Building

```bash
cd twinkle-rust
cargo build --release
```

## Running

```bash
cargo run
```

Or run the release binary:

```bash
./target/release/twinkle-linux
```

## Cross-Compilation

You can cross-compile Twinkle Linux for Linux from macOS using Docker and the `cross` tool.

### Quick Start

```bash
# Install cross
cargo install cross

# Build for Linux (x86_64)
./cross-build.sh x86_64-unknown-linux-gnu release

# Build for ARM64 (Raspberry Pi)
./cross-build.sh aarch64-unknown-linux-gnu release
```

For detailed instructions, see [`CROSS_COMPILE.md`](CROSS_COMPILE.md).

## Project Structure

```
twinkle-rust/
├── src/
│   ├── main.rs           # Application entry point
│   ├── lib.rs            # Library exports
│   ├── ddc/              # DDC/CI backend
│   │   ├── mod.rs
│   │   ├── command.rs    # ddcutil command executor
│   │   ├── ddc_manager.rs # Main DDC manager
│   │   ├── error.rs      # Error types
│   │   ├── monitor.rs    # Monitor detection and management
│   │   └── vcp_codes.rs  # VCP code definitions
│   ├── ui/               # UI components
│   │   ├── mod.rs
│   │   ├── brightness_popup.rs
│   │   ├── tray_icon.rs
│   │   └── widgets/
│   │       ├── mod.rs
│   │       ├── brightness_slider.rs
│   │       ├── settings_dialog.rs
│   │       └── vcp_controls.rs
│   ├── core/             # Core functionality
│   │   ├── mod.rs
│   │   └── config.rs     # Configuration management
│   └── utils/            # Utility functions
│       └── mod.rs
├── Cargo.toml
├── README.md
├── IMPLEMENTATION_SUMMARY.md  # Detailed implementation notes
├── CROSS_COMPILE.md            # Cross-compilation guide
├── .cross.toml               # Cross-compilation configuration
├── Dockerfile.cross           # Docker image for cross-compilation
└── cross-build.sh            # Cross-compilation build script
```

## Configuration

Configuration is stored in `~/.config/twinkle-linux/config.json`.

### Example Configuration

```json
{
  "general": {
    "autostart": false,
    "theme": "system",
    "language": "en_US"
  },
  "ui": {
    "auto_hide_delay_ms": 3000,
    "show_monitor_selector": false,
    "enable_presets": true,
    "preset_values": [20, 40, 60, 80, 100]
  },
  "behavior": {
    "debounce_delay_ms": 100,
    "remember_brightness": true,
    "restore_brightness": true,
    "enable_notifications": true
  },
  "monitors": [],
  "advanced": {
    "command_timeout_secs": 5.0,
    "max_retries": 3,
    "debug_logging": false,
    "log_file_path": ""
  }
}
```

## License

MIT License - See the LICENSE file in the parent directory.

## Contributing

Contributions are welcome! Please see the CONTRIBUTING.md file in the parent directory for guidelines.
