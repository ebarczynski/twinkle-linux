cmake_minimum_required(VERSION 3.25)
project(TwinkleLinux
    VERSION 1.0.0
    DESCRIPTION "GUI application for controlling external monitor brightness via DDC/CI"
    LANGUAGES CXX
)

# C++23 standard
set(CMAKE_CXX_STANDARD 23)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

# Compiler flags for best practices
if(MSVC)
    add_compile_options(/W4 /permissive- /Zc:__cplusplus /EHsc)
else()
    add_compile_options(-Wall -Wextra -Wpedantic -Werror)
endif()

# Enable testing
enable_testing()

# Find required packages
find_package(PkgConfig REQUIRED)
find_package(Threads REQUIRED)
find_package(fmt CONFIG REQUIRED)

# GTK4
pkg_check_modules(GTK4 REQUIRED IMPORTED_TARGET gtk4)

# Include directories
include_directories(
    ${CMAKE_SOURCE_DIR}/include
    ${GTK4_INCLUDE_DIRS}
)

# Source files
set(SOURCES
    src/main.cpp
    src/ddc/command_executor.cpp
    src/ddc/ddc_manager.cpp
    src/ddc/monitor.cpp
    src/ui/tray_icon.cpp
    src/ui/brightness_popup.cpp
    src/ui/widgets/brightness_slider.cpp
    src/ui/widgets/settings_dialog.cpp
    src/core/config_manager.cpp
    core/logger.cpp
)

# Header files
set(HEADERS
    include/twinkle/ddc/command_executor.hpp
    include/twinkle/ddc/ddc_manager.hpp
    include/twinkle/ddc/monitor.hpp
    include/twinkle/ddc/vcp_codes.hpp
    include/twinkle/ddc/error.hpp
    include/twinkle/ui/tray_icon.hpp
    include/twinkle/ui/brightness_popup.hpp
    include/twinkle/ui/widgets/brightness_slider.hpp
    include/twinkle/ui/widgets/settings_dialog.hpp
    include/twinkle/core/config_manager.hpp
    include/twinkle/core/logger.hpp
)

# Create executable
add_executable(twinkle-linux ${SOURCES} ${HEADERS})

# Link libraries
target_link_libraries(twinkle-linux
    PRIVATE
    PkgConfig::GTK4
    fmt::fmt
    Threads::Threads
)

# Install rules
install(TARGETS twinkle-linux
    RUNTIME DESTINATION bin
)

# Tests
add_subdirectory(tests)
