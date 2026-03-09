#include "twinkle/core/config_manager.hpp"
#include <fstream>
#include <sstream>

#ifdef __linux__
#include <cstdlib>
#include <sys/stat.h>
#endif

namespace twinkle::core {

ConfigManager::ConfigManager() {
    config_path_ = get_config_dir() / "config.json";
}

std::filesystem::path ConfigManager::get_config_dir() {
#ifdef __linux__
    // Check XDG_CONFIG_HOME environment variable
    const char* xdg_config = std::getenv("XDG_CONFIG_HOME");
    if (xdg_config && xdg_config[0] != '\0') {
        return std::filesystem::path(xdg_config) / "twinkle-linux";
    }

    // Fallback to ~/.config
    const char* home = std::getenv("HOME");
    if (home && home[0] != '\0') {
        return std::filesystem::path(home) / ".config" / "twinkle-linux";
    }
#endif

    // Fallback to current directory
    return std::filesystem::current_path() / ".twinkle-linux";
}

AppConfig ConfigManager::create_default_config() {
    AppConfig config;
    config.ui.enabled_vcp_codes = {0x10, 0x12, 0x14, 0x60, 0x62};
    config.behavior.brightness_step_size = 1;
    config.behavior.remember_brightness = true;
    config.advanced.command_timeout_ms = 5000;
    config.advanced.max_retries = 3;
    return config;
}

Result<void> ConfigManager::load() {
    // Try to open config file
    std::ifstream file(config_path_);
    if (!file.is_open()) {
        // Config file doesn't exist, create default
        config_ = create_default_config();
        return Result<void>();
    }

    // Read file content
    std::stringstream buffer;
    buffer << file.rdbuf();
    std::string content = buffer.str();

    // Parse JSON (simplified - in production, use a proper JSON library)
    // For now, just use default config
    config_ = create_default_config();

    return Result<void>();
}

Result<void> ConfigManager::save() const {
    // Ensure config directory exists
    std::filesystem::create_directories(config_path_.parent_path());

    // Open config file for writing
    std::ofstream file(config_path_);
    if (!file.is_open()) {
        return Result<void>("Failed to open config file for writing");
    }

    // Serialize config to JSON
    file << serialize();

    if (!file.good()) {
        return Result<void>("Failed to write config file");
    }

    return Result<void>();
}

void ConfigManager::reset_to_defaults() {
    config_ = create_default_config();
}

std::string ConfigManager::serialize() const {
    // Simplified JSON serialization
    // In production, use a proper JSON library like nlohmann/json
    std::stringstream ss;
    ss << "{\n";
    ss << "  \"ui\": {\n";
    ss << "    \"auto_hide_popup\": " << (config_.ui.auto_hide_popup ? "true" : "false") << ",\n";
    ss << "    \"auto_hide_delay_ms\": " << config_.ui.auto_hide_delay_ms << "\n";
    ss << "  },\n";
    ss << "  \"behavior\": {\n";
    ss << "    \"brightness_step_size\": " << config_.behavior.brightness_step_size << ",\n";
    ss << "    \"remember_brightness\": " << (config_.behavior.remember_brightness ? "true" : "false") << "\n";
    ss << "  },\n";
    ss << "  \"advanced\": {\n";
    ss << "    \"command_timeout_ms\": " << config_.advanced.command_timeout_ms << ",\n";
    ss << "    \"max_retries\": " << static_cast<int>(config_.advanced.max_retries) << "\n";
    ss << "  }\n";
    ss << "}\n";
    return ss.str();
}

Result<void> ConfigManager::deserialize(std::string_view json) {
    // Simplified JSON parsing
    // In production, use a proper JSON library like nlohmann/json
    // For now, just use default config
    config_ = create_default_config();
    return Result<void>();
}

} // namespace twinkle::core
