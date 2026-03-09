#include "twinkle/ddc/command_executor.hpp"
#include "twinkle/ddc/error.hpp"
#include <chrono>
#include <cstdio>
#include <memory>
#include <sstream>
#include <thread>
#include <unistd.h>

namespace twinkle::ddc {

CommandExecutor::CommandExecutor(CommandConfig config)
    : config_(std::move(config)) {}

bool CommandExecutor::is_available() const noexcept {
    // Check if ddcutil is available by trying to get version
    FILE* pipe = popen("ddcutil --version 2>/dev/null", "r");
    if (!pipe) {
        return false;
    }
    pclose(pipe);
    return true;
}

Result<uint8_t> CommandExecutor::get_vcp(uint8_t bus, uint16_t code) const {
    std::stringstream cmd;
    cmd << "ddcutil getvcp " << std::hex << static_cast<int>(code)
        << " --bus " << static_cast<int>(bus);

    auto result = execute_command(cmd.str());
    if (!result.has_value()) {
        return Result<uint8_t>(result.error());
    }

    return parse_vcp_value(result.value());
}

Result<void> CommandExecutor::set_vcp(uint8_t bus, uint16_t code, uint8_t value) const {
    std::stringstream cmd;
    cmd << "ddcutil setvcp " << std::hex << static_cast<int>(code)
        << " " << static_cast<int>(value)
        << " --bus " << static_cast<int>(bus);

    auto result = execute_command(cmd.str());
    if (!result.has_value()) {
        return Result<void>(result.error());
    }

    return Result<void>();
}

Result<std::string> CommandExecutor::detect_monitors() const {
    return execute_command("ddcutil detect");
}

Result<std::string> CommandExecutor::query_capabilities(uint8_t bus) const {
    std::stringstream cmd;
    cmd << "ddcutil capabilities --bus " << static_cast<int>(bus);
    return execute_command(cmd.str());
}

Result<std::string> CommandExecutor::vcp_info(uint8_t bus, uint16_t code) const {
    std::stringstream cmd;
    cmd << "ddcutil vcpinfo " << std::hex << static_cast<int>(code)
        << " --bus " << static_cast<int>(bus);
    return execute_command(cmd.str());
}

Result<std::string> CommandExecutor::execute_command(std::string_view command) const {
    return retry([this, command]() -> Result<std::string> {
        std::unique_ptr<FILE, decltype(&pclose)> pipe(popen(command.data(), "r"), pclose);
        if (!pipe) {
            return Result<std::string>(DDCError::CommandFailed);
        }

        std::stringstream output;
        char buffer[128];
        while (fgets(buffer, sizeof(buffer), pipe.get()) != nullptr) {
            output << buffer;
        }

        int status = pclose(pipe.release());
        if (status != 0) {
            return Result<std::string>(DDCError::CommandFailed);
        }

        return Result<std::string>(output.str());
    }, command);
}

Result<uint8_t> CommandExecutor::parse_vcp_value(std::string_view output) const {
    // Parse ddcutil output to extract VCP value
    // Output format: "VCP code 0x10 (Brightness): current value = 50"
    size_t pos = output.find("current value =");
    if (pos == std::string_view::npos) {
        return Result<uint8_t>(DDCError::ParseError);
    }

    pos += 15; // Skip "current value ="
    while (pos < output.size() && (output[pos] == ' ' || output[pos] == '\t')) {
        ++pos;
    }

    std::string value_str;
    while (pos < output.size() && std::isdigit(output[pos])) {
        value_str += output[pos++];
    }

    try {
        int value = std::stoi(value_str);
        if (value < 0 || value > 255) {
            return Result<uint8_t>(DDCError::InvalidValue);
        }
        return Result<uint8_t>(static_cast<uint8_t>(value));
    } catch (...) {
        return Result<uint8_t>(DDCError::ParseError);
    }
}

template<typename F>
auto CommandExecutor::retry(F&& func, std::string_view operation) const
    -> decltype(func()) {
    int attempt = 0;
    while (attempt < config_.max_retries) {
        auto result = func();
        if (result.has_value()) {
            return result;
        }

        ++attempt;
        if (attempt < config_.max_retries) {
            // Exponential backoff: 100ms, 200ms, 400ms, etc.
            auto delay = std::chrono::milliseconds(100 * (1 << attempt));
            std::this_thread::sleep_for(delay);
        }
    }

    return func();
}

} // namespace twinkle::ddc
