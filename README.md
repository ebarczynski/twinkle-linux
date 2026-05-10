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
- **Per-monitor sliders** — each detected display gets its own brightness card
  in a dark, card-based GTK4 popup. An "All Monitors" master slider at the top
  adjusts everything at once.
- **Quick presets** — right-click the tray icon for one-tap presets:
  Night (10%), Dusk (20%), Cloudy (40%), Sunny (60%), Full Sun (80%), Max (100%).
- **No root** — backlight is managed through systemd-logind D-Bus; external
  monitors use the `ddcutil` CLI. No `sudo`, no `udev` rules, no group
  membership.

## Features

| Feature | Details |
|---|---|
| System tray | StatusNotifierItem via ksni (pure Rust) |
| Per-monitor sliders | Card-based dark UI, one slider per display |
| "All Monitors" master | Always visible at the top of the popup |
| Quick presets | 6 presets from the tray context menu |
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
| Left-click tray icon | Open / close brightness popup |
| Drag a monitor slider | Adjust that monitor's brightness (debounced) |
| Drag "All Monitors" slider | Adjust all monitors simultaneously |
| Right-click tray icon | Open context menu with quick presets |
| Select a preset (e.g. "Night 10%") | Set all monitors to that brightness level |
| Click "Quit" in tray menu | Exit the application |

## Architecture

```
twinkle-rust/src/
├── main.rs              GTK4 Application + tokio runtime entry point
├── ddc/                 DDC/CI subsystem
│   ├── mod.rs           DDC manager, monitor detection
│   ├── wrapper.rs       ddcutil CLI wrapper (--brief --sleep-multiplier 0.5)
│   └── vcp.rs           VCP code definitions (0x10 Brightness)
├── ui/                  GTK4 user interface
│   ├── mod.rs           UI module root
│   ├── brightness.rs    Brightness popup (card layout, per-monitor sliders)
│   ├── tray.rs          System tray icon (ksni StatusNotifierItem)
│   ├── style.css        Dark theme stylesheet
│   └── settings.rs      Settings dialog
└── core/                Core logic
    ├── mod.rs           Core module root
    └── config.rs        Config persistence (JSON via serde)
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

The file is created automatically on first run with sensible defaults. It
persists window state, last-known brightness values, and user preferences.

Example:

```json
{
  "brightness": {
    "DP-1": 80,
    "HDMI-A-1": 60
  }
}
```

## Autostart

To launch Twinkle Linux automatically on login, copy the included `.desktop`
file into your XDG autostart directory:

```bash
cp twinkle-rust/data/twinkle-linux.desktop ~/.config/autostart/
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

---

Built by [Edwin Barczynski](https://github.com/ebarczynski) and contributors.
