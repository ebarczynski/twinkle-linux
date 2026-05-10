# Twinkle Linux (C++26)

Monitor brightness control for Linux — GTK4 system tray app with DDC/CI and
backlight support. The C++26 port of the proven Rust implementation.

## Features

- **DDC/CI Backend**: `ddcutil` subprocess wrapper with retry/backoff
- **Internal Backlight**: `/sys/class/backlight` + systemd-logind D-Bus
- **GTK4 System Tray**: StatusNotifierItem over D-Bus (sdbus-c++)
- **Card-Based UI**: Per-monitor slider cards, "All Monitors" override
- **300ms Debounce**: Smooth slider → monitor updates
- **6 Tray Presets**: 10% Night, 20% Dusk, 40% Cloudy, 60% Sunny, 80% Full Sun, 100% Max
- **Settings Dialog**: General, UI, Behavior, Advanced tabs
- **Dark Theme**: White-on-dark CSS, no subtle grays
- **JSON Config**: XDG-compliant (`~/.config/twinkle-linux/config.json`)

## C++26 Features

| Feature | Usage |
|---------|-------|
| `std::expected<T,E>` | Error handling (replaces custom Result) |
| `std::format` | Type-safe string formatting |
| `constexpr` tables | VCP code registry at compile time |
| Reflection (C++26) | Auto VCP enum→string, config serialization |
| `std::latch`/`std::barrier` | Async coordination |

See [DESIGN.md](DESIGN.md) for the full architecture with Mermaid diagrams.

## Requirements

- C++26 compiler: GCC 15+ or Clang 19+
- CMake 3.25+
- GTK4 development libraries
- fmt, nlohmann/json, sdbus-c++, GTest
- `ddcutil` installed and configured

### Installing Dependencies

```bash
# Ubuntu 24.10+
sudo apt install build-essential cmake libgtk-4-dev libfmt-dev \
    nlohmann-json3-dev libsdbus-c++-dev libgtest-dev ddcutil

# Fedora 41+
sudo dnf install gcc-c++ cmake gtk4-devel fmt-devel \
    nlohmann-json-devel sdbus-c++-devel gtest-devel ddcutil
```

## Building

```bash
mkdir build && cd build
cmake .. -DCMAKE_BUILD_TYPE=Release
cmake --build . -j$(nproc)
```

## Running

```bash
./build/twinkle-linux
```

## Testing

```bash
cd build
ctest --output-on-failure
```

## Project Structure

```
twinkle-cpp/
├── CMakeLists.txt
├── DESIGN.md              ← Architecture + Mermaid diagrams
├── README.md
├── data/
│   └── style.css           ← GTK4 dark theme
├── include/twinkle/
│   ├── ddc/                ← DDC/CI backend
│   │   ├── error.hpp       ← std::expected-based errors
│   │   ├── vcp_codes.hpp   ← constexpr VCP registry
│   │   ├── command.hpp      ← ddcutil subprocess wrapper
│   │   ├── monitor.hpp      ← Monitor struct + detector
│   │   └── ddc_manager.hpp  ← High-level DDC API
│   ├── core/
│   │   ├── config.hpp       ← JSON config management
│   │   └── logger.hpp       ← fmt-based logger
│   └── ui/
│       ├── brightness_popup.hpp  ← Card-based popup
│       ├── tray_icon.hpp         ← SNI D-Bus tray
│       └── widgets/
│           ├── brightness_slider.hpp  ← Debounced slider
│           └── settings_dialog.hpp    ← 4-tab settings
├── src/                     ← mirrors include/
├── tests/
│   ├── test_ddc.cpp
│   ├── test_config.cpp
│   ├── test_monitor.cpp
│   ├── test_vcp_codes.cpp
│   └── test_utils.cpp
└── scripts/
```

## Design Principles

- **RAII**: All resources (GTK widgets, file handles, subprocess) managed via destructors
- **`std::expected`**: No exceptions except for fatal errors; all fallible ops return expected
- **`[[nodiscard]]`**: Important return values must be checked
- **`noexcept`**: GTK signal handlers are noexcept
- **`constexpr`**: VCP tables, config defaults — zero runtime cost
- **Strong types**: `enum class` for VCP codes, monitor types, errors

## Code Style

- Follow C++ Core Guidelines
- Use `auto` when type is obvious from context
- Prefer `std::make_unique` over `new`
- Use `std::string_view` for string parameters
- Use `enum class` for type-safe enumerations
- Use `std::format` over `printf`/`snprintf`

## License

MIT License — see LICENSE file in parent directory.

## Acknowledgments

- Inspired by BetterDisplay on macOS
- DDC/CI via ddcutil
- Architecture proven in twinkle-rust
- C++ best practices from Jason Turner's C++ Weekly
