# Twinkle Linux

A GUI application for controlling external monitor brightness via DDC/CI on Linux/Ubuntu systems.

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Python](https://img.shields.io/badge/python-3.10%2B-blue.svg)
![Platform](https://img.shields.io/badge/platform-Linux-lightgrey.svg)

## Overview

Twinkle Linux provides system tray integration for easy brightness control of external monitors connected via DisplayPort or HDMI. It uses the DDC/CI (Display Data Channel Command Interface) protocol to communicate with monitors, allowing you to adjust brightness, contrast, and other settings directly from your desktop.

### Features

- **System Tray Integration**: Access brightness controls from your system tray
- **Multi-Monitor Support**: Control multiple monitors independently
- **Brightness Presets**: Quickly switch between day, night, and custom brightness levels
- **Keyboard Shortcuts**: Control brightness with customizable hotkeys
- **Auto-Start**: Automatically start with your desktop session
- **Configuration Persistence**: Save your preferences across sessions

### Supported Monitors

Twinkle Linux is designed to work with DDC/CI-compliant monitors, including:
- **Lenovo T24i** (Primary target)
- **Dell Monitors** (U-series, P-series, S-series)
- **LG Monitors** (UltraFine, UltraGear series)
- Other DDC/CI-compliant monitors

### Screenshots

<!-- Screenshots will be added here in future releases -->

*Coming soon: Screenshots showing the system tray integration, brightness slider, settings dialog, and multi-monitor support.*

## Requirements

### System Requirements

- Ubuntu 20.04 LTS or later (or compatible Linux distribution)
- Linux kernel with i2c-dev module support
- Python 3.10 or later

### System Packages

**ddcutil** is required for DDC/CI communication with monitors. It must be installed via your system package manager.

#### Ubuntu/Debian
```bash
# Install ddcutil for DDC/CI communication
sudo apt update
sudo apt install ddcutil

# Optional: Install i2c-tools for debugging and development
sudo apt install i2c-tools

# Optional: Install udev for permission management
sudo apt install udev
```

#### Fedora/RHEL/CentOS
```bash
# Install ddcutil
sudo dnf install ddcutil

# Optional: Install i2c-tools
sudo dnf install i2c-tools
```

#### Arch Linux/Manjaro
```bash
# Install ddcutil
sudo pacman -S ddcutil

# Optional: Install i2c-tools
sudo pacman -S i2c-tools
```

#### openSUSE
```bash
# Install ddcutil
sudo zypper install ddcutil

# Optional: Install i2c-tools
sudo zypper install i2c-tools
```

**Note**: ddcutil is not a Python package and must be installed at the system level. The application wraps ddcutil command-line tool via subprocess calls.

### Python Dependencies

See [`requirements.txt`](requirements.txt) for the complete list:
- PyQt6 >= 6.4.0
- pydantic >= 2.0.0
- pyxdg >= 0.28

## Installation

Twinkle Linux provides convenient installation scripts to make setup easy. Choose the installation method that best fits your needs.

### Quick Installation (Recommended)

#### System-Wide Installation

For a system-wide installation with desktop integration:

```bash
git clone https://github.com/twinkle-linux/twinkle-linux.git
cd twinkle-linux
sudo ./scripts/install.sh
```

This script will:
- Check for dependencies (Python 3.8+, pip, PyQt6)
- Install Python dependencies via pip
- Install ddcutil via system package manager
- Copy desktop entry file to `/usr/share/applications/`
- Copy icon to `/usr/share/icons/hicolor/scalable/apps/`
- Create symbolic link for the main executable
- Prompt you to install udev rules

**Options:**
- `--skip-udev` - Skip udev rules installation
- `--help` - Show help message

#### User-Level Installation

For a user-level installation (no sudo required, except for udev rules):

```bash
git clone https://github.com/twinkle-linux/twinkle-linux.git
cd twinkle-linux
./scripts/install-user.sh
```

This script will:
- Install Twinkle Linux to `~/.local/`
- Install Python dependencies to user site-packages
- Copy desktop entry to `~/.local/share/applications/`
- Copy icon to `~/.local/share/icons/`
- Prompt you to install udev rules (requires sudo)

**Options:**
- `--skip-udev` - Skip udev rules installation
- `--help` - Show help message

### Udev Rules Installation

To allow non-root access to I2C devices for DDC/CI communication:

```bash
sudo ./scripts/install-udev-rules.sh
```

This script will:
- Copy udev rules to `/etc/udev/rules.d/`
- Reload udev rules
- Add your user to the i2c group
- Inform you to log out and back in

**Options:**
- `--no-add-user` - Don't add current user to i2c group
- `--help` - Show help message

### Manual Installation

#### From Source

1. Clone the repository:
```bash
git clone https://github.com/twinkle-linux/twinkle-linux.git
cd twinkle-linux
```

2. Create a virtual environment (recommended):
```bash
python3 -m venv venv
source venv/bin/activate
```

3. Install dependencies:
```bash
pip install -r requirements.txt
```

4. Run the application:
```bash
python src/main.py
```

#### Manual System-Wide Installation

For a system-wide installation with desktop integration:

1. Clone the repository:
```bash
git clone https://github.com/twinkle-linux/twinkle-linux.git
cd twinkle-linux
```

2. Install dependencies:
```bash
sudo apt install ddcutil python3-pip
pip3 install -r requirements.txt
```

3. Install the desktop entry file:
```bash
sudo cp packaging/twinkle-linux.desktop /usr/share/applications/
```

4. Install the icon:
```bash
sudo mkdir -p /usr/share/icons/hicolor/scalable/apps/
sudo cp src/ui/resources/icons/twinkle.svg /usr/share/icons/hicolor/scalable/apps/twinkle-linux.svg
sudo update-icon-caches /usr/share/icons/hicolor/
```

5. Create a launcher script (optional):
```bash
sudo cat > /usr/local/bin/twinkle-linux << 'EOF'
#!/bin/bash
cd /opt/twinkle-linux
python3 src/main.py "$@"
EOF
sudo chmod +x /usr/local/bin/twinkle-linux
```

#### Manual User Installation

For a user-level installation (no sudo required):

1. Clone the repository:
```bash
git clone https://github.com/twinkle-linux/twinkle-linux.git
cd twinkle-linux
```

2. Install dependencies:
```bash
pip3 install -r requirements.txt
```

3. Install the desktop entry file:
```bash
mkdir -p ~/.local/share/applications/
cp packaging/twinkle-linux.desktop ~/.local/share/applications/
```

4. Install the icon:
```bash
mkdir -p ~/.local/share/icons/hicolor/scalable/apps/
cp src/ui/resources/icons/twinkle.svg ~/.local/share/icons/hicolor/scalable/apps/twinkle-linux.svg
update-icon-caches ~/.local/share/icons/hicolor/ 2>/dev/null || true
```

#### Manual Udev Rules Installation

To allow non-root access to I2C devices for DDC/CI communication:

1. Copy the udev rules file:
```bash
sudo cp packaging/99-twinkle-i2c.rules /etc/udev/rules.d/
```

2. Reload udev rules:
```bash
sudo udevadm control --reload-rules
sudo udevadm trigger
```

3. Add your user to the i2c group:
```bash
sudo usermod -a -G i2c $USER
```

4. Log out and log back in for the changes to take effect.

### Flatpak Installation

To build and install as a Flatpak:

1. Install Flatpak builder:
```bash
sudo apt install flatpak flatpak-builder
flatpak remote-add --if-not-exists flathub https://flathub.org/repo/flathub.flatpakrepo
```

2. Build the Flatpak:
```bash
cd packaging/flatpak
flatpak-builder build com.github.twinkle-twinkle.TwinkleLinux.yml
```

3. Install the Flatpak:
```bash
flatpak-builder --user --install --force build com.github.twinkle-twinkle.TwinkleLinux.yml
```

4. Run the application:
```bash
flatpak run com.github.twinkle-twinkle.TwinkleLinux
```

### Snap Installation

To build and install as a Snap:

1. Install Snapcraft:
```bash
sudo snap install snapcraft --classic
```

2. Build the Snap:
```bash
cd packaging/snap
snapcraft
```

3. Install the Snap:
```bash
sudo snap install --dangerous twinkle-linux_*.snap
```

4. Connect the required interfaces:
```bash
sudo snap connect twinkle-linux:i2c
sudo snap connect twinkle-linux:hardware-observe
```

5. Run the application:
```bash
twinkle-linux
```

### Development Installation

For development, install with development dependencies:
```bash
pip install -e ".[dev]"
```

## Uninstallation

### System-Wide Uninstallation

To remove Twinkle Linux system-wide:

```bash
sudo ./scripts/uninstall.sh
```

This script will:
- Remove desktop entry file
- Remove icon files
- Remove symbolic links
- Prompt about removing udev rules and config files
- Prompt about removing Python packages

**Options:**
- `--keep-config` - Keep configuration files
- `--keep-udev` - Keep udev rules
- `--help` - Show help message

### User-Level Uninstallation

To remove Twinkle Linux from your user directory:

```bash
./scripts/uninstall-user.sh
```

This script will:
- Remove desktop entry file
- Remove icon files
- Remove executable script
- Prompt about removing config files
- Prompt about removing Python packages

**Options:**
- `--keep-config` - Keep configuration files
- `--help` - Show help message

**Note:** Udev rules are not removed by the user-level uninstall script. To remove udev rules, run:
```bash
sudo rm /etc/udev/rules.d/99-twinkle-i2c.rules
sudo udevadm control --reload-rules
sudo udevadm trigger
```

### Manual Uninstallation

If you installed manually, you can remove Twinkle Linux by:

1. Remove the desktop entry:
```bash
sudo rm /usr/share/applications/twinkle-linux.desktop
# or for user installation:
rm ~/.local/share/applications/twinkle-linux.desktop
```

2. Remove the icon:
```bash
sudo rm /usr/share/icons/hicolor/scalable/apps/twinkle-linux.svg
# or for user installation:
rm ~/.local/share/icons/hicolor/scalable/apps/twinkle-linux.svg
```

3. Remove the executable:
```bash
sudo rm /usr/local/bin/twinkle-linux
# or for user installation:
rm ~/.local/bin/twinkle-linux
```

4. Remove the application directory:
```bash
sudo rm -rf /opt/twinkle-linux
# or for user installation:
rm -rf ~/.local/share/twinkle-linux
```

5. Remove configuration files (optional):
```bash
rm -rf ~/.config/twinkle-linux
rm -rf ~/.cache/twinkle-linux
```

## Usage

### Basic Usage

Run the application from the command line:
```bash
python src/main.py
```

### Command-Line Options

```
usage: twinkle-linux [-h] [--version] [-v] [--debug] [--quiet] [--config PATH]
                     [--log-file PATH] [--no-auto-start]

GUI application for controlling external monitor brightness via DDC/CI on Linux/Ubuntu.

options:
  -h, --help            Show help message and exit
  --version             Show version information and exit
  -v, --verbose         Enable verbose logging (INFO level)
  --debug               Enable debug mode (DEBUG level logging)
  --quiet               Suppress console output (only log to file)
  --config PATH         Path to configuration file
  --log-file PATH       Path to log file
  --no-auto-start       Disable auto-start on login
```

### Setting Up Permissions

To access I2C devices for DDC/CI communication, you need appropriate permissions.

**Note:** The recommended approach is to install the provided udev rules (see "Udev Rules Installation" in the Installation section above).

#### Quick Setup

```bash
# Install udev rules
sudo cp packaging/99-twinkle-i2c.rules /etc/udev/rules.d/
sudo udevadm control --reload-rules
sudo udevadm trigger

# Add user to i2c group
sudo usermod -a -G i2c $USER

# Log out and log back in for changes to take effect
```

#### Verify Permissions

After setting up permissions, verify I2C device access:

```bash
# Check i2c devices
ls -l /dev/i2c-*

# Test ddcutil detection
ddcutil detect
```

If you see permission denied errors, ensure you've logged out and back in after adding yourself to the i2c group.

## Configuration

Twinkle Linux stores its configuration in JSON format. By default, the configuration file is located at:

```
~/.config/twinkle-linux/config.json
```

### Configuration Schema

```json
{
  "version": "1.0",
  "monitors": {
    "by_serial": {
      "LEN123456": {
        "name": "Lenovo T24i",
        "bus": "i2c-3",
        "preferred_brightness": 75,
        "brightness_presets": {
          "day": 80,
          "night": 30,
          "gaming": 100
        },
        "enabled_vcp_codes": ["0x10", "0x12", "0x14"]
      }
    }
  },
  "ui": {
    "show_brightness_in_tray": true,
    "tray_icon_style": "percentage",
    "slider_position": "follow_cursor",
    "theme": "system"
  },
  "behavior": {
    "auto_start": true,
    "minimize_to_tray": true,
    "brightness_step": 5,
    "response_timeout_ms": 500,
    "max_retries": 3
  },
  "shortcuts": {
    "increase_brightness": "Ctrl+Alt+Up",
    "decrease_brightness": "Ctrl+Alt+Down"
  }
}
```

## Development

### Project Structure

```
twinkle-linux/
├── src/
│   ├── main.py              # Application entry point
│   ├── core/                # Core application logic
│   │   ├── app.py           # Application controller
│   │   ├── config.py        # Configuration management
│   │   └── logging.py       # Logging configuration
│   ├── ddc/                 # DDC/CI abstraction layer
│   ├── ui/                  # User interface layer
│   ├── services/            # Application services
│   └── utils/               # Utility functions
├── tests/                   # Test suite
├── packaging/               # Distribution packaging
├── scripts/                 # Utility scripts
├── docs/                    # Documentation
├── pyproject.toml           # Project configuration
├── requirements.txt         # Python dependencies
└── README.md
```

### Running Tests

```bash
# Run all tests
pytest

# Run with coverage
pytest --cov=src

# Run specific test file
pytest tests/unit/test_config.py
```

### Code Style

This project uses:
- **Black** for code formatting
- **Pylint** for linting
- **MyPy** for type checking

```bash
# Format code
black src tests

# Run linter
pylint src

# Run type checker
mypy src
```

## Troubleshooting

### Monitor Not Detected

1. Verify ddcutil is installed:
```bash
ddcutil detect
```

2. Check I2C permissions:
```bash
ls -l /dev/i2c-*
```

3. Ensure your monitor supports DDC/CI

### Permission Denied Errors

Add your user to the i2c group or install udev rules (see "Setting Up Permissions" above).

### Slow Response

Some monitors have slower DDC/CI response times. You can adjust the timeout and retry settings in the configuration file.

## Contributing

Contributions are welcome! Please see [`CONTRIBUTING.md`](CONTRIBUTING.md) for guidelines.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- **ddcutil**: The command-line tool that makes DDC/CI communication possible on Linux
- **PyQt6**: For the excellent GUI framework
- **BetterDisplay**: Inspiration for the system tray integration design

## Roadmap

- [x] Phase 1: Foundation (Core Infrastructure)
- [x] Phase 2: DDC/CI Backend
- [x] Phase 3: System Tray Integration
- [x] Phase 4: Multi-Monitor Support
- [x] Phase 5: Settings and Configuration
- [x] Phase 6: Polish and Packaging
- [x] Phase 7: Testing and Quality Assurance
- [x] Phase 8: Distribution and Release
- [x] Phase 9: Desktop Integration and Packaging

See [`plans/architecture-design.md`](plans/architecture-design.md) for detailed implementation plans.

## Support

- **Issues**: Report bugs and feature requests on [GitHub Issues](https://github.com/twinkle-linux/twinkle-linux/issues)
- **Documentation**: See the [`docs/`](docs/) directory for detailed documentation
- **Discussions**: Join the conversation in [GitHub Discussions](https://github.com/twinkle-linux/twinkle-linux/discussions)
