#pragma once

#include <cstdint>
#include <string>
#include <vector>
#include <fstream>

namespace devilution::rust_export {

// JSON 输出帮助类
class JsonWriter {
public:
    explicit JsonWriter(const std::string& filename);
    ~JsonWriter();

    void StartObject();
    void EndObject();
    void StartArray();
    void EndArray();

    void WriteKey(const std::string& key);
    void WriteString(const std::string& value);
    void WriteInt(int64_t value);
    void WriteUInt(uint64_t value);
    void WriteFloat(double value);
    void WriteBool(bool value);
    void WriteNull();

    // 便捷方法
    void WriteKeyValue(const std::string& key, const char* value);
    void WriteKeyValue(const std::string& key, const std::string& value);
    void WriteKeyValue(const std::string& key, int value);
    void WriteKeyValue(const std::string& key, int64_t value);
    void WriteKeyValue(const std::string& key, uint64_t value);
    void WriteKeyValue(const std::string& key, double value);
    void WriteKeyValue(const std::string& key, bool value);

    // 写入二进制数据为 base64
    void WriteKeyBinary(const std::string& key, const uint8_t* data, size_t size);

private:
    std::ofstream file_;
    bool needComma_ = false;
    int depth_ = 0;

    void WriteCommaIfNeeded();
    void WriteNewline();
};

// 导出动画帧数据
struct AnimationFrameData {
    uint32_t ticks;          // SDL_GetTicks 值
    uint32_t frameIndex;     // 计算出的帧索引
    uint32_t totalFrames;    // 总帧数
    uint32_t fps;            // FPS 参数
};

// 导出精灵数据
struct SpriteData {
    std::string name;
    int width;
    int height;
    int numFrames;
    std::vector<uint8_t> pixelData;  // RGBA 格式
};

// 导出 UI 元素位置数据
struct UIElementData {
    std::string type;
    int x, y, w, h;
    std::string text;
    uint32_t flags;
};

// 导出函数声明
void ExportAnimationData(const std::string& outputPath);
void ExportSpriteData(const std::string& outputPath, const std::string& spriteName);
void ExportUILayoutData(const std::string& outputPath);
void ExportFocusAnimationSequence(const std::string& outputPath, uint32_t durationMs);

} // namespace devilution::rust_export
