# MPQ 文件写入器 (mpq_writer.hpp / mpq_writer.cpp)

## 概述

本模块提供 MPQ 归档文件的创建和编辑功能，用于保存游戏存档等数据。

## 类定义

### MpqWriter

MPQ 归档文件的写入类，支持创建新归档、添加/删除/重命名文件。

#### 构造函数

```cpp
explicit MpqWriter(const char *path);
explicit MpqWriter(const std::string &path);
MpqWriter(MpqWriter &&other) = default;
MpqWriter &operator=(MpqWriter &&other) = default;
```

打开或创建一个 MPQ 文件用于写入。支持移动语义。

**行为：**
1. 如果目录不存在，递归创建
2. 如果文件不存在，创建新文件
3. 如果文件存在，读取现有的头部、块表和哈希表
4. 初始化默认的 MPQ 结构（2048 个哈希条目和块条目）

**实现要点：**
```cpp
// 固定的表大小
constexpr uint32_t HashEntriesCount = 2048;
constexpr uint32_t BlockEntriesCount = 2048;

// 块大小因子为 3，即 512 * 2^3 = 4096 字节
constexpr uint16_t BlockSizeFactor = 3;
constexpr uint32_t BlockSize = 512 << BlockSizeFactor;

// 表的存储位置（紧跟头部之后）
constexpr long MpqBlockEntryOffset = sizeof(MpqFileHeader);  // 104
constexpr long MpqHashEntryOffset = MpqBlockEntryOffset + BlockEntrySize;
```

#### 析构函数

```cpp
~MpqWriter();
```

关闭文件时自动：
1. 写入更新后的头部和表
2. 根据需要截断文件大小
3. 关闭文件流

---

### 公共方法

##### HasFile

```cpp
bool HasFile(std::string_view name) const;
```

检查归档中是否存在指定文件。

---

##### RemoveHashEntry

```cpp
void RemoveHashEntry(std::string_view filename);
```

从归档中删除文件。

**实现流程：**
1. 查找文件的哈希条目
2. 将块标记为删除（`DeletedBlock`）
3. 清空块表条目
4. 将释放的空间加入空闲块列表（通过 `AllocBlock`）

---

##### RemoveHashEntries

```cpp
void RemoveHashEntries(bool (*fnGetName)(uint8_t, char *));
```

批量删除文件，通过回调函数获取文件名列表。

---

##### WriteFile

```cpp
bool WriteFile(std::string_view filename, const std::byte *data, size_t size);
```

写入或覆盖文件。

**实现流程：**
1. 删除同名的旧文件（如果存在）
2. 添加新的哈希条目
3. 写入压缩后的文件内容
4. 失败时回滚更改

---

##### RenameFile

```cpp
void RenameFile(std::string_view name, std::string_view newName);
```

重命名文件。只更新哈希表，不移动数据块。

---

### 私有方法

##### IsValidMpqHeader

```cpp
bool IsValidMpqHeader(MpqFileHeader *hdr) const;
```

验证 MPQ 头部是否符合预期格式。

**验证条件：**
- 签名匹配
- 头部大小为 32
- 版本 ≤ 0
- 块大小因子为 3
- 文件大小匹配
- 表偏移量正确
- 表大小正确

---

##### GetHashIndex

```cpp
uint32_t GetHashIndex(MpqFileHash fileHash) const;
```

在哈希表中查找文件。

**算法：**
```cpp
// 使用线性探测解决冲突
for (unsigned idx = fileHash[0] & 0x7FF;
     hashTable_[idx].block != NullBlock;
     idx = (idx + 1) & 0x7FF) {
    // 检查 hashA 和 hashB 是否匹配
    if (hashTable_[idx].hashA == fileHash[1] &&
        hashTable_[idx].hashB == fileHash[2] &&
        hashTable_[idx].block != DeletedBlock) {
        return idx;
    }
}
return HashEntryNotFound;
```

---

##### FetchHandle

```cpp
uint32_t FetchHandle(std::string_view filename) const;
```

通过文件名获取哈希表索引。

---

##### ReadMPQHeader

```cpp
bool ReadMPQHeader(MpqFileHeader *hdr);
```

从文件读取 MPQ 头部。

---

##### InitDefaultMpqHeader

```cpp
void InitDefaultMpqHeader(MpqFileHeader *hdr);
```

初始化默认的 MPQ 头部结构。

---

##### NewBlock

```cpp
MpqBlockEntry *NewBlock(uint32_t *blockIndex = nullptr);
```

分配一个新的块表条目。

---

##### AllocBlock

```cpp
void AllocBlock(uint32_t blockOffset, uint32_t blockSize);
```

将空间标记为空闲。

**实现策略：**
1. 尝试与相邻的空闲块合并
2. 如果空间在文件末尾，直接缩小文件大小
3. 否则创建新的空闲块条目

---

##### FindFreeBlock

```cpp
uint32_t FindFreeBlock(uint32_t size);
```

查找足够大的空闲空间。

**算法：**
1. 遍历块表查找已分配但未使用的空闲块
2. 如果找到足够大的块，从中分配空间
3. 否则在文件末尾分配新空间

---

##### WriteFileContents

```cpp
bool WriteFileContents(const std::byte *fileData, uint32_t fileSize, MpqBlockEntry *block);
```

写入压缩后的文件数据。

**文件数据结构：**
```
┌─────────────────────────────────┐
│   扇区偏移表 (4字节 × (N+1))     │
│   [0] = 第一个扇区的偏移         │
│   [1] = 第二个扇区的偏移         │
│   ...                           │
│   [N] = 文件结束偏移             │
├─────────────────────────────────┤
│   扇区 0 (压缩数据)              │
├─────────────────────────────────┤
│   扇区 1 (压缩数据)              │
├─────────────────────────────────┤
│   ...                           │
└─────────────────────────────────┘
```

**压缩：**
- 使用 `PkwareCompress` 进行 PKZip 压缩
- 每个扇区独立压缩（4096 字节）
- 如果压缩后比原始数据大，保留原始数据

---

##### WriteHeader / WriteBlockTable / WriteHashTable / WriteHeaderAndTables

```cpp
bool WriteHeader();
bool WriteBlockTable();
bool WriteHashTable();
bool WriteHeaderAndTables();
```

写入 MPQ 结构。`WriteHeaderAndTables` 是便捷方法，依次调用上述三个写入函数。

**加密：**
- 块表使用 `LIBMPQ_BLOCK_TABLE_HASH_KEY` 加密
- 哈希表使用 `LIBMPQ_HASH_TABLE_HASH_KEY` 加密

```cpp
// 写入前加密
libmpq__encrypt_block(blockTable_.get(), BlockEntrySize, LIBMPQ_BLOCK_TABLE_HASH_KEY);
stream_.Write(blockTable_.get(), BlockEntrySize);
// 写入后解密（保持内存中的数据可用）
libmpq__decrypt_block(blockTable_.get(), BlockEntrySize, LIBMPQ_BLOCK_TABLE_HASH_KEY);
```

---

##### AddFile

```cpp
MpqBlockEntry *AddFile(std::string_view filename, MpqBlockEntry *block, uint32_t blockIndex);
```

添加文件到哈希表。

**冲突处理：**
- 使用线性探测法查找空槽
- 检测哈希冲突（同名文件）时报错

---

## 私有成员

| 成员 | 类型 | 说明 |
|------|------|------|
| `stream_` | `LoggedFStream` | 文件流 |
| `name_` | `std::string` | 文件路径 |
| `size_` | `uint32_t` | 当前文件大小 |
| `hashTable_` | `unique_ptr<MpqHashEntry[]>` | 哈希表（2048 条目） |
| `blockTable_` | `unique_ptr<MpqBlockEntry[]>` | 块表（2048 条目） |
| `streamBegin_` | `long` | 流起始位置（仅非 CAN_SEEKP_BEYOND_EOF 平台） |

## 使用示例

```cpp
// 创建或打开存档
MpqWriter writer("savegame.sv");

// 写入文件
std::string saveData = "...";
writer.WriteFile("hero.sv",
    reinterpret_cast<const std::byte*>(saveData.data()),
    saveData.size());

// 检查文件
if (writer.HasFile("hero.sv")) {
    // 文件存在
}

// 删除文件
writer.RemoveHashEntry("old_save.sv");

// 重命名文件
writer.RenameFile("hero.sv", "hero_backup.sv");

// 析构时自动保存
```

## 文件布局

DevilutionX 的 MPQ 写入器使用固定的文件布局：

```
偏移 0:        MpqFileHeader (32 字节)
偏移 104:      块表 (2048 × 16 = 32768 字节)
偏移 32872:    哈希表 (2048 × 16 = 32768 字节)
偏移 65640+:   文件数据
```

这与原版 Diablo 不同，原版将表放在文件末尾。

## 平台兼容性

```cpp
// Amiga 平台无法在 EOF 之后 Seek
#ifndef __AMIGA__
#define CAN_SEEKP_BEYOND_EOF
#endif
```

在不支持超出 EOF seek 的平台上，需要先写入填充数据。
