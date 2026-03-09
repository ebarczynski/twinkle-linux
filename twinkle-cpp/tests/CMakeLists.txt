# Tests configuration
enable_testing()

# Test executable
add_executable(twinkle-tests
    tests/test_main.cpp
)

target_link_libraries(twinkle-tests
    PRIVATE
    fmt::fmt
    Threads::Threads
)

# Add tests
add_test(NAME DDCMonitorTest COMMAND twinkle-tests)
add_test(NAME ConfigManagerTest COMMAND twinkle-tests)
