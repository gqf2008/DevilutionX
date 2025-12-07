# CMake configuration for DevilutionX C API

cmake_minimum_required(VERSION 3.10)

project(devilutionx_c_api
    VERSION 1.0.0
    LANGUAGES CXX
)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# Source files
set(SOURCES
    devilutionx_error.cpp
    devilutionx_core.cpp
    devilutionx_animation.cpp
    devilutionx_ui.cpp
)

# Header files
set(HEADERS
    devilutionx_types.h
    devilutionx_error.h
    devilutionx_c_api.h
)

# Create static library
add_library(devilutionx_c STATIC ${SOURCES})

# Set include directories
target_include_directories(devilutionx_c PUBLIC
    $<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}>
    $<INSTALL_INTERFACE:include>
)

# Compiler options
if(MSVC)
    target_compile_options(devilutionx_c PRIVATE /W4)
    # Hide all symbols except those explicitly exported
    target_compile_definitions(devilutionx_c PRIVATE DVLX_C_API_EXPORTS)
else()
    target_compile_options(devilutionx_c PRIVATE -Wall -Wextra -Wpedantic)
    # Hide symbols by default
    set_target_properties(devilutionx_c PROPERTIES
        CXX_VISIBILITY_PRESET hidden
        VISIBILITY_INLINES_HIDDEN ON
    )
endif()

# Installation rules
install(TARGETS devilutionx_c
    EXPORT devilutionx_c-targets
    ARCHIVE DESTINATION lib
    LIBRARY DESTINATION lib
    RUNTIME DESTINATION bin
)

install(FILES ${HEADERS}
    DESTINATION include/devilutionx
)

# Export configuration
install(EXPORT devilutionx_c-targets
    FILE devilutionx_c-config.cmake
    DESTINATION lib/cmake/devilutionx_c
)
