# CMake configuration for C API tests

cmake_minimum_required(VERSION 3.10)

project(c_api_test LANGUAGES C)

set(CMAKE_C_STANDARD 11)
set(CMAKE_C_STANDARD_REQUIRED ON)

# Find the C API library
add_subdirectory(../c_api ${CMAKE_CURRENT_BINARY_DIR}/c_api)

# Test executable
add_executable(c_api_test
    test_animation.c
)

# Link with C API library
target_link_libraries(c_api_test PRIVATE devilutionx_c)

# Enable testing
enable_testing()

# Add test
add_test(NAME AnimationTests COMMAND c_api_test)
