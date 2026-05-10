/// @file test_ddc.cpp
/// @brief Tests for DDC error types and command parsing.

#include <gtest/gtest.h>
#include "twinkle/ddc/error.hpp"
#include "twinkle/ddc/command.hpp"
#include "twinkle/ddc/vcp_codes.hpp"

using namespace twinkle::ddc;

TEST(DDCError, ToString) {
    EXPECT_EQ(to_string(DDCError::NotAvailable), "DDC/CI not available");
    EXPECT_EQ(to_string(DDCError::MonitorNotFound), "Monitor not found");
    EXPECT_EQ(to_string(DDCError::VCPNotSupported), "VCP code not supported");
    EXPECT_EQ(to_string(DDCError::PermissionDenied), "Permission denied");
    EXPECT_EQ(to_string(DDCError::CommandFailed), "Command failed");
    EXPECT_EQ(to_string(DDCError::Timeout), "Timeout");
    EXPECT_EQ(to_string(DDCError::InvalidValue), "Invalid value");
    EXPECT_EQ(to_string(DDCError::ParseError), "Parse error");
}

TEST(DDCError, IsPermissionError) {
    EXPECT_TRUE(is_permission_error("Permission denied"));
    EXPECT_TRUE(is_permission_error("EACCES error"));
    EXPECT_TRUE(is_permission_error("Access denied"));
    EXPECT_FALSE(is_permission_error("Success"));
    EXPECT_FALSE(is_permission_error("Command not found"));
}

TEST(DDCResult, ExpectedValue) {
    DDCResult<int> result = 42;
    EXPECT_TRUE(result.has_value());
    EXPECT_EQ(result.value(), 42);
    EXPECT_TRUE(static_cast<bool>(result));
}

TEST(DDCResult, ExpectedError) {
    DDCResult<int> result = std::unexpected(DDCError::Timeout);
    EXPECT_FALSE(result.has_value());
    EXPECT_EQ(result.error(), DDCError::Timeout);
    EXPECT_FALSE(static_cast<bool>(result));
}

TEST(DDCVoid, Success) {
    DDCVoid result = {};
    EXPECT_TRUE(result.has_value());
}

TEST(DDCVoid, Failure) {
    DDCVoid result = std::unexpected(DDCError::CommandFailed);
    EXPECT_FALSE(result.has_value());
    EXPECT_EQ(result.error(), DDCError::CommandFailed);
}

TEST(CommandResult, ParseVcpValue) {
    // Standard format
    auto val1 = CommandExecutor::parse_vcp_value(
        "VCP code 0x10 (Brightness): current value = 75, max value = 100");
    ASSERT_TRUE(val1.has_value());
    EXPECT_EQ(*val1, 75);

    // Value at boundary
    auto val2 = CommandExecutor::parse_vcp_value(
        "VCP code 0x10 (Brightness): current value = 0, max value = 100");
    ASSERT_TRUE(val2.has_value());
    EXPECT_EQ(*val2, 0);

    auto val3 = CommandExecutor::parse_vcp_value(
        "VCP code 0x10 (Brightness): current value = 100, max value = 100");
    ASSERT_TRUE(val3.has_value());
    EXPECT_EQ(*val3, 100);

    // Invalid input
    auto val4 = CommandExecutor::parse_vcp_value("DDC/CI communication failed");
    EXPECT_FALSE(val4.has_value());
}

TEST(CommandResult, ErrorMessage) {
    CommandResult result{
        .success = false,
        .return_code = 1,
        .stderr_out = "error msg",
        .stdout_out = "output",
        .command = "ddcutil detect",
    };
    auto msg = result.error_message();
    EXPECT_TRUE(msg.find("exit 1") != std::string::npos);
    EXPECT_TRUE(msg.find("error msg") != std::string::npos);

    CommandResult ok{.success = true};
    EXPECT_TRUE(ok.error_message().empty());
}

TEST(VCPValue, TableCompleteness) {
    // All common VCP codes should be in the table
    auto common = vcp::common_vcp_codes();
    for (uint8_t i = 0; i < common.count; ++i) {
        EXPECT_TRUE(vcp::get_vcp_info(common.codes[i]).has_value())
            << "VCP code 0x" << std::hex << (int)common.codes[i] << " not in table";
    }
}
