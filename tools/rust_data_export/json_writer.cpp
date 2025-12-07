/**
 * @file json_writer.cpp
 * @brief JSON 输出帮助类实现
 */

#include "export_types.h"
#include <stdexcept>

namespace devilution::rust_export {

// Base64 编码（从 export_sprite.cpp 复用）
static const char base64_chars[] =
    "ABCDEFGHIJKLMNOPQRSTUVWXYZ"
    "abcdefghijklmnopqrstuvwxyz"
    "0123456789+/";

static std::string Base64EncodeInternal(const uint8_t* data, size_t size)
{
    std::string result;
    result.reserve((size + 2) / 3 * 4);

    for (size_t i = 0; i < size; i += 3) {
        uint32_t n = static_cast<uint32_t>(data[i]) << 16;
        if (i + 1 < size) n |= static_cast<uint32_t>(data[i + 1]) << 8;
        if (i + 2 < size) n |= static_cast<uint32_t>(data[i + 2]);

        result.push_back(base64_chars[(n >> 18) & 0x3F]);
        result.push_back(base64_chars[(n >> 12) & 0x3F]);
        result.push_back((i + 1 < size) ? base64_chars[(n >> 6) & 0x3F] : '=');
        result.push_back((i + 2 < size) ? base64_chars[n & 0x3F] : '=');
    }

    return result;
}

JsonWriter::JsonWriter(const std::string& filename)
    : file_(filename)
{
    if (!file_.is_open()) {
        throw std::runtime_error("Cannot open file: " + filename);
    }
}

JsonWriter::~JsonWriter()
{
    file_.close();
}

void JsonWriter::WriteCommaIfNeeded()
{
    if (needComma_) {
        file_ << ",";
    }
}

void JsonWriter::WriteNewline()
{
    file_ << "\n";
    for (int i = 0; i < depth_; i++) {
        file_ << "  ";
    }
}

void JsonWriter::StartObject()
{
    WriteCommaIfNeeded();
    WriteNewline();
    file_ << "{";
    depth_++;
    needComma_ = false;
}

void JsonWriter::EndObject()
{
    depth_--;
    WriteNewline();
    file_ << "}";
    needComma_ = true;
}

void JsonWriter::StartArray()
{
    WriteCommaIfNeeded();
    WriteNewline();
    file_ << "[";
    depth_++;
    needComma_ = false;
}

void JsonWriter::EndArray()
{
    depth_--;
    WriteNewline();
    file_ << "]";
    needComma_ = true;
}

void JsonWriter::WriteKey(const std::string& key)
{
    WriteCommaIfNeeded();
    WriteNewline();
    file_ << "\"" << key << "\":";
    needComma_ = false;
}

void JsonWriter::WriteString(const std::string& value)
{
    file_ << "\"";
    // 转义特殊字符
    for (char c : value) {
        switch (c) {
        case '"': file_ << "\\\""; break;
        case '\\': file_ << "\\\\"; break;
        case '\n': file_ << "\\n"; break;
        case '\r': file_ << "\\r"; break;
        case '\t': file_ << "\\t"; break;
        default: file_ << c;
        }
    }
    file_ << "\"";
    needComma_ = true;
}

void JsonWriter::WriteInt(int64_t value)
{
    file_ << value;
    needComma_ = true;
}

void JsonWriter::WriteUInt(uint64_t value)
{
    file_ << value;
    needComma_ = true;
}

void JsonWriter::WriteFloat(double value)
{
    file_ << value;
    needComma_ = true;
}

void JsonWriter::WriteBool(bool value)
{
    file_ << (value ? "true" : "false");
    needComma_ = true;
}

void JsonWriter::WriteNull()
{
    file_ << "null";
    needComma_ = true;
}

void JsonWriter::WriteKeyValue(const std::string& key, const char* value)
{
    WriteKey(key);
    WriteString(std::string(value));
}

void JsonWriter::WriteKeyValue(const std::string& key, const std::string& value)
{
    WriteKey(key);
    WriteString(value);
}

void JsonWriter::WriteKeyValue(const std::string& key, int value)
{
    WriteKey(key);
    WriteInt(static_cast<int64_t>(value));
}

void JsonWriter::WriteKeyValue(const std::string& key, int64_t value)
{
    WriteKey(key);
    WriteInt(value);
}

void JsonWriter::WriteKeyValue(const std::string& key, uint64_t value)
{
    WriteKey(key);
    WriteUInt(value);
}

void JsonWriter::WriteKeyValue(const std::string& key, double value)
{
    WriteKey(key);
    WriteFloat(value);
}

void JsonWriter::WriteKeyValue(const std::string& key, bool value)
{
    WriteKey(key);
    WriteBool(value);
}

void JsonWriter::WriteKeyBinary(const std::string& key, const uint8_t* data, size_t size)
{
    WriteKeyValue(key, Base64EncodeInternal(data, size));
}

} // namespace devilution::rust_export
