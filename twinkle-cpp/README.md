# Twinkle Linux (C++23 Version)

A GUI application for controlling external monitor brightness via DDC/CI on Linux, written in modern C++23 following best practices from Jason Turner and the C++ community.

## Features

- **DDC/CI Backend**: Wrapper around ddcutil subprocess calls for monitor control
- **System Tray Integration**: GTK-based system tray icon with context menu
- **Brightness Control**: Brightness slider with real-time adjustment and debouncing
- **Multi-Monitor Support**: Monitor selector dropdown with per-monitor brightness tracking
- **Additional VCP Controls**: Contrast, Volume, Input Source, Color Temperature
- **Settings Dialog**: Comprehensive settings with General, Behavior, and Advanced tabs

## C++23 Best Practices

This implementation follows modern C++ best practices:

- **RAII**: All resources are managed with RAII principles
- **Smart Pointers**: `std::unique_ptr` for exclusive ownership
- **Move Semantics**: Non-copyable but movable types
- **Const Correctness**: All functions are properly marked `const` where appropriate
- **`[[nodiscard]]`**: Functions with important return values are marked
- **`noexcept`**: Functions that don't throw are marked `noexcept`
- **`std::expected`-style Result Type**: Custom Result type for error handling
- **Type Safety**: Strong types and enums for type safety
- **Zero-cost Abstractions**: Templates and constexpr for compile-time optimization

## Requirements

- Linux operating system
- C++23 compatible compiler (GCC 13+, Clang 16+)
- ddcutil installed and configured
- GTK3 development libraries
- fmt library for formatting

### Installing Dependencies

```bash
# Ubuntu/Debian
sudo apt install build-essential cmake libgtk-3-dev ddcutil pkg-config libfmt-dev

# Fedora
sudo dnf install gcc-c++ cmake gtk3-devel ddcutil pkg-config fmt-devel

# Arch Linux
sudo pacman -S base-devel cmake gtk3 ddcutil pkg-config fmt
```

## Building

```bash
mkdir build
cd build
cmake ..
make -j$(nproc)
```

## Running

```bash
./build/twinkle-linux
```

## Project Structure

```
twinkle-cpp/
├── include/
│   └── twinkle/
│       ├── ddc/              # DDC/CI backend
│       │   ├── error.hpp
│       │   ├── vcp_codes.hpp
│       │   ├── monitor.hpp
│       │   ├── command_executor.hpp
│       │   └── ddc_manager.hpp
│       ├── ui/               # UI components
│       │   ├── tray_icon.hpp
│       │   ├── brightness_popup.hpp
│       │   └── widgets/
│       │       ├── brightness_slider.hpp
│       │       └── settings_dialog.hpp
│       └── core/             # Core functionality
│           ├── logger.hpp
│           └── config_manager.hpp
├── src/
│   ├── main.cpp
│   ├── ddc/
│   ├── ui/
│   ├── ui/widgets/
│   └── core/
├── tests/
├── CMakeLists.txt
└── README.md
```

## Development

### Code Style

- Follow C++ Core Guidelines
- Use `auto` only when type is obvious
- Prefer `std::make_unique` over `new`
- Use `constexpr` for compile-time constants
- Use `enum class` for type-safe enums
- Use `std::string_view` for string parameters

### Building with Debug

```bash
cmake -DCMAKE_BUILD_TYPE=Debug ..
make -j$(nproc)
```

### Building with Release

```bash
cmake -DCMAKE_BUILD_TYPE=Release ..
make -j$(nproc)
```

## Testing

```bash
cd build
ctest --output-on-failure
```

## Monitor Compatibility

This application is designed to work with any DDC/CI-compliant monitor, including:

- **Lenovo** T24i (primary target)
- **Dell** monitors
- **LG** monitors
- Most other DDC/CI-capable displays

## License

MIT License - see LICENSE file in parent directory

## Contributing

Contributions are welcome! Please ensure your code follows the C++23 best practices outlined above.

## Acknowledgments

- Inspired by BetterDisplay on macOS
- DDC/CI communication via ddcutil
- C++ best practices from Jason Turner's C++ Weekly and C++ Now presentations
