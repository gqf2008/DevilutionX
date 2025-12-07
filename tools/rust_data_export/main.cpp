/**
 * @file main.cpp
 * @brief Rust 移植数据导出工具主程序
 *
 * 此工具从 DevilutionX C++ 代码中导出各种数据，供 Rust 移植版本
 * 进行验证和测试。
 *
 * 用法:
 *   rust_data_export [命令] [参数...]
 *
 * 命令:
 *   animation <输出文件>     - 导出动画帧计算测试数据
 *   sprite <输出文件> <名称> - 导出精灵元数据
 *   ui <输出文件>           - 导出 UI 布局数据
 *   focus <输出文件> <时长ms> - 导出 Focus 动画序列
 *   all <输出目录>          - 导出所有数据
 */

#include "export_types.h"
#include <iostream>
#include <string>
#include <filesystem>

namespace fs = std::filesystem;
using namespace devilution::rust_export;

void PrintUsage(const char* program)
{
    std::cout << "DevilutionX Rust 移植数据导出工具\n\n";
    std::cout << "用法: " << program << " <命令> [参数...]\n\n";
    std::cout << "命令:\n";
    std::cout << "  animation <输出文件>       导出动画帧计算测试数据\n";
    std::cout << "  sprite <输出文件> <名称>   导出精灵元数据\n";
    std::cout << "  ui <输出文件>              导出 UI 布局数据\n";
    std::cout << "  focus <输出文件> <时长ms>  导出 Focus 动画序列\n";
    std::cout << "  all <输出目录>             导出所有数据到指定目录\n\n";
    std::cout << "示例:\n";
    std::cout << "  " << program << " animation animation_test_data.json\n";
    std::cout << "  " << program << " all ./test_data/\n";
}

int main(int argc, char* argv[])
{
    if (argc < 2) {
        PrintUsage(argv[0]);
        return 1;
    }

    std::string command = argv[1];

    try {
        if (command == "animation") {
            if (argc < 3) {
                std::cerr << "错误: animation 命令需要输出文件路径\n";
                return 1;
            }
            ExportAnimationData(argv[2]);
        }
        else if (command == "sprite") {
            if (argc < 4) {
                std::cerr << "错误: sprite 命令需要输出文件路径和精灵名称\n";
                return 1;
            }
            ExportSpriteData(argv[2], argv[3]);
        }
        else if (command == "ui") {
            if (argc < 3) {
                std::cerr << "错误: ui 命令需要输出文件路径\n";
                return 1;
            }
            ExportUILayoutData(argv[2]);
        }
        else if (command == "focus") {
            if (argc < 4) {
                std::cerr << "错误: focus 命令需要输出文件路径和时长(ms)\n";
                return 1;
            }
            uint32_t duration = std::stoul(argv[3]);
            ExportFocusAnimationSequence(argv[2], duration);
        }
        else if (command == "all") {
            if (argc < 3) {
                std::cerr << "错误: all 命令需要输出目录路径\n";
                return 1;
            }

            fs::path outputDir = argv[2];
            fs::create_directories(outputDir);

            std::cout << "导出所有数据到: " << outputDir << "\n\n";

            ExportAnimationData((outputDir / "animation_data.json").string());
            ExportSpriteData((outputDir / "sprite_data.json").string(), "all");
            ExportUILayoutData((outputDir / "ui_layout_data.json").string());
            ExportFocusAnimationSequence((outputDir / "focus_sequence.json").string(), 2000);

            std::cout << "\n所有数据导出完成!\n";
        }
        else if (command == "--help" || command == "-h") {
            PrintUsage(argv[0]);
        }
        else {
            std::cerr << "未知命令: " << command << "\n";
            PrintUsage(argv[0]);
            return 1;
        }
    }
    catch (const std::exception& e) {
        std::cerr << "错误: " << e.what() << "\n";
        return 1;
    }

    return 0;
}
