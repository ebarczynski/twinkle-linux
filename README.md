# Twinkle Linux

A modern, minimalistic Linux monitor brightness control app with system tray integration.

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Rust](https://img.shields.io/badge/rust-1.80%2B-orange.svg)
![Platform](https://img.shields.io/badge/platform-Linux-lightgrey.svg)

## Overview

Twinkle Linux provides a clean, dark-themed UI for controlling monitor brightness on Linux. It sits in your system tray and offers per-monitor sliders, quick presets, and automatic detection of both external (DDC/CI) and internal (backlight) displays.

### Features

- **System Tray Integration** — Left-click the tray icon to open brightness control, right-click for presets and menu
- **Per-Monitor Control** — Each detected display gets its own brightness slider card
- **Quick Presets** — Set all monitors to 25%/50%/75%/100% from the tray menu
- **Internal Display Support** — Controls laptop panels via systemd-logind D-Bus (Intel, AMD, NVIDIA)
- **External Monitor Support** — DDC/CI protocol over I2C (HDMI, DisplayPort, USB-C)
- **Modern Dark UI** — Card-based layout with smooth sliders, inspired by Material Design
- **Debounced Sliders** — 300ms debounce prevents I2C flooding when dragging
- **No Root Required** — Uses D-Bus for backlight, udev rules for DDC/CI

### Supported Hardware

#### External Monitors (DDC/CI)

Any DDC/CI-compliant monitor connected via HDMI, DisplayPort, or USB-C, including:
- Lenovo (T24i, ThinkVision series)
- Dell (U-series, P-series, S-series)
- LG (UltraFine, UltraGear)
- HP, ASUS, BenQ, Samsung, Acer, and most modern monitors

#### Internal Laptop Displays

Twinkle Linux supports laptop internal panels through the Linux kernel backlight subsystem:

| GPU / APU | Backlight Interface | Driver |
|---|---|---|
| Intel (integrated, 6th–14th Gen) | `/sys/class/backlight/intel_backlight/` | `i915` |
| AMD (Ryzen, Ryzen AI, Ryzen AI Max 395+) | `/sys/class/backlight/amdgpu_bl0/` | `amdgpu` |
| NVIDIA (proprietary driver) | `/sys/class/backlight/nvidia_0/` or `acpi_video0` | `nvidia` |
| Generic (ACPI) | `/sys/class/backlight/acpi_video0/` | `acpi` |

Brightness control for internal displays uses `org.freedesktop.login1.Session.SetBrightness` via D-Bus, so no special group membership or udev rules are needed — it works as long as you have an active login session.

## Requirements

- Linux with a desktop environment (GNOME, KDE, XFCE, etc.)
- `ddcutil` CLI tool (for external monitor DDC/CI)
- GTK4 runtime libraries
- Rust 1.80+ (for building)

### Install ddcutil

```bash
# Ubuntu / Debian
sudo apt install ddcutil

# Fedora
sudo dnf install ddcutil

# Arch Linux
sudo pacman -S ddcutil
```

### I2C Permissions (for DDC/CI)

```bash
# Add your user to the i2c group
sudo usermod -aG i2c $USER

# Log out and back in for the group change to take effect
```

## Building

```bash
git clone https://github.com/ebarczynski/twinkle-linux.git
cd twinkle-linux/twinkle-rust
cargo build --release
```

The binary will be at `target/release/twinkle-linux`.

## Usage

### Running

```bash
# From the build directory
./target/release/twinkle-linux

# Or install to ~/.local/bin
cp target/release/twinkle-linux ~/.local/bin/
twinkle-linux
```

The app starts as a system tray icon. No window opens by default.

### Controls

| Action | Effect |
|---|---|
| Left-click tray icon | Open brightness control window |
| Right-click tray icon | Open context menu with presets |
| Drag slider | Adjust brightness (debounced 300ms) |
| "All Monitors" toggle | Link all sliders to move together |
| Preset (25/50/75/100%) | Set all monitors to that level |

### Command Line

```
twinkle-linux [OPTIONS]

Options:
  -v, --verbose    Enable verbose logging
  --debug          Enable debug logging
  --help           Show help
```

## Architecture

```
twinkle-rust/
├── src/
│   ├── main.rs              # App entry, GTK4 application + tokio runtime
│   ├── ddc/
│   │   ├── mod.rs           # DDC module exports
│   │   ├── command.rs       # ddcutil CLI wrapper (subprocess)
│   │   ├── ddc_manager.rs   # DDC operations + backlight via D-Bus
│   │   └── monitor.rs       # Monitor detection + backlight discovery
│   ├── ui/
│   │   ├── brightness_popup.rs  # Main window: card-based per-monitor sliders
│   │   ├── tray_icon.rs         # System tray via ksni (D-Bus StatusNotifierItem)
│   │   ├── style.css            # GTK4 CSS: dark theme, rounded cards
│   │   └── widgets/
│   │       ├── brightness_slider.rs  # Slider widget with debounce
│   │       └── settings_dialog.rs    # Settings dialog (DDC timeout, retries)
│   └── core/
│       └── config.rs        # Config persistence (JSON)
└── Cargo.toml
```

### Key Design Decisions

- **GTK4** for the UI — modern toolkit, Wayland-native, CSS theming
- **ksni** for system tray — pure Rust D-Bus StatusNotifierItem, no GTK3 dependency (GTK3+GTK4 in the same process causes a crash)
- **tokio** runtime entered once before `gtk::Application::run()` — GTK4 owns the main loop
- **Channel bridge** between ksni (async/tokio) and GTK4 (sync/GLib) via `mpsc` + `glib::timeout_add_local`
- **glib::spawn_future_local** for async DDC operations from GTK callbacks (avoids nested runtime panic)
- **systemd-logind D-Bus** for backlight control — works without root or group membership
- **`ddcutil --brief --sleep-multiplier 0.5`** for reliable DDC/CI with fast detection

### Monitor Detection Flow

1. `ddcutil detect --brief` — discovers I2C buses with monitors
2. `ddcutil capabilities --bus=N` — checks VCP 0x10 (brightness) support
   - Skips monitors where capabilities fail (internal panels, broken I2C)
3. `/sys/class/backlight/` scan — finds kernel backlight devices
   - Matches to display connectors via `/sys/class/backlight/*/device/drm/`
4. Duplicate elimination — if a panel appears in both DDC and backlight, only backlight is kept

### Brightness Control Flow

- **External (DDC/CI)**: `ddcutil --sleep-multiplier 0.5 setvcp --bus=N 0x10 <value>` (0-100)
- **Internal (backlight)**: `gdbus call ... org.freedesktop.login1.Session.SetBrightness backlight <name> <raw_value>`
  - Reads `max_brightness` from sysfs, converts percentage to raw value
  - Uses `session/auto` object path for automatic session detection

## Configuration

Config stored at `~/.config/twinkle-linux/config.json`:

```json
{
  "ddc_timeout": 0.5,
  "max_retries": 1,
  "sleep_multiplier": 0.5
}
```

| Key | Default | Description |
|---|---|---|
| `ddc_timeout` | 0.5 | Timeout in seconds for ddcutil commands (detection uses 5s override) |
| `max_retries` | 1 | Max retries for failed DDC commands |
| `sleep_multiplier` | 0.5 | DDC/CI I2C sleep multiplier for reliable communication |

## Autostart

To start Twinkle Linux automatically on login, create a desktop entry:

```bash
mkdir -p ~/.config/autostart/
cat > ~/.config/autostart/twinkle-linux.desktop << 'EOF'
[Desktop Entry]
Type=Application
Name=Twinkle Linux
Comment=Monitor brightness control
Exec=/path/to/twinkle-linux
Icon=display-brightness
Terminal=false
Categories=Utility;
EOF
```

## Troubleshooting

### No monitors detected
- Verify `ddcutil detect` works in a terminal
- Check I2C permissions: `ls -l /dev/i2c-*` (should show group `i2c`)
- Run `sudo modprobe i2c-dev` if the I2C module isn't loaded

### Internal display brightness not working
- Check if backlight device exists: `ls /sys/class/backlight/`
- Verify D-Bus access: `gdbus call --system --dest org.freedesktop.login1 --object-path /org/freedesktop/login1/session/auto --method org.freedesktop.login1.Session.SetBrightness backlight intel_backlight 50`
- The `type` file in the backlight directory indicates the interface: `firmware` (native GPU), `platform` (ACPI), or `raw`

### Tray icon not appearing
- Ensure your desktop environment supports StatusNotifierItem (KDE, GNOME with extension, XFCE)
- On GNOME, install the "AppIndicator" extension
- Check D-Bus session: `qdbus org.freedesktop.StatusNotifierWatcher /StatusNotifierWatcher RegisteredStatusNotifierItems`

## License

MIT License. See [LICENSE](LICENSE) for details.
