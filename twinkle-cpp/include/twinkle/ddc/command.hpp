#pragma once
/// @file command.hpp
/// @brief ddcutil subprocess wrapper with retry/backoff.

#include "twinkle/ddc/error.hpp"
#include <chrono>
#include <cstdint>
#include <string>
#include <expected>

namespace twinkle::ddc {

/// Result of a ddcutil command execution.
struct CommandResult {
    bool success{false};
    int return_code{0};
    std::string stdout_out;
    std::string stderr_out;
    std::string command;      ///< The command that was run
    std::optional<uint16_t> value; ///< Extracted VCP value (for getvcp)

    /// Formatted error message (includes both stdout and stderr).
    [[nodiscard]] std::string error_message() const {
        if (success) return {};
        auto msg = std::format("Command failed (exit {})", return_code);
        if (!stderr_out.empty()) msg += std::format(": {}", stderr_out);
        if (!stdout_out.empty()) msg += std::format(" | stdout: {}", stdout_out);
        return msg;
    }
};

/// Configuration for command execution.
struct CommandConfig {
    std::chrono::milliseconds timeout{3000};
    int max_retries{1};
    double retry_delay_secs{0.1};
    double retry_backoff{2.0};
};

/// Executes ddcutil commands with timeout, retry, and exponential backoff.
///
/// All methods are blocking (suitable for calling from a worker thread).
class CommandExecutor {
public:
    explicit CommandExecutor(CommandConfig config = {});
    ~CommandExecutor() = default;

    CommandExecutor(const CommandExecutor&) = delete;
    CommandExecutor& operator=(const CommandExecutor&) = delete;
    CommandExecutor(CommandExecutor&&) noexcept = default;
    CommandExecutor& operator=(CommandExecutor&&) noexcept = default;

    /// Check if ddcutil is installed and reachable.
    [[nodiscard]] bool check_available();

    /// Execute a raw ddcutil command with arguments.
    [[nodiscard]] DDCResult<CommandResult> execute(std::string_view args) const;

    /// Detect all monitors (ddcutil detect).
    [[nodiscard]] DDCResult<CommandResult> detect_monitors() const;

    /// Get a VCP value (ddcutil getvcp CODE --bus N).
    [[nodiscard]] DDCResult<CommandResult> get_vcp(int32_t bus, uint8_t code) const;

    /// Set a VCP value (ddcutil setvcp CODE VALUE --bus N).
    [[nodiscard]] DDCResult<CommandResult> set_vcp(int32_t bus, uint8_t code, uint16_t value) const;

    /// Query capabilities (ddcutil capabilities --bus N).
    [[nodiscard]] DDCResult<CommandResult> get_capabilities(int32_t bus) const;

    /// Parse a VCP value from getvcp output.
    /// Output format: "VCP code 0x10 (Brightness): current value = 75, max value = 100"
    [[nodiscard]] static std::optional<uint16_t> parse_vcp_value(std::string_view output);

    /// Get/set config.
    [[nodiscard]] const CommandConfig& config() const noexcept { return config_; }
    void set_config(CommandConfig cfg) noexcept { config_ = std::move(cfg); }

private:
    CommandConfig config_;
    std::string ddcutil_path_;

    /// Resolve the ddcutil binary path.
    [[nodiscard]] std::string resolve_path() const;

    /// Execute with retry logic.
    [[nodiscard]] DDCResult<CommandResult> execute_with_retry(std::string_view args) const;
};

} // namespace twinkle::ddc
