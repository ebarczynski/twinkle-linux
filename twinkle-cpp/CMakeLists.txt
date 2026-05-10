cmake_minimum_required(VERSION 3.25)
project(TwinkleLinux
    VERSION 0.1.0
    DESCRIPTION "Monitor brightness control via DDC/CI with GTK4 system tray"
    LANGUAGES CXX
)

# C++26 — requires GCC 15+ or Clang 19+
set(CMAKE_CXX_STANDARD 26)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

# Strict warnings
if(MSVC)
    add_compile_options(/W4 /permissive- /Zc:__cplusplus /EHsc)
else()
    add_compile_options(-Wall -Wextra -Wpedantic -Werror -Wno-missing-field-initializers)
endif()

# Generate compile_commands.json for tooling
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

# ─── Dependencies ────────────────────────────────────────────
find_package(PkgConfig REQUIRED)
find_package(Threads REQUIRED)
find_package(fmt CONFIG REQUIRED)
find_package(nlohmann_json CONFIG REQUIRED)
find_package(GTest CONFIG REQUIRED)

# GTK4
pkg_check_modules(GTK4 REQUIRED IMPORTED_TARGET gtk4)

# sdbus-c++ for D-Bus (system tray + systemd-logind backlight)
pkg_check_modules(SDBUSCPP REQUIRED IMPORTED_TARGET sdbus-c++)

# ─── Library: twinkle-core (non-UI) ─────────────────────────
add_library(twinkle-core STATIC
    src/ddc/command.cpp
    src/ddc/ddc_manager.cpp
    src/ddc/monitor.cpp
    src/core/config.cpp
    src/core/logger.cpp
)

target_include_directories(twinkle-core PUBLIC
    ${CMAKE_SOURCE_DIR}/include
)

target_link_libraries(twinkle-core PUBLIC
    PkgConfig::GTK4
    PkgConfig::SDBUSCPP
    fmt::fmt
    nlohmann_json::nlohmann_json
    Threads::Threads
)

# ─── Executable ──────────────────────────────────────────────
add_executable(twinkle-linux
    src/main.cpp
    src/ui/brightness_popup.cpp
    src/ui/tray_icon.cpp
    src/ui/widgets/brightness_slider.cpp
    src/ui/widgets/settings_dialog.cpp
)

target_link_libraries(twinkle-linux PRIVATE twinkle-core)

# Install
install(TARGETS twinkle-linux RUNTIME DESTINATION bin)

# ─── Tests ───────────────────────────────────────────────────
enable_testing()
add_subdirectory(tests)
