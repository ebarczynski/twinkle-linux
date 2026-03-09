#include "twinkle/ddc/ddc_manager.hpp"
#include "twinkle/core/logger.hpp"
#include <algorithm>
#include <sstream>

namespace twinkle::ddc {

DDCManager::DDCManager(CommandConfig config)
    : executor_(std::move(config)) {}

Result<void> DDCManager::initialize() {
    if (!executor_.is_available()) {
        return Result<void>(DDCError::NotAvailable);
    }

    return refresh_monitors();
}

Result<void> DDCManager::refresh_monitors() {
    monitors_ = detector_.detect_monitors();

    if (monitors_.empty()) {
        LOG_WARNING("No monitors detected");
    } else {
        LOG_INFO("Detected {} monitor(s)", monitors_.size());
    }

    if (monitor_callback_) {
        monitor_callback_(monitors_);
    }

    return Result<void>();
}

const Monitor* DDCManager::find_monitor(uint8_t bus) const {
    auto it = std::find_if(monitors_.begin(), monitors_.end(),
                          [bus](const Monitor& m) { return m.bus() == bus; });
    return it != monitors_.end() ? &(*it) : nullptr;
}

const Monitor* DDCManager::find_monitor(std::string_view unique_id) const {
    auto it = std::find_if(monitors_.begin(), monitors_.end(),
                          [unique_id](const Monitor& m) {
                              return m.unique_id() == unique_id;
                          });
    return it != monitors_.end() ? &(*it) : nullptr;
}

Result<uint8_t> DDCManager::get_vcp(uint8_t bus, uint16_t code) const {
    return executor_.get_vcp(bus, code);
}

Result<void> DDCManager::set_vcp(uint8_t bus, uint16_t code, uint8_t value) {
    return executor_.set_vcp(bus, code, value);
}

Result<uint8_t> DDCManager::get_brightness(uint8_t bus) const {
    return get_vcp(bus, vcp::Brightness);
}

Result<void> DDCManager::set_brightness(uint8_t bus, uint8_t value) {
    if (!validate_vcp_value(vcp::Brightness, value)) {
        return Result<void>(DDCError::InvalidValue);
    }
    return set_vcp(bus, vcp::Brightness, value);
}

Result<void> DDCManager::adjust_brightness(uint8_t bus, int8_t delta) {
    auto result = get_brightness(bus);
    if (!result.has_value()) {
        return Result<void>(result.error());
    }

    int new_value = static_cast<int>(result.value()) + delta;
    new_value = std::clamp(new_value, 0, 100);
    return set_brightness(bus, static_cast<uint8_t>(new_value));
}

Result<uint8_t> DDCManager::get_contrast(uint8_t bus) const {
    return get_vcp(bus, vcp::Contrast);
}

Result<void> DDCManager::set_contrast(uint8_t bus, uint8_t value) {
    if (!validate_vcp_value(vcp::Contrast, value)) {
        return Result<void>(DDCError::InvalidValue);
    }
    return set_vcp(bus, vcp::Contrast, value);
}

Result<uint8_t> DDCManager::get_volume(uint8_t bus) const {
    return get_vcp(bus, vcp::Volume);
}

Result<void> DDCManager::set_volume(uint8_t bus, uint8_t value) {
    if (!validate_vcp_value(vcp::Volume, value)) {
        return Result<void>(DDCError::InvalidValue);
    }
    return set_vcp(bus, vcp::Volume, value);
}

Result<uint8_t> DDCManager::get_input_source(uint8_t bus) const {
    return get_vcp(bus, vcp::InputSource);
}

Result<void> DDCManager::set_input_source(uint8_t bus, uint8_t value) {
    return set_vcp(bus, vcp::InputSource, value);
}

Result<uint8_t> DDCManager::get_color_temperature(uint8_t bus) const {
    return get_vcp(bus, vcp::ColorTemperature);
}

Result<void> DDCManager::set_color_temperature(uint8_t bus, uint8_t value) {
    return set_vcp(bus, vcp::ColorTemperature, value);
}

bool DDCManager::has_permissions() const noexcept {
    // Check if user has I2C permissions
    // This is simplified - in production, check /dev/i2c-* permissions
    return true;
}

Result<uint8_t> DDCManager::get_cached_vcp(uint8_t bus, uint16_t code) {
    auto key = cache_key(bus, code);
    auto it = vcp_cache_.find(key);

    if (it != vcp_cache_.end()) {
        auto age = std::chrono::steady_clock::now() - it->second.timestamp;
        if (age < std::chrono::seconds(5)) {
            return Result<uint8_t>(it->second.value);
        }
    }

    auto result = get_vcp(bus, code);
    if (result.has_value()) {
        vcp_cache_[key] = {result.value(), std::chrono::steady_clock::now()};
    }

    return result;
}

Result<void> DDCManager::set_cached_vcp(uint8_t bus, uint16_t code, uint8_t value) {
    auto result = set_vcp(bus, code, value);
    if (result.has_value()) {
        auto key = cache_key(bus, code);
        vcp_cache_[key] = {value, std::chrono::steady_clock::now()};
    }
    return result;
}

void DDCManager::invalidate_cache(uint8_t bus) {
    for (auto it = vcp_cache_.begin(); it != vcp_cache_.end(); ) {
        if (it->first.find(std::to_string(bus)) == 0) {
            it = vcp_cache_.erase(it);
        } else {
            ++it;
        }
    }
}

std::string DDCManager::cache_key(uint8_t bus, uint16_t code) const {
    std::stringstream ss;
    ss << static_cast<int>(bus) << ":" << std::hex << static_cast<int>(code);
    return ss.str();
}

} // namespace twinkle::ddc
