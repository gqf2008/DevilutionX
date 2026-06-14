# Utils 模块移植状态

## 已完成移植的模块 (Porting Completed)

### str_case.rs ✅
- **对应 C++ 文件**: `Source/utils/str_case.cpp`, `Source/utils/str_case.hpp`
- **移植日期**: 2024-XX-XX
- **功能**: ASCII 字符串大小写转换
- **测试**: 8 个单元测试
- **状态**: 完整移植，禁止修改

## 等待移植的模块 (Pending)

### parse_int (待移植)
- **对应 C++ 文件**: `Source/utils/parse_int.cpp`, `Source/utils/parse_int.hpp`
- **功能**: 整数解析和定点数解析
- **依赖**: 无外部依赖

## 移植规范 (Porting Standards)

1. 模块名称必须与 C++ 源文件保持一致
2. 严格按照 C++ 逻辑进行移植，不得添加便利函数
3. 不得添加占位符或 TODO 除非有明确的依赖说明
4. 完成的模块禁止修改（除非发现 bug）
5. 每个模块必须有完整的依赖文档和单元测试
