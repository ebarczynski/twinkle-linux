/// @file test_config.cpp
/// @brief Tests for configuration management.

#include <gtest/gtest.h>
#include "twinkle/core/config.hpp"

using namespace twinkle::core;

TEST(ConfigDefaults, General) {
    auto cfg = AppConfig{};
    EXPECT_FALSE(cfg.general.autostart);
    EXPECT_EQ(cfg.general.theme, "system");
    EXPECT_EQ(cfg.general.language, "en_US");
}

TEST(ConfigDefaults, UI) {
    auto cfg = AppConfig{};
    EXPECT_EQ(cfg.ui.auto_hide_delay_ms, 3000);
    EXPECT_FALSE(cfg.ui.show_monitor_selector);
    EXPECT_TRUE(cfg.ui.enable_presets);
    EXPECT_EQ(cfg.ui.preset_values.size(), 5);
    EXPECT_EQ(cfg.ui.preset_values[0], 10);
    EXPECT_EQ(cfg.ui.preset_values[4], 80);
}

TEST(ConfigDefaults, Behavior) {
    auto cfg = AppConfig{};
    EXPECT_EQ(cfg.behavior.debounce_delay_ms, 300);
    EXPECT_TRUE(cfg.behavior.remember_brightness);
    EXPECT_TRUE(cfg.behavior.restore_brightness);
    EXPECT_TRUE(cfg.behavior.enable_notifications);
}

TEST(ConfigDefaults, Advanced) {
    auto cfg = AppConfig{};
    EXPECT_EQ(cfg.advanced.command_timeout_ms, 3000);
    EXPECT_EQ(cfg.advanced.max_retries, 1);
    EXPECT_FALSE(cfg.advanced.debug_logging);
    EXPECT_TRUE(cfg.advanced.log_file_path.empty());
}

TEST(ConfigManager, Create) {
    ConfigManager mgr;
    EXPECT_TRUE(mgr.config().general.theme == "system");
    EXPECT_TRUE(mgr.config().ui.preset_values.size() == 5);
}

TEST(ConfigManager, ResetToDefaults) {
    ConfigManager mgr;
    mgr.config_mut().general.theme = "dark";
    mgr.config_mut().ui.auto_hide_delay_ms = 9999;
    mgr.reset_to_defaults();
    EXPECT_EQ(mgr.config().general.theme, "system");
    EXPECT_EQ(mgr.config().ui.auto_hide_delay_ms, 3000);
}

TEST(ConfigManager, ConfigPath) {
    ConfigManager mgr;
    auto path = mgr.config_path();
    EXPECT_TRUE(path.string().find("twinkle-linux") != std::string::npos);
    EXPECT_TRUE(path.string().find("config.json") != std::string::npos);
}

TEST(MonitorConfig, Defaults) {
    MonitorConfig mc;
    mc.unique_id = "test_monitor";
    mc.display_name = "Test Monitor";
    mc.last_brightness = 75;
    EXPECT_EQ(mc.unique_id, "test_monitor");
    EXPECT_EQ(mc.last_brightness, 75);
}
