#pragma once

#include <string>
#include <string_view>

namespace twinkle::ddc {

/// Simple result type for operations that can fail
template<typename T>
class Result {
public:
    Result(T value) : value_(std::move(value)), has_value_(true) {}
    Result(DDCError error) : error_(error), has_value_(false) {}

    [[nodiscard]] bool has_value() const noexcept { return has_value_; }
    [[nodiscard]] explicit operator bool() const noexcept { return has_value_; }

    [[nodiscard]] const T& value() const& {
        if (!has_value_) throw DDCException(error_);
        return value_;
    }

    [[nodiscard]] T& value() & {
        if (!has_value_) throw DDCException(error_);
        return value_;
    }

    [[nodiscard]] T value() && {
        if (!has_value_) throw DDCException(error_);
        return std::move(value_);
    }

    [[nodiscard]] DDCError error() const noexcept { return error_; }

private:
    T value_;
    DDCError error_;
    bool has_value_;
};

/// Specialization for void
template<>
class Result<void> {
public:
    Result() noexcept : error_(DDCError::NotAvailable), has_value_(true) {}
    explicit Result(DDCError error) noexcept : error_(error), has_value_(false) {}

    [[nodiscard]] bool has_value() const noexcept { return has_value_; }
    [[nodiscard]] explicit operator bool() const noexcept { return has_value_; }

    void value() const {
        if (!has_value_) throw DDCException(error_);
    }

    [[nodiscard]] DDCError error() const noexcept { return error_; }

private:
    DDCError error_;
    bool has_value_;
};

/// Error codes for DDC/CI operations
enum class DDCError {
    NotAvailable,        ///< ddcutil not installed or unavailable
    MonitorNotFound,      ///< Monitor not found by bus or serial
    VCPNotSupported,     ///< VCP code not supported by monitor
    PermissionDenied,     ///< Insufficient I2C device permissions
    CommandFailed,        ///< ddcutil command execution failed
    Timeout,             ///< DDC/CI operation timeout
    InvalidValue,         ///< Invalid VCP code value
    ParseError,          ///< Failed to parse ddcutil output
};

/// Get human-readable error message
[[nodiscard]] constexpr std::string_view to_string(DDCError error) noexcept {
    switch (error) {
        case DDCError::NotAvailable:   return "DDC/CI not available";
        case DDCError::MonitorNotFound: return "Monitor not found";
        case DDCError::VCPNotSupported:return "VCP code not supported";
        case DDCError::PermissionDenied:return "Permission denied";
        case DDCError::CommandFailed:   return "Command execution failed";
        case DDCError::Timeout:         return "Operation timeout";
        case DDCError::InvalidValue:    return "Invalid value";
        case DDCError::ParseError:      return "Parse error";
    }
    return "Unknown error";
}

/// Exception class for DDC/CI errors
class DDCException : public std::runtime_error {
public:
    explicit DDCException(DDCError error)
        : std::runtime_error(std::string(to_string(error))), error_(error) {}

    [[nodiscard]] DDCError error() const noexcept { return error_; }

private:
    DDCError error_;
};

} // namespace twinkle::ddc
