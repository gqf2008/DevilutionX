/**
 * @file export_animation.cpp
 * @brief 导出动画相关数据供 Rust 验证使用 - 大规模测试数据
 *
 * 此工具导出 DevilutionX 的动画计算逻辑，包括：
 * - GetAnimationFrame 函数的输入/输出对 (数千个测试用例)
 * - Focus 选择器动画序列 (10秒完整序列)
 * - Logo 动画数据
 * - 渲染时序数据
 */

#include "export_types.h"
#include <cstdint>
#include <vector>
#include <iostream>
#include <cmath>

namespace devilution::rust_export {

/**
 * @brief 模拟 GetAnimationFrame 函数
 *
 * 原始 C++ 实现：
 *   uint32_t GetAnimationFrame(uint32_t frames, uint32_t fps) {
 *       return (SDL_GetTicks() / fps) % frames;
 *   }
 *
 * 默认 fps = 60
 */
uint32_t SimulateGetAnimationFrame(uint32_t ticks, uint32_t frames, uint32_t fps = 60)
{
    return (ticks / fps) % frames;
}

void ExportAnimationData(const std::string& outputPath)
{
    JsonWriter writer(outputPath);

    writer.StartObject();

    // 元数据
    writer.WriteKeyValue("description", "DevilutionX Animation Frame Calculation - Comprehensive Test Data");
    writer.WriteKeyValue("version", "2.0");

    // ========== 1. GetAnimationFrame 大规模测试用例 ==========
    writer.WriteKey("animation_frame_tests");
    writer.StartArray();

    // 1.1 基本帧数测试 (1-32帧，不同ticks)
    for (uint32_t frames = 1; frames <= 32; frames++) {
        for (uint32_t ticks = 0; ticks < 3000; ticks += 30) {
            writer.StartObject();
            writer.WriteKeyValue("ticks", static_cast<uint64_t>(ticks));
            writer.WriteKeyValue("frames", static_cast<uint64_t>(frames));
            writer.WriteKeyValue("fps", 60);
            writer.WriteKeyValue("expected", static_cast<uint64_t>(
                SimulateGetAnimationFrame(ticks, frames, 60)));
            writer.EndObject();
        }
    }

    // 1.2 不同 FPS 测试
    uint32_t fpsValues[] = {15, 30, 45, 60, 90, 120};
    for (uint32_t fps : fpsValues) {
        for (uint32_t ticks = 0; ticks < 2000; ticks += 20) {
            writer.StartObject();
            writer.WriteKeyValue("ticks", static_cast<uint64_t>(ticks));
            writer.WriteKeyValue("frames", 8);
            writer.WriteKeyValue("fps", static_cast<uint64_t>(fps));
            writer.WriteKeyValue("expected", static_cast<uint64_t>(
                SimulateGetAnimationFrame(ticks, 8, fps)));
            writer.EndObject();
        }
    }

    // 1.3 边界值测试
    uint32_t edgeTicks[] = {0, 1, 59, 60, 61, 119, 120, 479, 480, 481,
                           1000, 5000, 10000, 60000, 0xFFFF, 0xFFFFFF};
    for (uint32_t ticks : edgeTicks) {
        for (uint32_t frames = 1; frames <= 20; frames++) {
            writer.StartObject();
            writer.WriteKeyValue("ticks", static_cast<uint64_t>(ticks));
            writer.WriteKeyValue("frames", static_cast<uint64_t>(frames));
            writer.WriteKeyValue("fps", 60);
            writer.WriteKeyValue("expected", static_cast<uint64_t>(
                SimulateGetAnimationFrame(ticks, frames, 60)));
            writer.EndObject();
        }
    }

    writer.EndArray();

    // ========== 2. Focus 动画完整序列 (10秒) ==========
    writer.WriteKey("focus_animation_10s");
    writer.StartObject();
    writer.WriteKeyValue("description", "Focus selector animation - 10 second sequence");
    writer.WriteKeyValue("total_frames", 8);
    writer.WriteKeyValue("fps", 60);
    writer.WriteKeyValue("duration_ms", 10000);
    writer.WriteKeyValue("samples", 10001);

    writer.WriteKey("frames");
    writer.StartArray();
    for (uint32_t t = 0; t <= 10000; t++) {
        writer.WriteInt(SimulateGetAnimationFrame(t, 8, 60));
    }
    writer.EndArray();
    writer.EndObject();

    // ========== 3. Focus 尺寸选择测试 (完整) ==========
    writer.WriteKey("focus_size_selection");
    writer.StartArray();
    for (int h = 1; h <= 100; h++) {
        int size;
        const char* name;
        if (h >= 42) {
            size = 2;
            name = "FOCUS_BIG";
        } else if (h >= 30) {
            size = 1;
            name = "FOCUS_MED";
        } else {
            size = 0;
            name = "FOCUS_SMALL";
        }

        writer.StartObject();
        writer.WriteKeyValue("height", h);
        writer.WriteKeyValue("size", size);
        writer.WriteKeyValue("name", name);
        writer.EndObject();
    }
    writer.EndArray();

    // ========== 4. Logo 动画数据 ==========
    writer.WriteKey("logo_animation");
    writer.StartObject();
    writer.WriteKeyValue("total_frames", 15);
    writer.WriteKeyValue("fps", 60);
    writer.WriteKeyValue("cycle_ms", 900);

    writer.WriteKey("frames_5s");
    writer.StartArray();
    for (uint32_t t = 0; t <= 5000; t++) {
        writer.WriteInt(SimulateGetAnimationFrame(t, 15, 60));
    }
    writer.EndArray();
    writer.EndObject();

    // ========== 5. 常用动画参数表 ==========
    writer.WriteKey("common_animations");
    writer.StartArray();

    struct AnimDef {
        const char* name;
        int frames;
        int fps;
        const char* usage;
    };
    AnimDef anims[] = {
        {"focus_selector", 8, 60, "UI menu focus indicator"},
        {"logo", 15, 60, "Main menu animated logo"},
        {"cursor_penta", 8, 60, "Pentagon cursor animation"},
        {"button_highlight", 4, 60, "Button hover effect"},
        {"loading_spinner", 12, 60, "Loading indicator"},
        {"player_idle", 8, 60, "Player idle animation"},
        {"player_walk", 8, 60, "Player walk cycle"},
        {"monster_idle", 6, 60, "Monster idle animation"},
        {"torch_flicker", 8, 60, "Torch fire animation"},
        {"water_ripple", 4, 60, "Water surface animation"},
    };

    for (const auto& anim : anims) {
        writer.StartObject();
        writer.WriteKeyValue("name", anim.name);
        writer.WriteKeyValue("frames", anim.frames);
        writer.WriteKeyValue("fps", anim.fps);
        writer.WriteKeyValue("usage", anim.usage);
        writer.WriteKeyValue("cycle_ms", anim.frames * anim.fps);
        writer.EndObject();
    }
    writer.EndArray();

    // ========== 6. 帧时间查找表 ==========
    writer.WriteKey("frame_timing_lut");
    writer.StartObject();

    // 8帧动画在不同时间点的帧索引
    writer.WriteKey("8_frames_60fps");
    writer.StartArray();
    for (uint32_t t = 0; t < 1000; t++) {
        writer.WriteInt(SimulateGetAnimationFrame(t, 8, 60));
    }
    writer.EndArray();

    // 15帧动画
    writer.WriteKey("15_frames_60fps");
    writer.StartArray();
    for (uint32_t t = 0; t < 1000; t++) {
        writer.WriteInt(SimulateGetAnimationFrame(t, 15, 60));
    }
    writer.EndArray();

    writer.EndObject();

    writer.EndObject();

    std::cout << "Animation data exported (comprehensive): " << outputPath << std::endl;
}

void ExportFocusAnimationSequence(const std::string& outputPath, uint32_t durationMs)
{
    JsonWriter writer(outputPath);

    writer.StartObject();
    writer.WriteKeyValue("type", "focus_animation_sequence");
    writer.WriteKeyValue("duration_ms", static_cast<uint64_t>(durationMs));
    writer.WriteKeyValue("total_frames", 8);
    writer.WriteKeyValue("frame_interval_ms", 60);

    writer.WriteKey("sequence");
    writer.StartArray();

    for (uint32_t t = 0; t <= durationMs; t++) {
        uint32_t frame = SimulateGetAnimationFrame(t, 8, 60);
        writer.WriteInt(frame);
    }

    writer.EndArray();
    writer.EndObject();
}

} // namespace devilution::rust_export
