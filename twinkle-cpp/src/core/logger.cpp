/// @file logger.cpp
/// @brief Logger implementation.

#include "twinkle/core/logger.hpp"
#include <chrono>
#include <cstdio>
#include <format>
#include <iomanip>
#include <sstream>

namespace twinkle::core {

void Logger::initialize(std::string_view log_file, LogLevel level) {
    level_ = level;
    if (!log_file.empty()) {
        file_.open(std::string{log_file}, std::ios::app);
    }
}

std::string Logger::timestamp() {
    auto now = std::chrono::system_clock::now();
    auto time = std::chrono::system_clock::to_time_t(now);
    auto ms = std::chrono::duration_cast<std::chrono::milliseconds>(
        now.time_since_epoch()) % 1000;

    std::tm buf{};
    localtime_r(&time, &buf);

    std::ostringstream ss;
    ss << std::put_time(&buf, "%Y-%m-%d %H:%M:%S")
       << '.' << std::setfill('0') << std::setw(3) << ms.count();
    return ss.str();
}

} // namespace twinkle::core
