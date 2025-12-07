# MPQ 文件读取器 (mpq_reader.hpp / mpq_reader.cpp)

## 概述

本模块提供 MPQ 归档文件的读取功能，封装了 `libmpq` 库的底层接口，提供高层次的 C++ API。

## 类定义

### MpqArchive

MPQ 归档文件的读取类，提供文件查找、读取和解压功能。

#### 公共方法

##### 静态工厂方法

```cpp
static std::optional<MpqArchive> Open(const char *path, int32_t &error);
```

打开一个 MPQ 归档文件。

**参数：**
- `path`: MPQ 文件路径
- `error`: 输出参数，存储错误码

**返回值：**
- 成功时返回 `MpqArchive` 对象
- 文件不存在时返回 `nullopt`（不设置错误）
- 其他错误时返回 `nullopt` 并设置错误码

**实现细节：**
```cpp
std::optional<MpqArchive> MpqArchive::Open(const char *path, int32_t &error)
{
    mpq_archive_s *archive;
    error = libmpq__archive_open(&archive, path, -1);
    if (error != 0) {
        if (error == LIBMPQ_ERROR_EXIST)
            error = 0;  // 文件不存在不视为错误
        return std::nullopt;
    }
    return MpqArchive { std::string(path), archive };
}
```

---

##### Clone

```cpp
std::optional<MpqArchive> Clone(int32_t &error);
```

创建当前归档的副本（用于多线程访问）。

---

##### ErrorMessage

```cpp
static const char *ErrorMessage(int32_t errorCode);
```

将错误码转换为可读的错误消息。

---

##### GetFileNumber

```cpp
bool GetFileNumber(MpqFileHash fileHash, uint32_t &fileNumber);
```

通过文件哈希获取文件编号。

**参数：**
- `fileHash`: 文件名的三元哈希值
- `fileNumber`: 输出参数，存储文件编号

**返回值：**
- `true` 如果文件存在
- `false` 如果文件不存在

---

##### ReadFile

```cpp
std::unique_ptr<std::byte[]> ReadFile(
    std::string_view filename,
    std::size_t &fileSize,
    int32_t &error
);
```

读取并解压整个文件。

**参数：**
- `filename`: 文件名
- `fileSize`: 输出参数，存储解压后的文件大小
- `error`: 输出参数，存储错误码

**返回值：**
- 成功时返回包含文件数据的智能指针
- 失败时返回 `nullptr`

**实现流程：**
1. 通过文件名获取文件编号
2. 获取解压后的文件大小
3. 打开块偏移表
4. 分配缓冲区
5. 获取临时缓冲区用于解压
6. 调用 `libmpq__file_read_with_filename_and_temporary_buffer_s` 读取
7. 关闭块偏移表

---

##### ReadBlock

```cpp
int32_t ReadBlock(
    uint32_t fileNumber,
    uint32_t blockNumber,
    uint8_t *out,
    size_t outSize
);
```

读取文件的单个数据块。

---

##### GetUnpackedFileSize

```cpp
std::size_t GetUnpackedFileSize(uint32_t fileNumber, int32_t &error);
```

获取文件解压后的大小。

---

##### GetNumBlocks

```cpp
uint32_t GetNumBlocks(uint32_t fileNumber, int32_t &error);
```

获取文件的数据块数量。

---

##### OpenBlockOffsetTable / CloseBlockOffsetTable

```cpp
int32_t OpenBlockOffsetTable(uint32_t fileNumber, std::string_view filename);
int32_t CloseBlockOffsetTable(uint32_t fileNumber);
```

打开/关闭文件的块偏移表。在读取大文件时需要先打开偏移表。

---

##### GetBlockSize

```cpp
std::size_t GetBlockSize(uint32_t fileNumber, uint32_t blockNumber, int32_t &error);
```

获取指定数据块的大小（需要先打开块偏移表）。

---

##### HasFile

```cpp
bool HasFile(std::string_view filename) const;
```

检查归档中是否存在指定文件。

**实现：**
```cpp
bool MpqArchive::HasFile(std::string_view filename) const
{
    std::uint32_t fileNumber;
    const int32_t error = libmpq__file_number_s(
        archive_, filename.data(), filename.size(), &fileNumber
    );
    return error == 0;
}
```

#### 移动语义

```cpp
MpqArchive(MpqArchive &&other) noexcept;
MpqArchive &operator=(MpqArchive &&other) noexcept;
```

类支持移动语义，不支持拷贝。

#### 析构函数

```cpp
~MpqArchive();
```

自动关闭归档文件，释放 `libmpq` 资源。

## 私有成员

| 成员 | 类型 | 说明 |
|------|------|------|
| `path_` | `std::string` | 归档文件路径 |
| `archive_` | `mpq_archive_s*` | libmpq 归档句柄 |
| `tmp_buf_` | `std::vector<uint8_t>` | 临时解压缓冲区 |

### GetTemporaryBuffer

```cpp
std::vector<std::uint8_t> &GetTemporaryBuffer(std::size_t size);
```

获取临时缓冲区，按需扩展大小。用于解压操作。

## 使用示例

```cpp
// 打开 MPQ 文件
int32_t error;
auto mpq = MpqArchive::Open("spawn.mpq", error);
if (!mpq) {
    std::cerr << "Failed to open: " << MpqArchive::ErrorMessage(error);
    return;
}

// 检查文件是否存在
if (mpq->HasFile("levels/l1data/l1.cel")) {
    // 读取文件
    std::size_t size;
    auto data = mpq->ReadFile("levels/l1data/l1.cel", size, error);
    if (data) {
        // 处理数据...
    }
}

// 使用哈希直接查找
MpqFileHash hash = CalculateMpqFileHash("levels/l1data/l1.cel");
uint32_t fileNum;
if (mpq->GetFileNumber(hash, fileNum)) {
    // 获取文件信息
    auto fileSize = mpq->GetUnpackedFileSize(fileNum, error);
}
```

## 错误处理

所有可能失败的操作都通过 `int32_t &error` 输出参数返回错误码。可以使用 `MpqArchive::ErrorMessage()` 获取错误描述。

常见错误：
- `LIBMPQ_ERROR_EXIST`: 文件不存在
- `LIBMPQ_ERROR_OPEN`: 无法打开文件
- `LIBMPQ_ERROR_READ`: 读取失败

## 线程安全

单个 `MpqArchive` 实例不是线程安全的。如需多线程访问，应使用 `Clone()` 方法为每个线程创建独立的副本。
