# ⚠️ 移植规范

1、如果有外部依赖那么C++用什么模块rust也必须用什么模块
2、必须严格按照C++的逻辑移植，禁止发散和过度设计
3、禁止增加不必要的便捷函数和过度测试
4、禁止占位符或TODO，如果是外部依赖原因必须要写清楚
5、已移植完成的文件在文件头部标注禁止变更

# ⛔ 已完成移植的文件 - 禁止再次移植！

## Source/player.cpp → src/game/player.rs
- 状态: ✅ 完成
- C++ 行数: 3505 行, 82 函数
- Rust 行数: 4745 行, 103 pub fn
- 测试: 7/7 通过

## Source/dead.cpp → src/game/dead.rs
- 状态: ⚠️ 部分完成（需要外部依赖）
- C++ 行数: 88 行, 5 函数
- Rust 行数: ~280 行
- 已实现:
  - `AddCorpse()` ✅ 完整实现
  - `Corpse` 结构 ✅
  - 常量 `MAX_CORPSES`, `MAXDUNX`, `MAXDUNY` ✅
  - 辅助函数 `get_corpse_id()`, `get_corpse_direction()` ✅
- 待实现（需要先移植依赖）:
  - `InitCorpses()` - 需要 monster.cpp 全局状态
  - `MoveLightsToCorpses()` - 需要 monster.cpp 全局状态
- 测试: 6/6 通过

## Source/utils/str_case.cpp → src/utils/str_case.rs
- 状态: ✅ 完成
- C++ 行数: 10 行, 2 函数
- Rust 行数: ~160 行
- 已实现:
  - `ascii_str_to_lower_in_place()` ✅
  - `ascii_str_to_lower()` ✅
- 测试: 8/8 通过

## Source/utils/parse_int.cpp → src/utils/parse_int.rs
- 状态: ✅ 完成
- C++ 行数: 29 行 (.cpp) + 95 行 (.hpp)
- Rust 行数: ~280 行
- 已实现:
  - `ParseIntError` 枚举 ✅
  - `parse_int()` ✅
  - `parse_fixed6_fraction()` ✅
  - `parse_fixed6()` ✅
  - `Bounded` trait ✅
- 测试: 8/8 通过

## Source/utils/math.h → src/utils/math.rs
- 状态: ✅ 完成
- C++ 行数: 70 行 (纯头文件)
- Rust 行数: ~190 行
- 已实现:
  - `sign()` ✅
  - `lerp()` ✅
  - `inv_lerp()` ✅
  - `remap()` ✅
- 测试: 9/9 通过

## Source/utils/bitset2d.hpp → src/utils/bitset2d.rs
- 状态: ✅ 完成
- C++ 行数: 50 行 (纯头文件)
- Rust 行数: ~170 行
- 已实现:
  - `Bitset2d` 结构 ✅
  - `test()` ✅
  - `set()` ✅
  - `reset_at()` / `reset()` ✅
  - `count()` ✅
- 测试: 6/6 通过

## Source/utils/enum_traits.h → src/utils/enum_traits.rs
- 状态: ✅ 完成
- C++ 行数: 120 行 (纯头文件模板)
- Rust 行数: ~200 行
- 已实现:
  - `EnumSize` trait ✅
  - `EnumValues` trait + `EnumIterator` ✅
  - `FlagsEnum` trait ✅
  - `has_any_of()`, `has_all_of()`, `has_none_of()` ✅
  - `impl_flags_enum!` 宏 ✅
- 测试: 3/3 通过

## Source/vision.cpp → src/game/vision.rs
- 状态: ✅ 完成
- C++ 行数: 122 行
- Rust 行数: ~590 行
- 已实现:
  - `VISION_RAYS` 表格 ✅
  - `RAY_LEN_ADJ` 射线长度调整 ✅
  - `QUADRANTS` 四象限镜像 ✅
  - `do_vision()` 核心函数 ✅
  - `is_tile_visible()` 辅助函数 ✅
  - `VisionState` 状态管理 ✅
- 测试: 14/14 通过
