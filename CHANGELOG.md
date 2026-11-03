# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 2026-11-03

### Changed
- CI builds x86_64 only: the aarch64 cross-compile leg is removed.
  Cross-compiling GTK4 on GitHub runners proved fragile (multiarch
  version skew between the amd64 image snapshot and ports.ubuntu.com —
  apt required exact M-A:same version equality per suite, then dpkg
  unpack of `libpango1.0-dev:arm64` still failed). x86_64 Ubuntu 24.04
  is the supported target; the multiarch recipe is preserved in git
  history and project notes for anyone resurrecting ARM builds.

### Fixed (round 5 — CI)
- **All jobs**: runner pinned to `ubuntu-24.04` — the distro this app
  targets, so CI always builds on the same GTK/apt base users run
  (silences the `ubuntu-latest` → Ubuntu 26 migration notice). GitHub
  actions upgraded past their Node 20 deprecation (`checkout@v7`,
  `cache@v6`, `upload-artifact@v7`). Pushing `v*` tags now triggers CI,
  so release binaries are built from the tag itself.
- **Test job**: schedule tests were timezone-dependent —
  `sun_times_local` returned instants converted to the HOST machine's
  timezone, so tests pinned to CET passed locally (CET machine) and failed
  on UTC CI runners. The whole schedule API (`decide`, `sun_times_local`,
  `Transition`, `ScheduleDecision`) is now offset-explicit (`FixedOffset`
  built from the caller's offset seconds); the engine passes
  `Local::now().fixed_offset()`. Results are identical on any machine.
- **Test job**: GTK widget tests SIGSEGV'd the whole test binary at exit on
  headless runners (`gtk4::init()` on a display-less machine leaves
  partially-initialized GLib state → GLib-GIO-CRITICAL + SIGSEGV 11).
  They now require `DISPLAY`/`WAYLAND_DISPLAY` to be set before touching
  GTK; headless CI skips them cleanly.
- **Check & Lint job**: newer stable clippy lints (`clone_on_copy`,
  `useless_borrows_in_formatting`, `arc_with_non_send_sync`,
  `manual_checked_ops`, unused import, unnecessary cast) fixed so
  `cargo clippy -- -D warnings` passes on the CI toolchain.
- **Build job (aarch64)**: cross-compilation actually never worked —
  pkg-config refused to resolve GTK for aarch64 (no arm64 sysroot).
  The workflow now registers the arm64 multiarch tree from
  ports.ubuntu.com, installs `libgtk-4-dev:arm64` +
  `gcc-aarch64-linux-gnu`, and points pkg-config at the arm64 pkgconfig
  dir (`PKG_CONFIG_LIBDIR` + `PKG_CONFIG_ALLOW_CROSS`).
  Also `fail-fast: false` so one matrix leg failing no longer cancels
  the other.

### Changed (round 4 — tray value list is back)
- The tray menu again carries a brightness value LIST, now generated from
  the config (`ui.preset_values` + `preset_labels`) — the same values the
  sliders in the details panel snap to. Selecting a value sets all
  monitors (clamped by `min_brightness`, suspends the day/night schedule
  for the current phase like slider drags). "Reload config" refreshes the
  list. The round-3 borderless quick slider was removed — on real GNOME
  sessions the panel did not appear on tray click; the config-driven list
  is the pragmatic replacement.

### Added (round 3 — quick slider)
- Quick all-monitors slider opens directly on tray icon click — no more
  indirection to a separate panel. Borderless, shows the live percentage,
  snaps to configured values on release, auto-closes on click-away/Escape/
  10 s idle. The full per-monitor panel moved to
  "Brightness details (per monitor)" in the menu. The shell tray menu
  itself cannot host a real slider (dbusmenu items are text-only), so this
  borderless panel is the closest native equivalent.

### Fixed (round 2 — on-device feedback)
- Tray menu rewritten to be shell-safe: PLAIN standard items only with a
  constant structure. The previous round's checkmark item, disabled info row
  and dynamic row count are the classic triggers for GNOME AppIndicator
  extension rendering an EMPTY menu ("box with invisible fonts") and dead
  clicks. Schedule state now lives in the item label
  (`Schedule: ON — Day 90% — until 16:26 Evening (Wrocław, Poland)`).
- Schedule engine pushes tray updates ONLY when the status text changes
  (was: every 30 s tick — layout churn destabilizes shell menus).
- Missing tray host (no StatusNotifierWatcher) no longer panics the app;
  it logs and continues. `TWINKLE_AUTOSHOW=1` opens the popup at startup —
  useful on tray-less systems and for debugging.
- The popup's settings gear button actually opens the settings dialog now
  (it previously only logged the config path).
- Dark theme detection fallback: when the GNOME color-scheme key is absent
  (XFCE/KDE/MATE), a `gtk-theme-name` ending in "-dark" (e.g. "Yaru-dark")
  now also enables the dark preference for native widgets.
- Monitor deduplication prefers the candidate with the richest capability
  set. Monitors behind docks/MST hubs expose the same serial on several
  I2C buses with broken DDC on some of them; blind last-wins could shadow
  a working bus with a dead one.

### Added (round 1)
- Day/night brightness schedule: brightness cycles automatically between
  configured night/day/evening levels based on computed sunrise/sunset times
  for a configured location (default: Wrocław, Poland). Pure offline math
  (sunrise equation) — no network needed. Validated against
  sunrise-sunset.org references within ±2 minutes.
- Smooth ramps: each schedule transition interpolates over
  `schedule.ramp_minutes` instead of jumping.
- Tray menu: "Day/night schedule" toggle (checkmark) with live status line
  (e.g. `Day 90% — until 16:26 Evening (Wrocław, Poland)`).
- Tray menu: "Reload config" — re-reads `config.json` without restarting.
- Slider snapping: sliders snap to the nearest `ui.preset_values` entry on
  release and show tick marks at the configured values, with the live
  percentage label while dragging.
- Manual override: dragging a slider suspends the schedule for the current
  phase; it resumes automatically at the next sunrise/sunset transition.
- Restore last brightness on startup (existing `restore_brightness` setting
  is now actually implemented) and persist per-monitor brightness on change
  (`remember_brightness`).
- Minimum brightness setting in the Behavior tab of the settings dialog.
- Author/contact information: About dialog, tray tooltip, Cargo metadata and
  README now carry the GitHub handle/repository and email.

### Changed
- Tray menu no longer shows a hardcoded selectable list of brightness
  values — the slider popup (with config-driven snapping) is the brightness
  control.
- Config loading is now forward/backward compatible: missing sections fall
  back to defaults (`#[serde(default)]`), unknown fields are ignored —
  new config sections never break existing setups.
- Theme setting is applied at startup and on settings save: native GTK
  widgets (dialog buttons, spinbuttons, comboboxes) follow dark mode via
  `gtk-application-prefer-dark-theme` and the GNOME `color-scheme` setting.

### Fixed
- Unreadable button labels on dark themes: theme gradients painted over the
  app's button background (`background-color` alone cannot override themed
  `background-image`). All button rules now clear `background-image`, and
  explicit label/headerbar colors are set.
- Fragile DOM walk to find per-card value labels removed; card entries keep
  direct references instead.
- `glib::SourceId` handled via `as_raw()` instead of `transmute`.
- Headless-safe software filter test (was failing on machines without xrandr).

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
