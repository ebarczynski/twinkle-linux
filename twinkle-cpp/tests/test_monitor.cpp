/// @file test_monitor.cpp
/// @brief Tests for Monitor struct.

#include <gtest/gtest.h>
#include "twinkle/ddc/monitor.hpp"

using namespace twinkle::ddc;

TEST(Monitor, ExternalFactory) {
    auto m = Monitor::external(5);
    EXPECT_EQ(m.bus, 5);
    EXPECT_EQ(m.monitor_type, MonitorType::External);
    EXPECT_EQ(m.model, "Unknown Monitor");
    EXPECT_TRUE(m.backlight_path.empty());
}

TEST(Monitor, InternalFactory) {
    auto m = Monitor::internal("intel_backlight", "/sys/class/backlight/intel_backlight");
    EXPECT_EQ(m.bus, -1);
    EXPECT_EQ(m.monitor_type, MonitorType::Internal);
    EXPECT_EQ(m.model, "intel_backlight");
    EXPECT_EQ(m.backlight_path, "/sys/class/backlight/intel_backlight");
    EXPECT_EQ(m.manufacturer, "Internal");
}

TEST(Monitor, UniqueId) {
    auto ext = Monitor::external(3);
    ext.model = "T24i-20";
    EXPECT_EQ(ext.unique_id(), "T24i-20_bus3");

    ext.serial = "ABC123";
    EXPECT_EQ(ext.unique_id(), "ABC123");

    auto internal = Monitor::internal("intel_backlight", "/sys/class/backlight/intel_backlight");
    EXPECT_EQ(internal.unique_id(), "internal_intel_backlight");
}

TEST(Monitor, DisplayName) {
    auto m = Monitor::external(1);
    EXPECT_EQ(m.display_name(), "Monitor (bus 1)");

    m.model = "T24i-20";
    EXPECT_EQ(m.display_name(), "T24i-20");

    m.manufacturer = "LEN";
    EXPECT_EQ(m.display_name(), "LEN T24i-20");

    m.serial = "XYZ";
    EXPECT_EQ(m.display_name(), "LEN T24i-20 (XYZ)");

    auto internal = Monitor::internal("intel_backlight", "/sys/class/backlight/intel_backlight");
    EXPECT_EQ(internal.display_name(), "intel_backlight (Internal)");
}

TEST(Monitor, Validate) {
    auto ext = Monitor::external(5);
    EXPECT_TRUE(ext.validate().has_value());

    auto bad = Monitor::external(-1);
    // External with bus=-1 should fail validation
    auto result = bad.validate();
    EXPECT_FALSE(result.has_value());
}

TEST(MonitorCapabilities, DefaultValues) {
    MonitorCapabilities caps;
    EXPECT_EQ(caps.max_brightness, 100);
    EXPECT_EQ(caps.max_contrast, 100);
    EXPECT_FALSE(caps.supports_input_source);
    EXPECT_FALSE(caps.supports_power_control);
    EXPECT_FALSE(caps.supports_audio);
    EXPECT_TRUE(caps.supported_vcp_codes.empty());
}

TEST(MonitorCapabilities, SupportsVCP) {
    MonitorCapabilities caps;
    caps.supported_vcp_codes.insert(0x10);
    caps.supported_vcp_codes.insert(0x12);

    EXPECT_TRUE(caps.supports_vcp(0x10));
    EXPECT_TRUE(caps.supports_vcp(0x12));
    EXPECT_FALSE(caps.supports_vcp(0x62));
}
