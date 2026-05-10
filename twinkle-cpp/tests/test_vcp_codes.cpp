/// @file test_vcp_codes.cpp
/// @brief Tests for VCP code definitions.

#include <gtest/gtest.h>
#include "twinkle/ddc/vcp_codes.hpp"

using namespace twinkle::ddc::vcp;

TEST(VCPCodes, BrightnessCode) {
    EXPECT_EQ(static_cast<uint8_t>(VCP::Brightness), 0x10);
    EXPECT_EQ(static_cast<uint8_t>(VCP::Contrast), 0x12);
    EXPECT_EQ(static_cast<uint8_t>(VCP::Volume), 0x62);
    EXPECT_EQ(static_cast<uint8_t>(VCP::InputSource), 0x60);
}

TEST(VCPCodes, GetVCPInfo) {
    auto brightness = get_vcp_info(0x10);
    ASSERT_TRUE(brightness.has_value());
    EXPECT_STREQ(brightness->name, "Brightness");
    EXPECT_EQ(brightness->type, ValueType::Continuous);
    EXPECT_EQ(brightness->min_val, 0);
    EXPECT_EQ(brightness->max_val, 100);

    auto contrast = get_vcp_info(0x12);
    ASSERT_TRUE(contrast.has_value());
    EXPECT_STREQ(contrast->name, "Contrast");

    auto unknown = get_vcp_info(0xFF);
    EXPECT_FALSE(unknown.has_value());
}

TEST(VCPCodes, ValidateValue) {
    EXPECT_TRUE(validate_vcp_value(0x10, 0));
    EXPECT_TRUE(validate_vcp_value(0x10, 50));
    EXPECT_TRUE(validate_vcp_value(0x10, 100));
    EXPECT_FALSE(validate_vcp_value(0x10, 101));
    EXPECT_FALSE(validate_vcp_value(0x10, 200));
    // Unknown code — always valid
    EXPECT_TRUE(validate_vcp_value(0xFF, 999));
}

TEST(VCPCodes, VCPCodeName) {
    EXPECT_EQ(vcp_code_name(0x10), "Brightness");
    EXPECT_EQ(vcp_code_name(0x12), "Contrast");
    EXPECT_EQ(vcp_code_name(0x62), "Volume");
    // Unknown code
    auto name = vcp_code_name(0xFE);
    EXPECT_TRUE(name.find("Unknown") != std::string::npos);
}

TEST(VCPCodes, CommonCodes) {
    auto codes = common_vcp_codes();
    EXPECT_EQ(codes.count, 5);
    EXPECT_EQ(codes.codes[0], 0x10); // Brightness
    EXPECT_EQ(codes.codes[1], 0x12); // Contrast
}

TEST(VCPCodes, ConstexprLookup) {
    // Verify constexpr works
    static_assert(get_vcp_info(0x10).has_value());
    static_assert(get_vcp_info(0x10)->code == 0x10);
    static_assert(!get_vcp_info(0xFF).has_value());
    static_assert(validate_vcp_value(0x10, 50));
    static_assert(!validate_vcp_value(0x10, 101));
}
