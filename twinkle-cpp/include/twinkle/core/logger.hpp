#pragma once

#include <fmt/core.h>
#include <fstream>
#include <memory>
#include <mutex>
#include <string>
#include <string_view>

namespace twinkle::core {

/// Log level
enum class LogLevel : uint8_t {
    Trace = 0,
    Debug = 1,
    Info = 2,
    Warning = 3,
    Error = 4,
    Critical = 5
};

/// Logger class for thread-safe logging
class Logger {
public:
    /// Get singleton instance
    [[nodiscard]] static Logger& instance() noexcept {
        static Logger logger;
        return logger;
    }

    ~Logger();

    // Non-copyable and non-movable
    Logger(const Logger&) = delete;
    Logger& operator=(const Logger&) = delete;
    Logger(Logger&&) = delete;
    Logger& operator=(Logger&&) = delete;

    /// Initialize logger with file output
    void initialize(std::string_view log_file, LogLevel level = LogLevel::Info);

    /// Set log level
    void set_level(LogLevel level) noexcept { level_ = level; }

    /// Enable/disable console output
    void set_console_enabled(bool enabled) noexcept { console_enabled_ = enabled; }

    /// Log a message
    template<typename... Args>
    void log(LogLevel level, std::string_view format, Args&&... args) {
        if (level < level_) return;

        const auto message = fmt::format(format, std::forward<Args>(args)...);
        const auto timestamp = get_timestamp();

        std::lock_guard lock(mutex_);

        if (console_enabled_) {
            fmt::print(stderr, "[{}] [{}] {}\n", timestamp, level_to_string(level), message);
        }

        if (file_) {
            fmt::print(*file_, "[{}] [{}] {}\n", timestamp, level_to_string(level), message);
            file_->flush();
        }
    }

    /// Convenience logging functions
    template<typename... Args>
    void trace(std::string_view format, Args&&... args) {
        log(LogLevel::Trace, format, std::forward<Args>(args)...);
    }

    template<typename... Args>
    void debug(std::string_view format, Args&&... args) {
        log(LogLevel::Debug, format, std::forward<Args>(args)...);
    }

    template<typename... Args>
    void info(std::string_view format, Args&&... args) {
        log(LogLevel::Info, format, std::forward<Args>(args)...);
    }

    template<typename... Args>
    void warning(std::string_view format, Args&&... args) {
        log(LogLevel::Warning, format, std::forward<Args>(args)...);
    }

    template<typename... Args>
    void error(std::string_view format, Args&&... args) {
        log(LogLevel::Error, format, std::forward<Args>(args)...);
    }

    template<typename... Args>
    void critical(std::string_view format, Args&&... args) {
        log(LogLevel::Critical, format, std::forward<Args>(args)...);
    }

private:
    Logger() = default;

    /// Get current timestamp
    [[nodiscard]] static std::string get_timestamp();

    /// Convert log level to string
    [[nodiscard]] static std::string_view level_to_string(LogLevel level) noexcept;

    std::unique_ptr<std::ofstream> file_;
    LogLevel level_{LogLevel::Info};
    bool console_enabled_{true};
    std::mutex mutex_;
};

/// Convenience macros for logging
#define LOG_TRACE(...) ::twinkle::core::Logger::instance().trace(__VA_ARGS__)
#define LOG_DEBUG(...) ::twinkle::core::Logger::instance().debug(__VA_ARGS__)
#define LOG_INFO(...)  ::twinkle::core::Logger::instance().info(__VA_ARGS__)
#define LOG_WARNING(...) ::twinkle::core::Logger::instance().warning(__VA_ARGS__)
#define LOG_ERROR(...)   ::twinkle::core::Logger::instance().error(__VA_ARGS__)
#define LOG_CRITICAL(...) ::twinkle::core::Logger::instance().critical(__VA_ARGS__)

} // namespace twinkle::core
