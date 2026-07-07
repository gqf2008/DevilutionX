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

## 测试基线: ✅ GREEN（2026-07-07 更新）

`cargo test --lib` → **1826 passed; 0 failed; 2 ignored**
`cargo test --bins` → **1479 passed; 0 failed; 2 ignored**
`cargo test --test archive_manager` → 8 passed（真实 MPQ 解压/优先级覆盖）

此前基线为 🔴 RED（全量并行运行以 `STATUS_STACK_BUFFER_OVERRUN` 崩溃，且 `drlg_l2::test_create_dungeon_with_fill_voids` 死循环卡死）。
第 1 轮清理了 15 个失败 + 1 个卡死测试；第 2 轮并行推进了 RNG 移植 / 常量去重 / MPQ 归档管理（+41 测试）。

### 本轮修复清单（15 failed + 1 hang → 0）

| 测试 | 根因 | 修复 |
|------|------|------|
| `sha::test_circular_shift_positive` | `bits=0` → `>>32` 溢出 panic | `wrapping_shl/shr` 让函数 total |
| `sha::test_circular_shift_negative` | 期望值算术错误（注释把 `0x80000000>>27=16` 写成 1） | 修正期望 `0xFFFFFFE4→0xFFFFFFF0` |
| `utils::options::test_default_options` | 默认分辨率 640→1280，测试未跟上 | 更新期望为 1280×720 |
| `utils::options::test_to_ini_format` | 同上 | 更新期望 |
| `pack::test_item_pack_serialize` | `ItemPack::SIZE=18` 错误（C++ `ItemPack` packed = 19 字节） | SIZE 改 19 + 更新 size 断言 |
| `pack::test_player_pack_serialize_deserialize` | 同上（共享 SIZE） | 同上 |
| `spells_cast::test_spell_bitmask` | `player_exact::SpellId` Firebolt=0 与 C++ Firebolt=1 偏移；`None=-1`→移位溢出 | wrapping + 负值返回 0 |
| `movie::test_movie_info_new` | `is_finished`: `0>=0` 误判新建为完成 | `total_frames>0 &&` 守卫 |
| `movie::test_smacker_video_play` | 同上（共享 is_finished） | 同上 |
| `multi::test_player_join_leave` | `init_multiplayer` 未重置 `active_players`（沿用 new()=1） | MP 初始化置 0（host 经 join 加入） |
| `ui::diabloui::mainmenu::test_main_menu_selection` | `from_index` 顺序与 C++ 不符 + 测试期望错 | 对齐 C++ 菜单顺序 + 修正测试（4→ShowCredits, 5→ExitDiablo） |
| `player_new::test_melee_to_hit` | 测试期望 31 用了错误 base=20；权威 TSV base=70 | 期望改 81（lvl1+dex/2 10+0+70） |
| `drlg_l4::test_complete_l4_generation` | `levels::types::DMAXX/DMAXY=112`（应 40）→ flood 循环越界 | flood 用本地 ACTIVE=40 |
| `drlg_l4::test_flood_transparency_values` | `is_floor` 坐标映射错（`x/2` 应为 `(x-16)/2`） | 对齐 C++ `IsFloor` |
| `drlg_l4::test_generate_diablo_lair` | `trans_val_counter: i8` 递增溢出 + 上面的越界 | `wrapping_add` |
| `drlg_l2::test_create_dungeon_with_fill_voids` | `random_chance` 是占位符（`percent>50`）→ `ConnectHall` 方向逻辑振荡死循环 | 加步数上限防御（真实 RNG 接入后可移除） |

## 已知技术债务（供后续移植参考）

1. **常量系统不一致**：`DMAXX/DMAXY/DMAXX/MAXDUNY` 在多处定义且值冲突。
   - 权威（C++）：`DMAXX=DMAXY=40`（活跃区），`MAXDUNX=MAXDUNY=112`（含 16 边距渲染区）。
   - `levels/types.rs` 错误地把 `DMAXX/DMAXY` 也设成 112，导致 `Dungeon.tiles` 被 padding 覆盖。
   - `engine/render/light_render.rs` 的值是对的；`game/automap.rs`、`game/cursor.rs` 又各自重定义。
   - **建议**：统一到一处，消除重复定义。

2. **RNG 部分接入**：`utils/random.rs` 已移植 Diablo 的 Borland LCG（确定性，对齐 C++ `Source/engine/random.cpp`），并接入 `drlg_l2`（`generate()` 调 `set_seed`，`random_chance`/坐标选择用真实引擎）。
   - **遗留**：`drlg_l2::ConnectHall` 对部分种子仍振荡死循环（该端口的走廊导向逻辑与 C++ 有偏差），保留了 `MAX_HALL_STEPS` 步数上限安全网。根治需逐行比对 Rust 与 C++ `connect_hall`。
   - **遗留**：其他生成器（drlg_l1/l3/l4、town、themes、objects 等）的随机调用仍未接入真实 RNG，需要逐个迁移。

2b. **drlg_l4 生成循环范围错误**（阻塞 DMAXX 修正）：`src/levels/drlg_l4.rs` 把 C++ 中 `for (x=0; x<DMAXX=40; ...)` 的生成循环（`make_dmt`/`fix_tiles_patterns`/`add_wall`/`general_fix`/`apply_shadows`/`fix_corner_tiles`/`substitution`/`place_miniset` 等 ~30 处）端口为 `0..MAXDUNX(112)`，并按 `[MAXDUNX][MAXDUNY]` 维度索引 `dungeon.tiles`（C++ 是 `[DMAXX=40][DMAXY=40]`）。这是真实端口 bug，导致把 `levels::types::DMAXX` 从 112 改回 40 时会触发 13 个越界失败。**修正需要把 drlg_l4 的循环/数组语义整体改回 DMAX(40)**，这是独立重构任务。修好后才能把全局 DMAXX 改 40 并清理 `drlg_l4::flood_transparency_values` 的 `ACTIVE_DMAXX` 本地补丁。

3. **两套 SpellId 枚举**：`game/player.rs::SpellId`（Firebolt=1，对齐 C++）与 `game/player_exact.rs::SpellId`（Firebolt=0，遗留适配模块）判别值不同。
   - `spells_cast.rs` 用后者，`get_spell_bitmask` 已做兼容；但长期应统一到权威枚举。

4. **两套战斗数据表**：`playerdat.rs`（Warrior base_melee_to_hit=20，错误）与 `player_dat.rs`（=70，对齐 TSV）并存。
   - **建议**：删除 `playerdat.rs` 的错误副本，统一到 `player_dat.rs`（TSV 来源）。

5. **自报完成度虚高**：`PROGRESS.md`/`README.md` 声称多数模块"100% 完成"，但 `PORTING/151.M80-HONEST-STATUS.md` 已诚实指出游戏流程层（UI/菜单/角色创建/MPQ 加载/启动流程）基本为 0%。后续评估以实际运行 + 测试为准，不信自报数字。

6. **全局 static 导致测试偶发污染**：`cargo check` 报 145 条 `static_mut_refs` 警告。后果：`cargo test --lib` 与 `cargo test --bins` 各自全绿，但 `cargo test --lib --bins` 混合运行时偶发 1 个 monster 测试失败（如 `test_ai_counselor_ranged_attack`，单独/分组跑均通过）。根因是测试间通过全局 static 互相污染，属架构层债务，非逻辑 bug。

## 端到端基线: ❌ 不存在
| `game::missiles` | 83 | 12 | `attempt to multiply with overflow` 算术溢出 panic |
| `game::pathfinding` | 2 | 0 | ✅ 全过 |
| `game::lighting` | — | 多 | 有失败 |
| 其他（cel/controls/automap/capture/combat_system/dialogue/gamemenu/inventory） | — | 多 | 重构遗留 |

**（上表为历史采样，已在 2026-07-07 全部转绿，保留作背景。）**

## 端到端基线: ❌ 不存在

Rust 端没有确定性回放测试。C++ 的黄金标准是 `test/timedemo_test.cpp::RunTimedemo("WarriorLevel1to2")`：
- 无头回放 `test/fixtures/timedemo/WarriorLevel1to2/demo_0.dmo`
- 起始存档 `spawn_0.sv`，比对 `demo_0_reference_spawn_0.sv`（**逐字节**）
- 这是**移植接近完成的终态验收**，非近期可达门

## 规模（2026-07-07 核实）

- Rust: 320 文件 / ~174,500 行 / 2,172 个 `#[test]`（lib 1826 + bins 1479 + 集成 8，合计 ~3,313 测试）
- C++ 参考: ~250 文件 / 143,170 行
- 自报完成度: ~38%（注：自报数字普遍虚高，见"已知技术债务"#5）

## 快速命令

```bash
cd devilutionx-rs
cargo check                          # ✅ green (146 warnings, 0 errors)
cargo test --lib                     # ✅ 1826 passed / 0 failed
cargo test --bins                    # ✅ 1479 passed / 0 failed
cargo test --test archive_manager    # ✅ 8 passed（真实 MPQ 端到端）
cargo test --lib game::monster       # 分模块跑
```
