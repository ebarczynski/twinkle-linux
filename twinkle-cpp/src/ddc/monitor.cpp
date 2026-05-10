/// @file monitor.cpp
/// @brief Monitor detection and parsing.

#include "twinkle/ddc/monitor.hpp"
#include "twinkle/ddc/command.hpp"
#include "twinkle/core/logger.hpp"
#include <algorithm>
#include <cstring>
#include <filesystem>
#include <fstream>
#include <regex>
#include <sstream>

namespace twinkle::ddc {

DDCResult<std::vector<Monitor>> MonitorDetector::detect_monitors() {
    std::vector<Monitor> monitors;

    // 1. Internal backlight displays
    detect_internal_backlights(monitors);

    // 2. External DDC/CI monitors
    auto result = executor_.detect_monitors();
    if (!result) {
        LOG_WARN("ddcutil detect failed: {}", static_cast<int>(result.error()));
        // Return whatever we have (maybe just internal displays)
        return monitors;
    }

    if (!result->success) {
        LOG_WARN("ddcutil detect exited with code {}", result->return_code);
        return monitors;
    }

    auto external = parse_detect_output(result->stdout_out);
    if (!external) return std::unexpected(external.error());

    for (auto& m : *external) {
        monitors.push_back(std::move(m));
    }

    LOG_INFO("Detected {} monitors total", monitors.size());
    return monitors;
}

void MonitorDetector::detect_internal_backlights(std::vector<Monitor>& out) {
    namespace fs = std::filesystem;
    constexpr auto bl_dir = "/sys/class/backlight";

    if (!fs::exists(bl_dir)) {
        LOG_INFO("No /sys/class/backlight directory");
        return;
    }

    std::error_code ec;
    for (const auto& entry : fs::directory_iterator(bl_dir, ec)) {
        if (ec) continue;
        auto name = entry.path().filename().string();
        auto path = entry.path().string();

        auto brightness_file = path + "/brightness";
        auto max_file = path + "/max_brightness";

        if (!fs::exists(brightness_file)) continue;

        uint16_t max_brightness = 100;
        if (auto f = std::ifstream(max_file); f) {
            f >> max_brightness;
        }

        LOG_INFO("Found internal backlight: {} (max={})", name, max_brightness);

        auto mon = Monitor::internal(name, path);
        mon.capabilities.max_brightness = max_brightness;
        out.push_back(std::move(mon));
    }
}

DDCResult<std::vector<Monitor>> MonitorDetector::parse_detect_output(std::string_view output) {
    std::vector<Monitor> monitors;

    // Regex patterns for ddcutil detect output
    static const std::regex bus_re(R"(I2C bus:\s*/dev/i2c-(\d+))");
    static const std::regex model_re(R"(Model:\s*(.+))");
    static const std::regex serial_re(R"(Serial number:\s*(.+))");
    static const std::regex mfg_re(R"(Mfg id:\s*(.+))");
    static const std::regex monitor_brief_re(R"(Monitor:\s*(.+))");

    std::istringstream stream{std::string{output}};
    std::vector<std::string> lines;
    for (std::string line; std::getline(stream, line);) {
        lines.push_back(line);
    }

    for (size_t i = 0; i < lines.size(); ++i) {
        std::smatch bus_match;
        if (!std::regex_search(lines[i], bus_match, bus_re)) continue;

        int32_t bus = std::stoi(bus_match[1]);
        LOG_INFO("Found monitor on bus {}", bus);
        auto mon = Monitor::external(bus);

        // Look ahead for monitor info
        for (size_t j = i + 1; j < std::min(i + 20, lines.size()); ++j) {
            if (lines[j].starts_with("Display ")) break;

            std::smatch m;
            if (std::regex_search(lines[j], m, model_re) && m[1].str().size() > 0) {
                auto val = m[1].str();
                auto trimmed = val.substr(0, val.find_last_not_of(" \t\r\n") + 1);
                if (!trimmed.empty()) mon.model = trimmed;
            }
            if (std::regex_search(lines[j], m, serial_re) && m[1].str().size() > 0) {
                auto val = m[1].str();
                auto trimmed = val.substr(0, val.find_last_not_of(" \t\r\n") + 1);
                if (!trimmed.empty()) mon.serial = trimmed;
            }
            if (std::regex_search(lines[j], m, mfg_re) && m[1].str().size() > 0) {
                auto val = m[1].str();
                auto trimmed = val.substr(0, val.find_last_not_of(" \t\r\n") + 1);
                if (!trimmed.empty()) mon.manufacturer = trimmed;
            }

            // Brief format fallback
            if (mon.model == "Unknown Monitor") {
                if (std::regex_search(lines[j], m, monitor_brief_re)) {
                    auto val = m[1].str();
                    auto trimmed = val.substr(0, val.find_last_not_of(" \t\r\n") + 1);
                    if (!trimmed.empty()) {
                        if (auto sp = trimmed.find(' '); sp != std::string::npos) {
                            mon.manufacturer = trimmed.substr(0, sp);
                            mon.model = trimmed.substr(sp + 1);
                            while (!mon.model.empty() && mon.model.front() == ' ')
                                mon.model.erase(0, 1);
                        } else {
                            mon.model = trimmed;
                        }
                    }
                }
            }
        }

        // Query capabilities — skip monitor if this fails (internal panel)
        auto caps = get_capabilities(bus);
        if (!caps) {
            LOG_WARN("Skipping bus {}: capabilities query failed — likely internal panel", bus);
            continue;
        }
        mon.capabilities = std::move(*caps);

        LOG_INFO("Added monitor: {}", mon.display_name());
        monitors.push_back(std::move(mon));
    }

    return monitors;
}

DDCResult<MonitorCapabilities> MonitorDetector::get_capabilities(int32_t bus) {
    auto result = executor_.get_capabilities(bus);
    if (!result) return std::unexpected(result.error());
    if (!result->success) return std::unexpected(DDCError::CommandFailed);
    return parse_capabilities(result->stdout_out);
}

MonitorCapabilities MonitorDetector::parse_capabilities(std::string_view output) {
    MonitorCapabilities caps;

    // Parse "Feature: XX (Name)" entries
    static const std::regex vcp_re(R"(Feature:\s*([0-9A-Fa-f]{2))");
    std::string s{output};
    for (std::sregex_iterator it{s.begin(), s.end(), vcp_re}, end; it != end; ++it) {
        auto code = static_cast<uint8_t>(std::stoul((*it)[1].str(), nullptr, 16));
        caps.supported_vcp_codes.insert(code);
    }

    caps.supports_input_source = caps.supports_vcp(0x60);
    caps.supports_power_control = caps.supports_vcp(0xD6);
    caps.supports_audio = caps.supports_vcp(0x62) || caps.supports_vcp(0x8D);

    return caps;
}

} // namespace twinkle::ddc
