#include "twinkle/core/logger.hpp"
#include <chrono>
#include <ctime>
#include <iomanip>
#include <sstream>

namespace twinkle::core {

Logger::~Logger() {
    if (file_) {
        file_->close();
    }
}

void Logger::initialize(std::string_view log_file, LogLevel level) {
    level_ = level;

    // Open log file
    file_ = std::make_unique<std::ofstream>(std::string(log_file), std::ios::app);

    if (!file_->is_open()) {
        std::cerr << "Failed to open log file: " << log_file << std::endl;
    }
}

std::string Logger::get_timestamp() {
    auto now = std::chrono::system_clock::now();
    auto time = std::chrono::system_clock::to_time_t(now);
    auto ms = std::chrono::duration_cast<std::chrono::milliseconds>(
        now.time_since_epoch()) % 1000;

    std::stringstream ss;
    ss << std::put_time(std::localtime(&time), "%Y-%m-%d %H:%M:%S");
    ss << '.' << std::setfill('0') << std::setw(3) << ms.count();

    return ss.str();
}

std::string_view Logger::level_to_string(LogLevel level) noexcept {
    switch (level) {
        case LogLevel::Trace:    return "TRACE";
        case LogLevel::Debug:    return "DEBUG";
        case LogLevel::Info:     return "INFO";
        case LogLevel::Warning:  return "WARN";
        case LogLevel::Error:    return "ERROR";
        case LogLevel::Critical: return "CRIT";
    }
    return "UNKNOWN";
}

} // namespace twinkle::core
