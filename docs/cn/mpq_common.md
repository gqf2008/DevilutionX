# MPQ 文件通用定义 (mpq_common.hpp / mpq_common.cpp)

## 概述

本模块定义了 MPQ (Mike O'Brien Pack) 文件格式的核心数据结构和通用功能。MPQ 是暴雪娱乐开发的一种归档文件格式，用于存储游戏资源。

## 常量定义

```cpp
constexpr size_t MaxMpqPathSize = 256;
```
- MPQ 文件内路径的最大长度

## 数据结构

### MpqFileHeader - MPQ 文件头

文件头大小：32 字节（不含填充）

| 字段 | 类型 | 说明 |
|------|------|------|
| `signature` | `uint32_t` | 签名，固定为 `'MPQ\x1A'` (0x1A51504D) |
| `headerSize` | `uint32_t` | 头部大小，Diablo MPQ 固定为 32 |
| `fileSize` | `uint32_t` | MPQ 文件总大小（字节） |
| `version` | `uint16_t` | 版本号，Diablo MPQ 固定为 0 |
| `blockSizeFactor` | `uint16_t` | 块大小因子，实际块大小 = `512 * 2^blockSizeFactor` |
| `hashEntriesOffset` | `uint32_t` | 哈希表的文件偏移量 |
| `blockEntriesOffset` | `uint32_t` | 块表的文件偏移量 |
| `hashEntriesCount` | `uint32_t` | 哈希表条目数量 |
| `blockEntriesCount` | `uint32_t` | 块表条目数量 |
| `pad[72]` | `uint8_t[]` | 头部后的填充空间 |

#### 静态常量

```cpp
static constexpr uint32_t DiabloSignature = LoadLE32("MPQ\x1A");
static constexpr uint32_t DiabloSize = 32;
```

### MpqHashEntry - 哈希表条目

用于文件名查找的哈希表条目，大小为 16 字节。

| 字段 | 类型 | 说明 |
|------|------|------|
| `hashA` | `uint32_t` | 用于解决哈希冲突的第一个哈希值 |
| `hashB` | `uint32_t` | 用于解决哈希冲突的第二个哈希值 |
| `locale` | `uint16_t` | 区域设置，Diablo 中固定为 0 |
| `platform` | `uint16_t` | 平台标识，Diablo 中固定为 0 |
| `block` | `uint32_t` | 块表索引或特殊值 |

#### 特殊块值

```cpp
static constexpr uint32_t NullBlock = -1;     // 未使用的哈希条目
static constexpr uint32_t DeletedBlock = -2;  // 已删除的条目（可回收）
```

### MpqBlockEntry - 块表条目

描述文件数据块的信息，大小为 16 字节。

| 字段 | 类型 | 说明 |
|------|------|------|
| `offset` | `uint32_t` | 块在文件中的起始偏移 |
| `packedSize` | `uint32_t` | 压缩后的大小 |
| `unpackedSize` | `uint32_t` | 解压后的大小 |
| `flags` | `uint32_t` | 标志位 |

#### 标志位

```cpp
static constexpr uint32_t FlagExists = 0x80000000;    // 文件存在
static constexpr uint32_t CompressPkZip = 0x00000100; // 使用 PKZip 压缩
```

### MpqFileHash - 文件哈希数组

```cpp
using MpqFileHash = std::array<std::uint32_t, 3>;
```

存储文件名的三个哈希值：
- `[0]`: 用于计算哈希表索引
- `[1]`: hashA - 用于验证
- `[2]`: hashB - 用于验证

## 函数

### CalculateMpqFileHash

```cpp
MpqFileHash CalculateMpqFileHash(std::string_view filename);
```

计算给定文件名的 MPQ 哈希值。

**参数：**
- `filename`: 要计算哈希的文件名

**返回值：**
- 包含三个哈希值的数组

**实现说明：**
- 内部调用 `libmpq__file_hash_s` 函数
- 仅在未定义 `UNPACKED_MPQS` 或 `UNPACKED_SAVES` 时编译

## 内存布局

所有结构体使用 `#pragma pack(push, 1)` 进行 1 字节对齐，确保与文件中的二进制布局完全匹配。

## 文件结构概览

```
┌─────────────────────────────────┐
│      MpqFileHeader (32字节)      │
├─────────────────────────────────┤
│         填充 (72字节)            │
├─────────────────────────────────┤
│       块表 (BlockEntries)        │
│   每个条目 16 字节 × N 个条目     │
├─────────────────────────────────┤
│       哈希表 (HashEntries)       │
│   每个条目 16 字节 × M 个条目     │
├─────────────────────────────────┤
│          文件数据块              │
│      (压缩或未压缩的数据)         │
└─────────────────────────────────┘
```

## 使用示例

```cpp
// 计算文件哈希
MpqFileHash hash = CalculateMpqFileHash("levels/l1data/l1.cel");

// 检查块是否有效
MpqHashEntry entry;
if (entry.block != MpqHashEntry::NullBlock &&
    entry.block != MpqHashEntry::DeletedBlock) {
    // 有效的块引用
}

// 检查块标志
MpqBlockEntry block;
if (block.flags & MpqBlockEntry::FlagExists) {
    // 文件存在
}
```
