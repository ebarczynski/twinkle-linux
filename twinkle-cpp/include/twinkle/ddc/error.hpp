#pragma once
/// @file error.hpp
/// @brief DDC/CI error types using std::expected.

#include <cstdint>
#include <string>
#include <string_view>
#include <expected>

namespace twinkle::ddc {

/// DDC error codes.
enum class DDCError : uint8_t {
    NotAvailable,       ///< ddcutil not installed
    MonitorNotFound,    ///< No monitor at bus/serial
    VCPNotSupported,    ///< VCP code not supported
    PermissionDenied,   ///< No I2C permissions
    CommandFailed,      ///< Subprocess exit != 0
    Timeout,            ///< ddcutil timed out
    InvalidValue,       ///< VCP value out of range
    ParseError,         ///< Could not parse ddcutil output
    DbusError,          ///< D-Bus (systemd-logind) failure
    IoError,            ///< Filesystem / sysfs error
};

/// Convert DDCError to human-readable string.
[[nodiscard]] constexpr std::string_view to_string(DDCError e) noexcept {
    switch (e) {
        case DDCError::NotAvailable:    return "DDC/CI not available";
        case DDCError::MonitorNotFound: return "Monitor not found";
        case DDCError::VCPNotSupported: return "VCP code not supported";
        case DDCError::PermissionDenied:return "Permission denied";
        case DDCError::CommandFailed:   return "Command failed";
        case DDCError::Timeout:         return "Timeout";
        case DDCError::InvalidValue:    return "Invalid value";
        case DDCError::ParseError:      return "Parse error";
        case DDCError::DbusError:       return "D-Bus error";
        case DDCError::IoError:         return "I/O error";
    }
    return "Unknown error";
}

/// Command execution failure details.
struct CommandFailure {
    std::string command;
    int exit_code{0};
    std::string stderr_out;
    std::string stdout_out;
};

/// Convenience alias — every DDC operation returns this.
template<typename T>
using DDCResult = std::expected<T, DDCError>;

/// void specialization
using DDCVoid = std::expected<void, DDCError>;

/// Check if an error message indicates a permission problem.
[[nodiscard]] inline bool is_permission_error(std::string_view msg) noexcept {
    return msg.find("Permission") != std::string_view::npos ||
           msg.find("permission") != std::string_view::npos ||
           msg.find("EACCES") != std::string_view::npos ||
           msg.find("Access denied") != std::string_view::npos;
}

} // namespace twinkle::ddc
