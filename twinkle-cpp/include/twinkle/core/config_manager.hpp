#pragma once

#include <cstdint>
#include <filesystem>
#include <optional>
#include <string>
#include <vector>

/// Simple result type for operations that can fail
template<typename T>
class Result {
public:
    Result(T value) : value_(std::move(value)), has_value_(true) {}
    Result(std::string error) : error_(std::move(error)), has_value_(false) {}

    [[nodiscard]] bool has_value() const noexcept { return has_value_; }
    [[nodiscard]] explicit operator bool() const noexcept { return has_value_; }

    [[nodiscard]] const T& value() const& {
        if (!has_value_) throw std::runtime_error(error_);
        return value_;
    }

    [[nodiscard]] T& value() & {
        if (!has_value_) throw std::runtime_error(error_);
        return value_;
    }

    [[nodiscard]] T value() && {
        if (!has_value_) throw std::runtime_error(error_);
        return std::move(value_);
    }

    [[nodiscard]] const std::string& error() const noexcept { return error_; }

private:
    T value_;
    std::string error_;
    bool has_value_;
};

/// Specialization for void
template<>
class Result<void> {
public:
    Result() : has_value_(true) {}
    Result(std::string error) : error_(std::move(error)), has_value_(false) {}

    [[nodiscard]] bool has_value() const noexcept { return has_value_; }
    [[nodiscard]] explicit operator bool() const noexcept { return has_value_; }

    void value() const {
        if (!has_value_) throw std::runtime_error(error_);
    }

    [[nodiscard]] const std::string& error() const noexcept { return error_; }

private:
    std::string error_;
    bool has_value_;
};

namespace twinkle::core {

/// UI configuration
struct UIConfig {
    bool auto_hide_popup{true};
    uint32_t auto_hide_delay_ms{3000};
    bool show_monitor_selector{false};
    std::vector<uint16_t> enabled_vcp_codes{0x10, 0x12, 0x14, 0x60, 0x62};
};

/// Monitor configuration
struct MonitorConfig {
    std::string unique_id;
    uint8_t brightness_preset_low{25};
    uint8_t brightness_preset_medium{50};
    uint8_t brightness_preset_high{75};
};

/// Behavior configuration
struct BehaviorConfig {
    uint32_t brightness_step_size{1};
    bool remember_brightness{true};
    bool restore_brightness_on_startup{false};
    bool show_notifications{true};
};

/// Advanced configuration
struct AdvancedConfig {
    uint32_t command_timeout_ms{5000};
    uint8_t max_retries{3};
    bool debug_mode{false};
};

/// Application configuration
struct AppConfig {
    UIConfig ui;
    BehaviorConfig behavior;
    AdvancedConfig advanced;
    std::string default_monitor_id;
    std::vector<MonitorConfig> monitor_configs;
};

/// Configuration manager
class ConfigManager {
public:
    ConfigManager();
    ~ConfigManager() = default;

    // Non-copyable but movable
    ConfigManager(const ConfigManager&) = delete;
    ConfigManager& operator=(const ConfigManager&) = delete;
    ConfigManager(ConfigManager&&) noexcept = default;
    ConfigManager& operator=(ConfigManager&&) noexcept = default;

    /// Load configuration from file
    [[nodiscard]] Result<void> load();

    /// Save configuration to file
    [[nodiscard]] Result<void> save() const;

    /// Get application configuration
    [[nodiscard]] const AppConfig& config() const noexcept { return config_; }

    /// Get mutable application configuration
    [[nodiscard]] AppConfig& config() noexcept { return config_; }

    /// Reset to default configuration
    void reset_to_defaults();

    /// Get config file path
    [[nodiscard]] const std::filesystem::path& config_path() const noexcept {
        return config_path_;
    }

private:
    AppConfig config_;
    std::filesystem::path config_path_;

    /// Get default config directory (XDG compliant)
    [[nodiscard]] static std::filesystem::path get_config_dir();

    /// Create default configuration
    [[nodiscard]] static AppConfig create_default_config();

    /// Serialize configuration to JSON
    [[nodiscard]] std::string serialize() const;

    /// Deserialize configuration from JSON
    [[nodiscard]] Result<void> deserialize(std::string_view json);
};

} // namespace twinkle::core
