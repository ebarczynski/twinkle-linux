/// @file test_utils.cpp
/// @brief Utility and integration tests.

#include <gtest/gtest.h>
#include "twinkle/ddc/error.hpp"
#include "twinkle/ddc/vcp_codes.hpp"
#include "twinkle/ddc/monitor.hpp"

using namespace twinkle::ddc;

TEST(Utils, FormatBrightness) {
    auto m = Monitor::external(1);
    m.model = "Test";
    EXPECT_EQ(m.display_name(), "Test");

    // Test percentage formatting via the value range
    for (uint16_t val = 0; val <= 100; val += 25) {
        EXPECT_TRUE(vcp::validate_vcp_value(0x10, static_cast<uint16_t>(val)));
    }
}

TEST(Utils, ExpectedChaining) {
    // Test that std::expected works for chaining patterns
    auto get_value = []() -> DDCResult<uint8_t> { return 75; };
    auto set_value = [](uint8_t v) -> DDCVoid {
        if (v > 100) return std::unexpected(DDCError::InvalidValue);
        return {};
    };

    auto val = get_value();
    ASSERT_TRUE(val.has_value());
    auto result = set_value(val.value());
    EXPECT_TRUE(result.has_value());

    auto bad = set_value(200);
    EXPECT_FALSE(bad.has_value());
}

TEST(Utils, MonitorTypeComparison) {
    auto ext = Monitor::external(1);
    auto internal = Monitor::internal("test", "/sys/test");

    EXPECT_EQ(ext.monitor_type, MonitorType::External);
    EXPECT_EQ(internal.monitor_type, MonitorType::Internal);
    EXPECT_NE(ext.monitor_type, internal.monitor_type);
}
