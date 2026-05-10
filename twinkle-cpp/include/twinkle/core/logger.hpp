#pragma once
/// @file logger.hpp
/// @brief Thread-safe logger using fmt.

#include <fmt/core.h>
#include <fstream>
#include <mutex>
#include <string_view>

namespace twinkle::core {

enum class LogLevel : uint8_t { Trace, Debug, Info, Warning, Error, Critical };

class Logger {
public:
    static Logger& instance() noexcept {
        static Logger logger;
        return logger;
    }

    Logger(const Logger&) = delete;
    Logger& operator=(const Logger&) = delete;

    void initialize(std::string_view log_file, LogLevel level = LogLevel::Info);
    void set_level(LogLevel level) noexcept { level_ = level; }

    template<typename... Args>
    void log(LogLevel level, fmt::format_string<Args...> fmt, Args&&... args) {
        if (level < level_) return;
        auto msg = fmt::format(fmt, std::forward<Args>(args)...);
        auto ts = timestamp();
        auto lvl = level_str(level);
        std::lock_guard lock(mutex_);
        if (console_enabled_) {
            fmt::print(stderr, "[{}] [{}] {}\n", ts, lvl, msg);
        }
        if (file_.is_open()) {
            file_ << "[" << ts << "] [" << lvl << "] " << msg << "\n";
            file_.flush();
        }
    }

    template<typename... Args>
    void info(fmt::format_string<Args...> fmt, Args&&... args) {
        log(LogLevel::Info, fmt, std::forward<Args>(args)...);
    }
    template<typename... Args>
    void warn(fmt::format_string<Args...> fmt, Args&&... args) {
        log(LogLevel::Warning, fmt, std::forward<Args>(args)...);
    }
    template<typename... Args>
    void error(fmt::format_string<Args...> fmt, Args&&... args) {
        log(LogLevel::Error, fmt, std::forward<Args>(args)...);
    }
    template<typename... Args>
    void debug(fmt::format_string<Args...> fmt, Args&&... args) {
        log(LogLevel::Debug, fmt, std::forward<Args>(args)...);
    }

private:
    Logger() = default;
    static std::string timestamp();

    static constexpr std::string_view level_str(LogLevel l) noexcept {
        switch (l) {
            case LogLevel::Trace:    return "TRACE";
            case LogLevel::Debug:    return "DEBUG";
            case LogLevel::Info:     return "INFO";
            case LogLevel::Warning:  return "WARN";
            case LogLevel::Error:    return "ERROR";
            case LogLevel::Critical: return "CRIT";
        }
        return "????";
    }

    std::ofstream file_;
    LogLevel level_{LogLevel::Info};
    bool console_enabled_{true};
    std::mutex mutex_;
};

#define LOG_INFO(...)    ::twinkle::core::Logger::instance().info(__VA_ARGS__)
#define LOG_WARN(...)    ::twinkle::core::Logger::instance().warn(__VA_ARGS__)
#define LOG_ERROR(...)   ::twinkle::core::Logger::instance().error(__VA_ARGS__)
#define LOG_DEBUG(...)   ::twinkle::core::Logger::instance().debug(__VA_ARGS__)

} // namespace twinkle::core
