# Twinkle Linux User Guide

A comprehensive guide to using Twinkle Linux for controlling external monitor brightness via DDC/CI.

## Table of Contents

1. [Introduction](#introduction)
2. [Getting Started](#getting-started)
3. [Basic Usage](#basic-usage)
4. [System Tray Integration](#system-tray-integration)
5. [Multi-Monitor Setup](#multi-monitor-setup)
6. [Brightness Controls](#brightness-controls)
7. [Settings and Configuration](#settings-and-configuration)
8. [Advanced Features](#advanced-features)
9. [Troubleshooting](#troubleshooting)
10. [FAQ](#faq)

## Introduction

Twinkle Linux is a GUI application that allows you to control external monitor brightness, contrast, and other settings using the DDC/CI (Display Data Channel Command Interface) protocol. It provides system tray integration for quick and easy access to monitor controls.

### Key Features

- **System Tray Integration**: Access brightness controls from your system tray
- **Multi-Monitor Support**: Control multiple monitors independently
- **Brightness Presets**: Quickly switch between day, night, and custom brightness levels
- **Keyboard Shortcuts**: Control brightness with customizable hotkeys
- **Auto-Start**: Automatically start with your desktop session
- **Additional VCP Controls**: Access contrast, color temperature, and more

### Supported Monitors

Twinkle Linux works with DDC/CI-compliant monitors, including:

- Lenovo T24i (Primary target)
- Dell Monitors (U-series, P-series, S-series)
- LG Monitors (UltraFine, UltraGear series)
- Other DDC/CI-compliant monitors

## Getting Started

### Installation

See the [README.md](../README.md) for detailed installation instructions. Quick start:

```bash
git clone https://github.com/twinkle-linux/twinkle-linux.git
cd twinkle-linux
sudo ./scripts/install.sh
```

### First-Time Setup

1. **Install Udev Rules** (Recommended)

   The installation script will prompt you to install udev rules. This allows non-root access to I2C devices.

   ```bash
   sudo ./scripts/install-udev-rules.sh
   ```

2. **Log Out and Back In**

   After installing udev rules, log out and log back in for the changes to take effect.

3. **Launch the Application**

   - From your application menu: Search for "Twinkle Linux"
   - From the command line: `twinkle-linux`
   - Or run from source: `python src/main.py`

4. **Verify Monitor Detection**

   The application should automatically detect your DDC/CI-compatible monitors. You can verify this by:

   ```bash
   ddcutil detect
   ```

## Basic Usage

### Launching the Application

Twinkle Linux can be launched in several ways:

**From Application Menu**
- Open your desktop's application menu
- Search for "Twinkle Linux"
- Click to launch

**From Command Line**
```bash
twinkle-linux
```

**Command Line Options**
```bash
twinkle-linux --help           # Show help message
twinkle-linux --version        # Show version
twinkle-linux --debug          # Enable debug mode
twinkle-linux --config PATH    # Use custom config file
twinkle-linux --no-auto-start  # Disable auto-start
```

### Quitting the Application

- Right-click the system tray icon and select "Quit"
- Or use the keyboard shortcut (if configured)

## System Tray Integration

The system tray icon provides quick access to all monitor controls.

### Tray Icon Features

**Single Click**
- Opens the brightness slider popup for the primary monitor

**Right Click**
- Opens the context menu with:
  - Monitor selection
  - Brightness presets
  - Settings
  - About
  - Quit

### Tray Icon Styles

The tray icon can display different information based on your settings:

- **Percentage**: Shows current brightness level (e.g., "75%")
- **Icon Only**: Shows a simple brightness icon
- **Monitor Name**: Shows the monitor name

Configure this in Settings → UI → Tray Icon Style

## Multi-Monitor Setup

Twinkle Linux supports multiple monitors, each with independent controls.

### Monitor Detection

Monitors are automatically detected when the application starts. Each monitor is identified by:

- **Serial Number**: Unique identifier for the monitor
- **Name**: Model name (e.g., "Lenovo T24i")
- **Bus**: I2C bus number (e.g., "i2c-3")

### Switching Between Monitors

**From Context Menu**
1. Right-click the tray icon
2. Hover over "Monitors"
3. Select the monitor you want to control

**From Brightness Popup**
1. Click the tray icon
2. Use the monitor dropdown to select a different monitor

### Per-Monitor Configuration

Each monitor can have its own:
- Brightness level
- Brightness presets
- Enabled VCP codes
- Preferred settings

## Brightness Controls

### Using the Brightness Slider

1. Click the system tray icon to open the brightness popup
2. Drag the slider to adjust brightness (0-100)
3. The change is applied immediately

### Using Keyboard Shortcuts

Default shortcuts:
- `Ctrl + Alt + Up`: Increase brightness
- `Ctrl + Alt + Down`: Decrease brightness

Customize these in Settings → Shortcuts

### Using Brightness Presets

Quick brightness presets for common scenarios:

**Day Preset**
- Higher brightness for well-lit environments
- Default: 80%

**Night Preset**
- Lower brightness for dark environments
- Default: 30%

**Custom Presets**
- Create your own presets in Settings
- Assign names and brightness levels

Access presets from:
- Right-click menu → Presets
- Settings → Brightness → Presets

## Settings and Configuration

### Accessing Settings

- Right-click the tray icon → Settings
- Or from the brightness popup → Settings button

### Settings Categories

#### Monitor Settings

Configure individual monitor settings:

- **Name**: Custom name for the monitor
- **Bus**: I2C bus assignment
- **Preferred Brightness**: Default brightness level
- **Brightness Presets**: Configure day, night, and custom presets
- **Enabled VCP Codes**: Select which VCP codes to control

#### UI Settings

Customize the user interface:

- **Show Brightness in Tray**: Display brightness level in tray icon
- **Tray Icon Style**: Choose display style (percentage, icon only, etc.)
- **Slider Position**: Where the brightness popup appears
- **Theme**: Light, dark, or system theme

#### Behavior Settings

Configure application behavior:

- **Auto-Start**: Start automatically on login
- **Minimize to Tray**: Minimize to system tray instead of closing
- **Brightness Step**: Amount to change per keyboard shortcut press
- **Response Timeout**: How long to wait for monitor response (ms)
- **Max Retries**: Number of retry attempts for failed commands

#### Shortcuts Settings

Configure keyboard shortcuts:

- **Increase Brightness**: Shortcut to increase brightness
- **Decrease Brightness**: Shortcut to decrease brightness

### Configuration File

Configuration is stored in JSON format at:
```
~/.config/twinkle-linux/config.json
```

You can edit this file directly, but it's recommended to use the Settings dialog.

### Resetting Configuration

To reset to default settings:

1. Close the application
2. Remove the configuration file:
   ```bash
   rm ~/.config/twinkle-linux/config.json
   ```
3. Restart the application

## Advanced Features

### VCP Controls

Access additional monitor settings beyond brightness:

**Supported VCP Codes**

| VCP Code | Name | Description |
|----------|------|-------------|
| 0x10 | Brightness | Monitor brightness level |
| 0x12 | Contrast | Monitor contrast level |
| 0x14 | Color Temperature | Color temperature presets |
| 0x60 | Input Source | Select input source (HDMI, DisplayPort, etc.) |
| 0x62 | Audio Volume | Audio volume (for monitors with speakers) |

**Accessing VCP Controls**
1. Right-click the tray icon
2. Select "VCP Controls"
3. Choose the monitor and VCP code you want to adjust

### Command Line Usage

You can also control monitors from the command line using ddcutil directly:

```bash
# Get current brightness
ddcutil getvcp 0x10

# Set brightness to 50%
ddcutil setvcp 0x10 50

# Set contrast to 75%
ddcutil setvcp 0x12 75

# Detect monitors
ddcutil detect
```

### Log Files

Application logs are stored at:
```
~/.cache/twinkle-linux/twinkle-linux.log
```

Use these logs for troubleshooting. Enable debug logging with:
```bash
twinkle-linux --debug
```

## Troubleshooting

### Monitor Not Detected

**Symptoms**: Application starts but no monitors are shown

**Solutions**:

1. Verify ddcutil is installed:
   ```bash
   ddcutil --version
   ```

2. Check monitor detection with ddcutil:
   ```bash
   sudo ddcutil detect
   ```

3. Ensure DDC/CI is enabled in your monitor's OSD menu

4. Try a different cable (DisplayPort preferred over HDMI)

5. Check if your graphics driver supports DDC/CI

### Permission Denied Errors

**Symptoms**: "Permission denied" when trying to control monitors

**Solutions**:

1. Install udev rules:
   ```bash
   sudo ./scripts/install-udev-rules.sh
   ```

2. Add user to i2c group:
   ```bash
   sudo usermod -a -G i2c $USER
   ```

3. Log out and log back in

4. Verify permissions:
   ```bash
   groups $USER
   ls -l /dev/i2c-*
   ```

### Slow Response

**Symptoms**: Monitor controls are slow to respond

**Solutions**:

1. Increase response timeout in Settings → Behavior → Response Timeout
2. Increase max retries in Settings → Behavior → Max Retries
3. Some monitors are inherently slower - this is normal

### Brightness Changes Not Applied

**Symptoms**: Slider moves but brightness doesn't change

**Solutions**:

1. Check logs for errors:
   ```bash
   tail -f ~/.cache/twinkle-linux/twinkle-linux.log
   ```

2. Test with ddcutil directly:
   ```bash
   ddcutil setvcp 0x10 50
   ```

3. Verify monitor supports brightness control (0x10)

4. Try restarting the application

### Application Won't Start

**Symptoms**: Application crashes or won't launch

**Solutions**:

1. Run with debug logging:
   ```bash
   twinkle-linux --debug
   ```

2. Check for Python errors:
   ```bash
   python src/main.py
   ```

3. Verify all dependencies are installed:
   ```bash
   pip install -r requirements.txt
   ```

4. Try resetting configuration:
   ```bash
   rm ~/.config/twinkle-linux/config.json
   ```

## FAQ

### Q: Does Twinkle Linux work with laptop displays?

**A**: No, Twinkle Linux is designed for external monitors connected via DisplayPort or HDMI. Laptop displays typically don't support DDC/CI.

### Q: Can I control built-in monitor speakers?

**A**: Yes, if your monitor has speakers and supports the audio volume VCP code (0x62), you can control volume through the VCP Controls panel.

### Q: Why is my monitor not detected?

**A**: Several reasons:
- Monitor doesn't support DDC/CI
- DDC/CI is disabled in monitor's OSD menu
- Graphics driver doesn't support DDC/CI
- Cable doesn't support DDC/CI (use DisplayPort when possible)

### Q: Can I use Twinkle Linux with Wayland?

**A**: Twinkle Linux uses PyQt6 which has Wayland support. However, system tray integration may vary depending on your Wayland compositor.

### Q: How do I set different brightness for each monitor?

**A**: Twinkle Linux automatically maintains separate brightness settings for each monitor. Simply select the monitor you want to adjust and set its brightness.

### Q: Can I schedule brightness changes?

**A**: This feature is not yet implemented but is planned for a future release.

### Q: Does Twinkle Linux work with multiple graphics cards?

**A**: Yes, Twinkle Linux should work with multiple graphics cards as long as they support DDC/CI.

### Q: Can I control monitors over the network?

**A**: No, Twinkle Linux only controls monitors directly connected to your system.

### Q: What's the difference between system-wide and user installation?

**A**: System-wide installation makes Twinkle Linux available to all users, while user installation only installs it for the current user. User installation doesn't require sudo (except for udev rules).

### Q: How do I uninstall Twinkle Linux?

**A**: Run the uninstall script:
```bash
sudo ./scripts/uninstall.sh  # For system-wide
./scripts/uninstall-user.sh  # For user-level
```

### Q: Can I contribute to Twinkle Linux?

**A**: Absolutely! See [CONTRIBUTING.md](../CONTRIBUTING.md) for guidelines on how to contribute.

## Getting Help

If you need additional help:

- **Documentation**: Check the [docs/](../docs/) directory for more information
- **GitHub Issues**: Report bugs and ask questions at [github.com/twinkle-linux/twinkle-linux/issues](https://github.com/twinkle-linux/twinkle-linux/issues)
- **GitHub Discussions**: Join the community discussions at [github.com/twinkle-linux/twinkle-linux/discussions](https://github.com/twinkle-linux/twinkle-linux/discussions)

## License

Twinkle Linux is licensed under the MIT License. See [LICENSE](../LICENSE) for details.
