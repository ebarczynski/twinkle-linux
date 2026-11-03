# Twinkle Linux

[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Rust 1.80+](https://img.shields.io/badge/rust-1.80%2B-orange.svg)](https://www.rust-lang.org/)
[![Platform: Linux](https://img.shields.io/badge/platform-Linux-informational.svg)](https://www.kernel.org/)

A Linux monitor brightness control application with system tray integration.
Built with Rust, GTK4, and D-Bus — no root required.

Twinkle Linux lets you adjust the brightness of every connected display from a
single, lightweight system tray popup. External monitors are controlled via
DDC/CI and internal laptop panels via systemd-logind, so it works across a wide
range of hardware without elevated privileges.

---

## Overview

- **System tray icon** — lives in your panel via the StatusNotifierItem protocol
  (ksni, pure Rust D-Bus). Click to open the brightness popup.
- **Slider with snap points** — the popup gives you an "All Monitors" master
  slider plus a per-monitor slider. Both show the live percentage while you
  drag and snap to the brightness values configured in `ui.preset_values`
  (rendered as tick marks on the scale).
- **Day/night schedule** — brightness cycles automatically between night,
  day, and evening levels based on computed sunrise/sunset times for your
  location (default: Wrocław, Poland). Toggle it from the tray menu.
- **No root** — backlight is managed through systemd-logind D-Bus; external
  monitors use the `ddcutil` CLI. No `sudo`, no `udev` rules, no group
  membership.

## Features

| Feature | Details |
|---|---|
| System tray | StatusNotifierItem via ksni (pure Rust) |
| Config-driven value list | Tray menu presets = `ui.preset_values`, set all monitors on click |
| Brightness details panel | Card-based dark UI, per-monitor sliders, live %, snapping |
| Config-driven snap points | `ui.preset_values` → tick marks + snap-on-release |
| Day/night schedule | Sunrise/sunset cycling (NOAA sunrise equation, offline) |
| Smooth ramps | Transitions interpolate over `schedule.ramp_minutes` |
| Manual override | Sliders and menu presets suspend the schedule until the next phase |
| DDC/CI (external) | HDMI, DisplayPort, USB-C monitors via `ddcutil` |
| Backlight (internal) | Laptop panels via systemd-logind D-Bus |
| Debounced I2C | 300 ms debounce to avoid flooding the DDC bus |
| GTK4 dark theme | CSS-styled, card-based popup window |
| Persistent config | JSON config at `~/.config/twinkle-linux/config.json` |
| Autostart | Ships a `.desktop` file for XDG autostart |

## Supported Hardware

| Driver / Subsystem | Connection | Method | Notes |
|---|---|---|---|
| Intel i915 | Internal (eDP) | systemd-logind D-Bus | Backlight sysfs via logind |
| AMD amdgpu | Internal (eDP) | systemd-logind D-Bus | Backlight sysfs via logind |
| NVIDIA proprietary | Internal (eDP) | systemd-logind D-Bus | Requires `NVreg_RegistryDwords=EnableBrightnessControl=1` or nvidia-backlight |
| ACPI generic | Internal (eDP) | systemd-logind D-Bus | Fallback path |
| DDC/CI (any GPU) | HDMI | `ddcutil` CLI | VCP code 0x10 (Brightness) |
| DDC/CI (any GPU) | DisplayPort | `ddcutil` CLI | VCP code 0x10 |
| DDC/CI (any GPU) | USB-C / USB-C DP Alt | `ddcutil` CLI | VCP code 0x10 |

> **Note:** USB-C docks and hubs that expose DDC/CI to downstream monitors are
> also supported, as long as `ddcutil detect` lists the display.

## Requirements

| Requirement | Version |
|---|---|
| Rust | 1.80 or newer |
| GTK4 development libraries | 4.x |
| `ddcutil` CLI | 2.x recommended (`sudo apt install ddcutil`) |
| Linux kernel | 6.x recommended (any version with DDC/CI support) |
| systemd-logind | For internal panel backlight (present on all modern distros) |
| D-Bus session bus | For system tray (StatusNotifierItem) |

### Install dependencies (Debian / Ubuntu)

```bash
sudo apt install libgtk-4-dev ddcutil
```

### Install dependencies (Fedora)

```bash
sudo dnf install gtk4-devel ddcutil
```

### Install dependencies (Arch Linux)

```bash
sudo pacman -S gtk4 ddcutil
```

## Building

```bash
cd twinkle-rust
cargo build --release
```

The binary is placed at `twinkle-rust/target/release/twinkle-linux`.

### Quick install (optional)

```bash
cp twinkle-rust/target/release/twinkle-linux ~/.local/bin/
```

## Usage

Launch the application:

```bash
twinkle-linux
```

A brightness icon will appear in your system tray.

### Controls

| Action | Effect |
|---|---|
| Left-click tray icon | Open the brightness panel |
| Tray → brightness value list | Set all monitors to a configured value (`ui.preset_values`, same values the sliders snap to) |
| Drag a slider | Adjust brightness; the label shows the live percentage |
| Release a slider | Snaps to the nearest configured value |
| Tray → "Brightness details (per monitor)" | Full panel with a slider per display |
| Tray → "Day/night schedule" | Toggle automatic sunrise/sunset cycling |
| Tray → "Reload config" | Re-read `config.json` without restarting (also refreshes the menu value list) |
| Tray → "Quit" | Exit the application |

## Day/Night Schedule

When enabled (on by default), Twinkle computes sunrise/sunset for your
location with the sunrise equation (pure math — works fully offline) and
cycles three configured brightness levels:

```
midnight ──── sunrise ──── sunset ──── sunset + evening_duration_min ── midnight
   Night          Day          Evening                Night
```

- Manual slider changes suspend the schedule for the current phase, so your
  choice sticks until the next transition.
- Each transition ramps linearly over `schedule.ramp_minutes`.
- The tray menu shows the live status, e.g.
  `Day 90% — until 16:24 Evening (Wrocław, Poland)`.

Configure in `~/.config/twinkle-linux/config.json` under `"schedule"`:
latitude/longitude (defaults: Wrocław 51.1079 / 17.0385), the three
brightness levels, evening duration, and ramp minutes.

To debug the schedule on your machine:

```bash
cargo run --example schedule_check
```

## Architecture

```
twinkle-rust/src/
├── main.rs              GTK4 Application + tokio runtime entry point
├── ddc/                 DDC/CI subsystem
│   ├── mod.rs           DDC manager re-exports
│   ├── ddc_manager.rs   Monitor cache, get/set brightness, VCP access
│   ├── command.rs       ddcutil CLI wrapper (--brief --sleep-multiplier 0.5)
│   ├── monitor.rs       Monitor detection + EDID parsing
│   ├── software_filter.rs  Gamma-based sub-minimum dimming (xrandr)
│   ├── error.rs         Error types
│   └── vcp_codes.rs     VCP code definitions (0x10 Brightness)
├── ui/                  GTK4 user interface
│   ├── mod.rs           Theme preference (system dark-mode detection)
│   ├── brightness_popup.rs  Brightness popup (cards, snapping sliders)
│   ├── tray_icon.rs     System tray icon (ksni StatusNotifierItem)
│   ├── style.css        Dark theme stylesheet
│   └── widgets/
│       └── settings_dialog.rs  Settings dialog (General/UI/Behavior/Advanced)
├── core/                Core logic
│   ├── mod.rs           Core module root
│   ├── config.rs        Config persistence (JSON via serde)
│   └── schedule.rs      Day/night schedule engine + sunrise equation
└── utils/mod.rs         Helpers (snap-to-preset, formatting)
```

### Channel bridge

ksni is async (tokio), but GTK4 is single-threaded. The bridge between them:

1. **ksni async handlers** send messages through an `mpsc` channel.
2. `glib::timeout_add_local` polls the channel on the GTK main thread and
   updates the UI.
3. For DDC/CI writes from slider callbacks, `glib::spawn_future_local` runs
   the async `ddcutil` invocation on the local tokio handle.

This avoids all cross-thread GTK access while keeping the UI responsive.

### Key design decisions

| Decision | Rationale |
|---|---|
| **ksni** over libappindicator | libappindicator links GTK3, which crashes inside a GTK4 process. ksni is pure Rust D-Bus. |
| **systemd-logind D-Bus** for backlight | No root, no `video` group, works on Intel/AMD/NVIDIA out of the box. |
| **`ddcutil` CLI** for DDC/CI | Battle-tested I2C reliability with `--brief --sleep-multiplier 0.5`. Avoids raw I2C edge cases. |
| **300 ms debounce** | Prevents I2C bus flooding when dragging sliders. |
| **`glib::spawn_future_local`** | Safely runs async DDC operations from GTK signal handlers without blocking the UI. |

## Configuration

Twinkle Linux stores its configuration at:

```
~/.config/twinkle-linux/config.json
```

The file is created automatically on first run with sensible defaults.
Missing sections fall back to defaults when loading, so old config files
keep working when new features ship — and unknown fields are ignored.

Example:

```json
{
  "general": {
    "autostart": false,
    "theme": "system",
    "language": "en_US"
  },
  "ui": {
    "preset_values": [10, 20, 40, 60, 80],
    "preset_labels": ["Night", "Dusk", "Cloudy", "Sunny", "Full Sun"]
  },
  "behavior": {
    "min_brightness": 1,
    "remember_brightness": true,
    "restore_brightness": true
  },
  "schedule": {
    "enabled": true,
    "latitude": 51.1079,
    "longitude": 17.0385,
    "location_name": "Wrocław, Poland",
    "day_brightness": 90,
    "evening_brightness": 45,
    "night_brightness": 15,
    "evening_duration_min": 60,
    "ramp_minutes": 30
  }
}
```

`ui.preset_values` drives the slider snap points and tick marks in the
popup. The `schedule` section drives the day/night cycling.

## Autostart

To launch Twinkle Linux automatically on login, copy the included `.desktop`
file into your XDG autostart directory:

```bash
cp packaging/twinkle-linux.desktop ~/.config/autostart/
```

Or, if you installed the binary to `~/.local/bin/`:

```bash
cat > ~/.config/autostart/twinkle-linux.desktop << 'EOF'
[Desktop Entry]
Type=Application
Name=Twinkle Linux
Comment=Monitor brightness control
Exec=/home/YOURUSER/.local/bin/twinkle-linux
Icon=video-display
Terminal=false
Categories=Utility;
EOF
```

## Troubleshooting

### DDC/CI monitors not detected

1. Verify `ddcutil detect` lists your monitor.
2. If not, try `sudo modprobe i2c-dev` and check again.
3. Some monitors need a few seconds after power-on; wait 30s and retry.
4. USB-C docks may not forward DDC/CI — check your dock's documentation.

### Internal laptop backlight not working

1. Confirm `cat /sys/class/backlight/*/brightness` shows a value.
2. Ensure your user session runs under systemd-logind (standard on modern
   distros).
3. NVIDIA users: you may need to set the kernel parameter
   `NVreg_RegistryDwords=EnableBrightnessControl=1` or install
   `nvidia-backlight` for a sysfs backlight node.

### Tray icon doesn't appear

- Your desktop environment must support the StatusNotifierItem (SNI) protocol
  (KDE Plasma, GNOME with AppIndicator extension, XFCE, Sway/swaybar, etc.).
- On GNOME, install the
  [AppIndicator Extension](https://extensions.gnome.org/extension/615/appindicator-support/).

### DDC commands are slow

- Twinkle uses `--sleep-multiplier 0.5` by default for a balance of speed and
  reliability. If you see errors, the I2C bus may be busy. Avoid running
  `ddcutil` from other tools simultaneously.

## Repository Structure

```
twinkle-linux/
├── README.md              This file
├── LICENSE                MIT license
├── twinkle-rust/          Primary Rust implementation (GTK4 + ksni)
│   ├── Cargo.toml
│   ├── src/               Application source
│   └── data/              .desktop file, icons
└── twinkle-cpp/           C++23/26 implementation (work in progress)
```

## License

This project is licensed under the [MIT License](LICENSE).

## Author & Contact

**Edwin Barczynski** ([@ebarczynski](https://github.com/ebarczynski))

- Repository: [github.com/ebarczynski/twinkle-linux](https://github.com/ebarczynski/twinkle-linux)
- Issues: [github.com/ebarczynski/twinkle-linux/issues](https://github.com/ebarczynski/twinkle-linux/issues)
- Email: barczynski.e@gmail.com

---
