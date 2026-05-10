#pragma once
/// @file config.hpp
/// @brief Configuration management — JSON via nlohmann/json.
///
/// C++26 reflection would auto-generate to_json/from_json.
/// For now we use explicit serialization functions.

#include <cstdint>
#include <filesystem>
#include <string>
#include <vector>
#include <expected>

namespace twinkle::core {

/// General settings.
struct GeneralConfig {
    bool autostart{false};
    std::string theme{"system"};  ///< "system", "light", "dark"
    std::string language{"en_US"};
};

/// UI settings.
struct UIConfig {
    uint32_t auto_hide_delay_ms{3000};
    bool show_monitor_selector{false};
    bool enable_presets{true};
    std::vector<uint16_t> preset_values{10, 20, 40, 60, 80};
    std::vector<std::string> preset_labels{"Night", "Dusk", "Cloudy", "Sunny", "Full Sun"};
};

/// Behavior settings.
struct BehaviorConfig {
    uint32_t debounce_delay_ms{300};
    bool remember_brightness{true};
    bool restore_brightness{true};
    bool enable_notifications{true};
};

/// Per-monitor settings.
struct MonitorConfig {
    std::string unique_id;
    std::string display_name;
    uint16_t last_brightness{100};
};

/// Advanced settings.
struct AdvancedConfig {
    uint32_t command_timeout_ms{3000};
    uint32_t max_retries{1};
    bool debug_logging{false};
    std::string log_file_path;
};

/// Top-level application configuration.
struct AppConfig {
    GeneralConfig general;
    UIConfig ui;
    BehaviorConfig behavior;
    AdvancedConfig advanced;
    std::string default_monitor_id;
    std::vector<MonitorConfig> monitors;
};

/// Configuration error.
enum class ConfigError {
    IoError,
    ParseError,
    NotFound,
};

/// Config result type.
template<typename T>
using ConfigResult = std::expected<T, ConfigError>;

/// Manages loading/saving application configuration.
///
/// Uses XDG-compliant paths: ~/.config/twinkle-linux/config.json
class ConfigManager {
public:
    ConfigManager();
    ~ConfigManager() = default;

    ConfigManager(const ConfigManager&) = delete;
    ConfigManager& operator=(const ConfigManager&) = delete;

    /// Load config from disk (or use defaults).
    [[nodiscard]] ConfigResult<void> load();

    /// Save config to disk.
    [[nodiscard]] ConfigResult<void> save() const;

    /// Reset to defaults.
    void reset_to_defaults();

    /// Access the config.
    [[nodiscard]] const AppConfig& config() const noexcept { return config_; }
    [[nodiscard]] AppConfig& config_mut() noexcept { return config_; }

    /// Path to config file.
    [[nodiscard]] const std::filesystem::path& config_path() const noexcept { return config_path_; }

private:
    AppConfig config_;
    std::filesystem::path config_path_;

    [[nodiscard]] static std::filesystem::path get_config_dir();
    [[nodiscard]] static AppConfig create_defaults();
};

} // namespace twinkle::core
