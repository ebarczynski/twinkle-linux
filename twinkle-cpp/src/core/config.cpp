/// @file config.cpp
/// @brief Configuration management — JSON serialization.

#include "twinkle/core/config.hpp"
#include "twinkle/core/logger.hpp"
#include <nlohmann/json.hpp>
#include <filesystem>
#include <fstream>

namespace twinkle::core {

namespace fs = std::filesystem;
using json = nlohmann::json;

// ── JSON serialization ──────────────────────────────────────
// C++26 reflection would replace all of this with auto-generated code.

void to_json(json& j, const GeneralConfig& c) {
    j = json{{"autostart", c.autostart}, {"theme", c.theme}, {"language", c.language}};
}
void from_json(const json& j, GeneralConfig& c) {
    j.at("autostart").get_to(c.autostart);
    j.at("theme").get_to(c.theme);
    j.at("language").get_to(c.language);
}

void to_json(json& j, const UIConfig& c) {
    j = json{{"auto_hide_delay_ms", c.auto_hide_delay_ms},
             {"show_monitor_selector", c.show_monitor_selector},
             {"enable_presets", c.enable_presets},
             {"preset_values", c.preset_values},
             {"preset_labels", c.preset_labels}};
}
void from_json(const json& j, UIConfig& c) {
    j.at("auto_hide_delay_ms").get_to(c.auto_hide_delay_ms);
    j.at("show_monitor_selector").get_to(c.show_monitor_selector);
    j.at("enable_presets").get_to(c.enable_presets);
    j.at("preset_values").get_to(c.preset_values);
    j.at("preset_labels").get_to(c.preset_labels);
}

void to_json(json& j, const BehaviorConfig& c) {
    j = json{{"debounce_delay_ms", c.debounce_delay_ms},
             {"remember_brightness", c.remember_brightness},
             {"restore_brightness", c.restore_brightness},
             {"enable_notifications", c.enable_notifications}};
}
void from_json(const json& j, BehaviorConfig& c) {
    j.at("debounce_delay_ms").get_to(c.debounce_delay_ms);
    j.at("remember_brightness").get_to(c.remember_brightness);
    j.at("restore_brightness").get_to(c.restore_brightness);
    j.at("enable_notifications").get_to(c.enable_notifications);
}

void to_json(json& j, const MonitorConfig& c) {
    j = json{{"unique_id", c.unique_id}, {"display_name", c.display_name},
             {"last_brightness", c.last_brightness}};
}
void from_json(const json& j, MonitorConfig& c) {
    j.at("unique_id").get_to(c.unique_id);
    j.at("display_name").get_to(c.display_name);
    j.at("last_brightness").get_to(c.last_brightness);
}

void to_json(json& j, const AdvancedConfig& c) {
    j = json{{"command_timeout_ms", c.command_timeout_ms},
             {"max_retries", c.max_retries},
             {"debug_logging", c.debug_logging},
             {"log_file_path", c.log_file_path}};
}
void from_json(const json& j, AdvancedConfig& c) {
    j.at("command_timeout_ms").get_to(c.command_timeout_ms);
    j.at("max_retries").get_to(c.max_retries);
    j.at("debug_logging").get_to(c.debug_logging);
    j.at("log_file_path").get_to(c.log_file_path);
}

void to_json(json& j, const AppConfig& c) {
    j = json{{"general", c.general}, {"ui", c.ui}, {"behavior", c.behavior},
             {"advanced", c.advanced}, {"default_monitor_id", c.default_monitor_id},
             {"monitors", c.monitors}};
}
void from_json(const json& j, AppConfig& c) {
    if (j.contains("general")) j.at("general").get_to(c.general);
    if (j.contains("ui")) j.at("ui").get_to(c.ui);
    if (j.contains("behavior")) j.at("behavior").get_to(c.behavior);
    if (j.contains("advanced")) j.at("advanced").get_to(c.advanced);
    if (j.contains("default_monitor_id")) j.at("default_monitor_id").get_to(c.default_monitor_id);
    if (j.contains("monitors")) j.at("monitors").get_to(c.monitors);
}

// ── ConfigManager ───────────────────────────────────────────

ConfigManager::ConfigManager()
    : config_(create_defaults()),
      config_path_(get_config_dir() / "config.json") {}

fs::path ConfigManager::get_config_dir() {
    // XDG_CONFIG_HOME or ~/.config
    if (auto xdg = std::getenv("XDG_CONFIG_HOME"); xdg && xdg[0]) {
        return fs::path{xdg} / "twinkle-linux";
    }
    if (auto home = std::getenv("HOME"); home && home[0]) {
        return fs::path{home} / ".config" / "twinkle-linux";
    }
    return fs::path{"/tmp"} / "twinkle-linux";
}

AppConfig ConfigManager::create_defaults() {
    return AppConfig{};
}

ConfigResult<void> ConfigManager::load() {
    std::error_code ec;
    if (!fs::exists(config_path_, ec)) {
        LOG_INFO("No config file at {}, using defaults", config_path_.string());
        return {}; // defaults already set
    }

    try {
        auto f = std::ifstream(config_path_);
        if (!f) return std::unexpected(ConfigError::IoError);

        json j = json::parse(f);
        config_ = j.get<AppConfig>();
        LOG_INFO("Loaded config from {}", config_path_.string());
        return {};
    } catch (const json::parse_error& e) {
        LOG_ERROR("Config parse error: {}", e.what());
        return std::unexpected(ConfigError::ParseError);
    }
}

ConfigResult<void> ConfigManager::save() const {
    try {
        // Ensure directory exists
        auto dir = config_path_.parent_path();
        std::error_code ec;
        fs::create_directories(dir, ec);

        auto j = json(config_);
        auto f = std::ofstream(config_path_);
        if (!f) return std::unexpected(ConfigError::IoError);
        f << j.dump(2) << std::endl;
        LOG_INFO("Saved config to {}", config_path_.string());
        return {};
    } catch (const std::exception& e) {
        LOG_ERROR("Config save error: {}", e.what());
        return std::unexpected(ConfigError::IoError);
    }
}

void ConfigManager::reset_to_defaults() {
    config_ = create_defaults();
}

} // namespace twinkle::core
