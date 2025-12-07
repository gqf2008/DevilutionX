cmake_minimum_required(VERSION 3.22)
project(RustDataExport)

# 使用与主项目相同的 C++ 标准
set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# 独立的导出数据工具（不依赖 DevilutionX 库）
add_executable(rust_data_export
    main.cpp
    json_writer.cpp
    export_animation.cpp
    export_sprite.cpp
    export_ui.cpp
)

# 这是一个独立工具，不需要链接任何外部库
# 它只导出算法和数据定义，不需要 SDL 或其他依赖
