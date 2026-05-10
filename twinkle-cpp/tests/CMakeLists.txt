# Tests
enable_testing()

add_executable(twinkle-tests
    test_ddc.cpp
    test_config.cpp
    test_monitor.cpp
    test_vcp_codes.cpp
    test_utils.cpp
    test_main.cpp
)

target_link_libraries(twinkle-tests PRIVATE
    twinkle-core
    GTest::gtest
    GTest::gtest_main
)

include(GoogleTest)
gtest_discover_tests(twinkle-tests)
