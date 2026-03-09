#pragma once

#include "twinkle/ddc/error.hpp"
#include "twinkle/ddc/vcp_codes.hpp"
#include <chrono>
#include <cstdint>
#include <expected>
#include <string>

namespace twinkle::ddc {

/// Configuration for command execution
struct CommandConfig {
    std::chrono::milliseconds timeout{5000}; ///< Command timeout (default: 5s)
    int max_retries{3};                     ///< Maximum retry attempts
    bool verbose{false};                     ///< Enable verbose output
};

/// Command executor for ddcutil subprocess calls
class CommandExecutor {
public:
    explicit CommandExecutor(CommandConfig config = {});
    ~CommandExecutor() = default;

    // Non-copyable but movable
    CommandExecutor(const CommandExecutor&) = delete;
    CommandExecutor& operator=(const CommandExecutor&) = delete;
    CommandExecutor(CommandExecutor&&) noexcept = default;
    CommandExecutor& operator=(CommandExecutor&&) noexcept = default;

    /// Check if ddcutil is available
    [[nodiscard]] bool is_available() const noexcept;

    /// Get VCP value
    [[nodiscard]] Result<uint8_t> get_vcp(uint8_t bus, uint16_t code) const;

    /// Set VCP value
    [[nodiscard]] Result<void> set_vcp(uint8_t bus, uint16_t code, uint8_t value) const;

    /// Detect all monitors
    [[nodiscard]] Result<std::string> detect_monitors() const;

    /// Query VCP capabilities
    [[nodiscard]] Result<std::string> query_capabilities(uint8_t bus) const;

    /// Get VCP info
    [[nodiscard]] Result<std::string> vcp_info(uint8_t bus, uint16_t code) const;

    /// Set command configuration
    void set_config(CommandConfig config) noexcept { config_ = std::move(config); }

    /// Get current configuration
    [[nodiscard]] const CommandConfig& config() const noexcept { return config_; }

private:
    /// Execute ddcutil command
    [[nodiscard]] Result<std::string> execute_command(std::string_view command) const;

    /// Parse VCP value from ddcutil output
    [[nodiscard]] Result<uint8_t> parse_vcp_value(std::string_view output) const;

    /// Retry logic with exponential backoff
    template<typename F>
    [[nodiscard]] auto retry(F&& func, std::string_view operation) const
        -> decltype(func());

    CommandConfig config_;
};

} // namespace twinkle::ddc
