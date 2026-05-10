/// @file command.cpp
/// @brief ddcutil subprocess wrapper implementation.

#include "twinkle/ddc/command.hpp"
#include "twinkle/core/logger.hpp"
#include <algorithm>
#include <array>
#include <chrono>
#include <cstdio>
#include <cstdlib>
#include <thread>

namespace twinkle::ddc {

CommandExecutor::CommandExecutor(CommandConfig config)
    : config_(std::move(config)) {}

bool CommandExecutor::check_available() {
    auto result = execute("--version");
    if (result && result->success) {
        ddcutil_path_ = "ddcutil";
        LOG_INFO("ddcutil found: {}", result->stdout_out);
        return true;
    }
    LOG_WARN("ddcutil not available");
    return false;
}

std::string CommandExecutor::resolve_path() const {
    if (!ddcutil_path_.empty()) return ddcutil_path_;
    return "ddcutil";
}

DDCResult<CommandResult> CommandExecutor::execute(std::string_view args) const {
    return execute_with_retry(args);
}

DDCResult<CommandResult> CommandExecutor::detect_monitors() const {
    return execute("detect");
}

DDCResult<CommandResult> CommandExecutor::get_vcp(int32_t bus, uint8_t code) const {
    auto args = std::format("getvcp {:02x} --bus {}", code, bus);
    return execute(args);
}

DDCResult<CommandResult> CommandExecutor::set_vcp(int32_t bus, uint8_t code, uint16_t value) const {
    auto args = std::format("setvcp {:02x} {} --bus {}", code, value, bus);
    return execute(args);
}

DDCResult<CommandResult> CommandExecutor::get_capabilities(int32_t bus) const {
    auto args = std::format("capabilities --bus {}", bus);
    return execute(args);
}

DDCResult<CommandResult> CommandExecutor::execute_with_retry(std::string_view args) const {
    CommandResult last_result;
    double delay = config_.retry_delay_secs;

    for (int attempt = 0; attempt <= config_.max_retries; ++attempt) {
        if (attempt > 0) {
            std::this_thread::sleep_for(
                std::chrono::duration<double>(delay));
            delay *= config_.retry_backoff;
        }

        // Build command string with timeout
        auto cmd = std::format("timeout {} {} {} 2>&1",
            config_.timeout.count() / 1000.0,
            resolve_path(), args);

        // Open subprocess pipe
        std::array<char, 4096> buffer;
        std::string output;
        auto pipe = popen(cmd.c_str(), "r");
        if (!pipe) {
            last_result = CommandResult{
                .success = false,
                .return_code = -1,
                .command = cmd,
            };
            continue;
        }

        while (auto len = std::fgets(buffer.data(), buffer.size(), pipe)) {
            output.append(buffer.data(), len);
        }

        int status = pclose(pipe);
        int exit_code = WEXITSTATUS(status);

        // timeout command returns 124 on timeout
        if (exit_code == 124) {
            last_result = CommandResult{
                .success = false,
                .return_code = exit_code,
                .stdout_out = output,
                .command = cmd,
            };
            if (attempt < config_.max_retries) continue;
            return std::unexpected(DDCError::Timeout);
        }

        // Try to separate stdout from stderr — popen merges them (2>&1)
        // but for our purposes the merged output is fine.
        CommandResult result{
            .success = (exit_code == 0),
            .return_code = exit_code,
            .stdout_out = output,
            .command = cmd,
        };

        if (result.success) {
            // Try to parse VCP value if this was a getvcp command
            if (auto val = parse_vcp_value(output)) {
                result.value = val;
            }
            return result;
        }

        last_result = result;
    }

    return std::unexpected(DDCError::CommandFailed);
}

std::optional<uint16_t> CommandExecutor::parse_vcp_value(std::string_view output) {
    // ddcutil getvcp output:
    // "VCP code 0x10 (Brightness): current value = 75, max value = 100"
    auto pos = output.find("current value = ");
    if (pos == std::string_view::npos) {
        // Brief format: "DDC/CI communication failed" or similar
        // Also try: "value = 75" in some versions
        pos = output.find("value = ");
        if (pos == std::string_view::npos) return std::nullopt;
        pos += 8; // skip "value = "
    } else {
        pos += 16; // skip "current value = "
    }

    // Parse number
    uint16_t val = 0;
    while (pos < output.size() && output[pos] >= '0' && output[pos] <= '9') {
        val = val * 10 + (output[pos] - '0');
        ++pos;
    }
    return val;
}

} // namespace twinkle::ddc
