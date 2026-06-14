# DevilutionX-RS 基线记录

**建立日期**: 2026-06-13
**分支**: `devilutionx-rs`
**建立者**: Phase 0 / Phase 0b 调研

## 编译基线: ✅ GREEN

`cargo check` (lib + bin) 通过，0 错误。

> 起点: lib 有 3 个编译错误（`game_state.rs` / `combat_integration.rs` 引用被删的 `monster_exact` / `monster_object_interaction` 模块）。根因是工作区有一个**未完成的大重构**：把 `monster_exact`(1743行) + `monster_object_interaction`(457行) + `monster_dat`/`monster_new` 合并进 `monster.rs`/`monstdat.rs`，并把代码库重组为镜像 C++ 源码树的扁平结构（新增 `diablo.rs`/`inv.rs`/`objdat.rs`/`spelldat.rs`/`monstdat.rs`/`loadsave.rs` 等 + 整套 `DiabloUI/`/`items/`/`monsters/`/`quests/`/`platform/`/`storm/` 目录）。文件和 `mod.rs` 声明删了，但引用方没改完。

**修复手段**（最小侵入，未回退重构）:
- 重建 `monster_exact.rs` 为**适配模块**：re-export `Monster`/`MonsterMode` + 定义 `MonsterManager`（适配新 API：`x`/`y` 字段、`distance_to(x,y)`、`!is_alive()`）
- 重建 `monster_object_interaction.rs`：`monster_check_doors` / `is_position_blocked_by_object` + helper（适配 `monster.x`/`monster.y`、`objdat::ObjectId`、`super::types::DungeonType`）
- 给 `MonsterFlags` 补 `CAN_OPEN_DOOR` 常量 + `can_open_door()` 方法
- 修调用方 API 漂移：`monster.position` → `monster.position()`，`hit_points`/`max_hit_points` → `hp`/`max_hp`，`modify_hp(-x)` → `hp -= x`
- 修 `objdat.rs` import：`crate::levels::types`（bin 上下文不存在）→ `super::types`
- 修测试/示例：`Monster::new` 4参→5参、`GameState::new` 补 seed

## 测试基线: 🔴 RED（+ 崩溃）

`cargo test --lib --bins` 全量并行运行以 **STATUS_STACK_BUFFER_OVERRUN (0xc0000409, 疑栈溢出)** 崩溃，无法给出完整 pass/fail 计数。

**分模块采样**（崩溃源不在这些模块）:

| 模块 | pass | fail | 备注 |
|------|------|------|------|
| `game::monster` | 45 | 8 | 纯断言失败（重构 API 漂移） |
| `game::missiles` | 83 | 12 | `attempt to multiply with overflow` 算术溢出 panic |
| `game::pathfinding` | 2 | 0 | ✅ 全过 |
| `game::lighting` | — | 多 | 有失败 |
| 其他（cel/controls/automap/capture/combat_system/dialogue/gamemenu/inventory） | — | 多 | 重构遗留 |

**结论**: 测试大面积红的根因是**未完成重构留下的 API 漂移**——大量测试按旧 `monster_exact`/旧 `Monster` 字段 API 编写。这是工作流要逐模块清理的债务。

**未决**: 全量崩溃的元凶测试尚未隔离（待办 #5，用 `--test-threads=1` 二分）。

## 端到端基线: ❌ 不存在

Rust 端没有确定性回放测试。C++ 的黄金标准是 `test/timedemo_test.cpp::RunTimedemo("WarriorLevel1to2")`：
- 无头回放 `test/fixtures/timedemo/WarriorLevel1to2/demo_0.dmo`
- 起始存档 `spawn_0.sv`，比对 `demo_0_reference_spawn_0.sv`（**逐字节**）
- 这是**移植接近完成的终态验收**，非近期可达门

## 规模

- Rust: 273 文件 / 163k 行 / 2096 个 `#[test]`
- C++ 参考: 250 文件 / 113k 行
- 自报完成度: ~38%

## 快速命令

```bash
cd devilutionx-rs
cargo check                          # ✅ green
cargo test --lib game::monster       # 分模块跑（避开全量崩溃）
cargo test --lib --bins              # 全量（当前会崩）
```
