# DevilutionX Rust 移植 TODO List

## 阶段 1: C ABI 包装器 + 数据验证

### 1.1 设计 C ABI 接口
- [ ] 分析 C++ 核心函数，确定需要导出的函数列表
  - [ ] 动画系统函数 (`GetAnimationFrame`, 动画状态管理)
  - [ ] UI 渲染函数 (Focus 绘制, 布局计算, 文本渲染)
  - [ ] 精灵渲染函数 (CLX 解码, PCX 加载, 像素格式转换)
  - [ ] 游戏逻辑函数 (物品管理, 角色属性, 法术系统)
  - [ ] 输入处理函数 (鼠标/键盘事件处理)
  - [ ] 音频系统函数 (音效播放, 音乐控制)
- [ ] 设计 C 兼容的数据结构
  - [ ] 定义 POD (Plain Old Data) 结构体
  - [ ] 处理 C++ 类到 C 结构体的转换
  - [ ] 设计不透明指针 (opaque pointer) 策略
  - [ ] 定义错误码枚举
- [ ] 设计内存管理策略
  - [ ] 确定谁负责内存分配/释放 (C++ 侧 vs Rust 侧)
  - [ ] 设计对象生命周期管理方案
  - [ ] 处理字符串传递 (UTF-8 vs UTF-16)
  - [ ] 设计缓冲区管理策略

### 1.2 实现 C ABI 包装器
- [x] 创建 C ABI 头文件
  - [x] `devilutionx_c_api.h`: 主 API 头文件
  - [x] `devilutionx_types.h`: C 兼容类型定义
  - [x] `devilutionx_error.h`: 错误码定义
- [x] 实现动画系统包装器
  - [x] `dvlx_get_animation_frame()`: 帧计算函数
  - [x] `dvlx_get_animation_frame_ex()`: 帧计算函数(带错误检查)
  - [ ] `dvlx_animation_create()`: 创建动画对象
  - [ ] `dvlx_animation_update()`: 更新动画状态
  - [ ] `dvlx_animation_destroy()`: 销毁动画对象
- [x] 实现 UI 系统包装器
  - [ ] `dvlx_ui_render_focus()`: 渲染 Focus 高亮
  - [x] `dvlx_ui_calculate_center()`: 计算居中位置
  - [x] `dvlx_ui_select_focus_size()`: 选择 Focus 尺寸
  - [x] `dvlx_ui_calculate_focus_position()`: 计算 Focus 位置
  - [ ] `dvlx_ui_get_dialog_position()`: 获取对话框位置
- [ ] 实现精灵系统包装器
  - [ ] `dvlx_sprite_load_pcx()`: 加载 PCX 文件
  - [ ] `dvlx_sprite_load_clx()`: 加载 CLX 精灵
  - [ ] `dvlx_sprite_render()`: 渲染精灵到缓冲区
  - [ ] `dvlx_sprite_free()`: 释放精灵资源
- [ ] 实现游戏逻辑包装器
  - [ ] `dvlx_player_get_stats()`: 获取玩家属性
  - [ ] `dvlx_item_get_info()`: 获取物品信息
  - [ ] `dvlx_spell_cast()`: 施放法术
  - [ ] `dvlx_dungeon_get_level_info()`: 获取地牢层级信息
- [ ] 实现错误处理机制
  - [x] `dvlx_get_last_error()`: 获取最后错误码
  - [x] `dvlx_get_error_message()`: 获取错误消息
  - [x] `dvlx_clear_error()`: 清除错误状态

### 1.3 添加数据验证层

- [x] 实现参数验证函数
  - [x] 空指针检查
  - [x] 数值范围验证 (帧数、FPS、时间刻度)
  - [ ] 枚举值有效性检查
  - [ ] 字符串长度和编码验证
- [ ] 实现状态一致性检查
  - [ ] 对象生命周期验证 (防止 use-after-free)
  - [ ] 线程安全检查 (如需要)
  - [ ] 资源引用计数验证
- [ ] 添加断言和调试日志
  - [ ] 在 Debug 模式下启用详细日志
  - [ ] 记录函数调用参数
  - [ ] 记录返回值和错误状态
- [ ] 编写单元测试
  - [ ] 测试有效参数调用
  - [ ] 测试无效参数处理
  - [ ] 测试边界条件
  - [x] 测试错误恢复机制

### 1.4 创建测试工具

- [x] 创建 C ABI 测试程序 (`tools/c_api_test/`)
  - [x] `test_animation.c`: 测试动画函数
  - [ ] `test_ui.c`: 测试 UI 函数
  - [ ] `test_sprite.c`: 测试精灵函数
  - [ ] `test_game_logic.c`: 测试游戏逻辑函数
  - [ ] `main.c`: 测试主程序
- [ ] 加载并验证测试数据
  - [ ] 解析 `test_data/animation_data.json`
  - [ ] 解析 `test_data/sprite_data.json`
  - [ ] 解析 `test_data/ui_layout_data.json`
- [ ] 运行所有测试用例
  - [ ] 对比 C ABI 函数输出与预期结果
  - [ ] 统计通过/失败数量
  - [ ] 生成测试报告
- [ ] 集成到 CMake 构建系统
  - [ ] 添加 `c_api_test` 目标
  - [ ] 配置 CTest 自动运行测试
  - [ ] 设置持续集成 (CI) 钩子

## 阶段 2: 编译静态库供 Rust 调用

### 2.1 配置 CMake 构建静态库
- [ ] 创建静态库 CMake 目标
  - [ ] 在 `CMakeLists.txt` 中添加 `add_library(devilutionx_c STATIC ...)`
  - [ ] 列出所有 C ABI 包装器源文件
  - [ ] 配置编译选项 (C++20, 优化级别)
- [ ] 设置导出头文件
  - [ ] 配置 `PUBLIC_HEADER` 属性
  - [ ] 设置头文件安装路径
  - [ ] 生成 `devilutionx_c-config.cmake`
- [ ] 配置依赖关系
  - [ ] 链接 DevilutionX 核心库
  - [ ] 链接 SDL3 库
  - [ ] 链接其他必要依赖 (libpng, zlib 等)
- [ ] 设置编译器标志
  - [ ] 启用 C++20 标准
  - [ ] 设置 `-fvisibility=hidden` (仅导出 C ABI 符号)
  - [ ] 添加 `-DDVLX_C_API_EXPORTS` 宏定义
  - [ ] 配置 Release 优化选项 (`-O3`, LTO)

### 2.2 编译和安装静态库
- [ ] 编译静态库
  ```powershell
  cmake --build build --config Release --target devilutionx_c
  ```
- [ ] 验证库文件生成
  - [ ] Windows: `devilutionx_c.lib`
  - [ ] Linux/macOS: `libdevilutionx_c.a`
- [ ] 检查导出符号
  - [ ] Windows: `dumpbin /SYMBOLS devilutionx_c.lib`
  - [ ] Linux: `nm -C libdevilutionx_c.a`
  - [ ] macOS: `nm -g libdevilutionx_c.a`
- [ ] 安装到本地目录
  ```powershell
  cmake --install build --prefix devilutionx-rs/native
  ```
- [ ] 验证安装文件结构
  - [ ] `native/lib/devilutionx_c.lib` (静态库)
  - [ ] `native/include/devilutionx_c_api.h` (头文件)
  - [ ] `native/include/devilutionx_types.h`
  - [ ] `native/include/devilutionx_error.h`

### 2.3 配置 Rust 项目调用 C 库
- [ ] 创建 Rust 项目结构
  - [ ] `devilutionx-rs/Cargo.toml`: 项目配置
  - [ ] `devilutionx-rs/build.rs`: 构建脚本
  - [ ] `devilutionx-rs/src/ffi/mod.rs`: FFI 绑定模块
  - [ ] `devilutionx-rs/src/lib.rs`: Rust 包装器
- [ ] 配置 `Cargo.toml` 依赖
  ```toml
  [dependencies]
  libc = "0.2"

  [build-dependencies]
  bindgen = "0.70"
  cc = "1.0"
  ```
- [ ] 编写 `build.rs` 构建脚本
  - [ ] 设置静态库搜索路径
    ```rust
    println!("cargo:rustc-link-search=native=../native/lib");
    ```
  - [ ] 链接静态库
    ```rust
    println!("cargo:rustc-link-lib=static=devilutionx_c");
    ```
  - [ ] 链接系统依赖
    ```rust
    println!("cargo:rustc-link-lib=dylib=SDL3");
    println!("cargo:rustc-link-lib=dylib=png");
    ```
  - [ ] 使用 `bindgen` 生成 Rust 绑定
    ```rust
    bindgen::Builder::default()
        .header("../native/include/devilutionx_c_api.h")
        .generate()
        .write_to_file("src/ffi/bindings.rs");
    ```
- [ ] 配置目标平台特定设置
  - [ ] Windows: 链接 MSVC 运行时
  - [ ] Linux: 链接 pthread, dl
  - [ ] macOS: 设置 Framework 搜索路径

### 2.4 创建 Rust FFI 绑定
- [ ] 生成自动绑定 (`bindgen`)
  - [ ] 运行 `cargo build` 生成 `src/ffi/bindings.rs`
  - [ ] 检查生成的函数签名和类型定义
  - [ ] 验证不透明指针类型正确性
- [ ] 创建安全的 Rust 包装器
  - [ ] `src/animation.rs`: 动画系统封装
    ```rust
    pub fn get_animation_frame(ticks: i32, fps: i32, frames: i32) -> i32
    ```
  - [ ] `src/ui.rs`: UI 系统封装
    ```rust
    pub fn select_focus_size(height: i32) -> FocusSize
    ```
  - [ ] `src/sprite.rs`: 精灵系统封装
    ```rust
    pub struct Sprite { ... }
    impl Sprite {
        pub fn load_pcx(path: &str) -> Result<Self, Error>
        pub fn render(&self, target: &mut Buffer) -> Result<(), Error>
    }
    ```
- [ ] 实现 RAII 资源管理
  - [ ] 为不透明指针实现 `Drop` trait
  - [ ] 确保资源自动释放
  - [ ] 防止 double-free 和内存泄漏
- [ ] 实现错误处理
  - [ ] 定义 Rust 错误类型
    ```rust
    #[derive(Debug)]
    pub enum DevilutionError {
        NullPointer,
        InvalidParameter,
        ResourceNotFound,
        // ...
    }
    ```
  - [ ] 从 C 错误码转换为 Rust `Result`
  - [ ] 实现 `std::error::Error` trait

### 2.5 编写 Rust 单元测试
- [ ] 创建测试模块 (`tests/`)
  - [ ] `tests/animation_tests.rs`: 动画测试
  - [ ] `tests/ui_tests.rs`: UI 测试
  - [ ] `tests/sprite_tests.rs`: 精灵测试
  - [ ] `tests/integration_tests.rs`: 集成测试
- [ ] 加载 JSON 测试数据
  ```rust
  use serde_json::Value;
  use std::fs;

  #[test]
  fn test_animation_frame_calculation() {
      let data = fs::read_to_string("test_data/animation_data.json").unwrap();
      let json: Value = serde_json::from_str(&data).unwrap();
      // ...
  }
  ```
- [ ] 验证所有测试用例
  - [ ] 遍历 `basic_tests` (3,200 个用例)
  - [ ] 验证 `fps_tests` (600 个用例)
  - [ ] 验证 `edge_case_tests` (320 个用例)
  - [ ] 验证 `focus_sequence` (10,001 个样本)
- [ ] 运行测试套件
  ```powershell
  cargo test --release
  ```
- [ ] 确保 100% 测试通过率
  - [ ] 修复失败的测试用例
  - [ ] 调查差异原因 (C++ vs Rust 实现)
  - [ ] 生成测试覆盖率报告

### 2.6 性能测试和优化
- [ ] 创建性能基准测试 (`benches/`)
  ```rust
  use criterion::{black_box, criterion_group, criterion_main, Criterion};

  fn benchmark_animation_frame(c: &mut Criterion) {
      c.bench_function("get_animation_frame", |b| {
          b.iter(|| get_animation_frame(black_box(1000), black_box(20), black_box(16)))
      });
  }
  ```
- [ ] 对比 C++ 和 Rust 性能
  - [ ] 测试函数调用开销
  - [ ] 测试批量操作吞吐量
  - [ ] 识别性能瓶颈
- [ ] 优化 FFI 调用
  - [ ] 减少跨边界调用次数
  - [ ] 批量传输数据
  - [ ] 使用内联函数 (适当时)
- [ ] 配置 LTO (Link-Time Optimization)
  ```toml
  [profile.release]
  lto = true
  codegen-units = 1
  ```

### 2.7 文档和示例
- [ ] 编写 API 文档
  - [ ] 为所有公开函数添加 Rust 文档注释
    ```rust
    /// 计算动画的当前帧索引
    ///
    /// # Arguments
    /// * `ticks` - 当前时间刻度
    /// * `fps` - 动画帧率
    /// * `frames` - 总帧数
    ///
    /// # Returns
    /// 当前应显示的帧索引 (0 到 frames-1)
    pub fn get_animation_frame(ticks: i32, fps: i32, frames: i32) -> i32
    ```
  - [ ] 生成 `cargo doc` 文档
  - [ ] 添加使用示例
- [ ] 创建示例程序 (`examples/`)
  - [ ] `examples/simple_animation.rs`: 简单动画演示
  - [ ] `examples/sprite_rendering.rs`: 精灵渲染示例
  - [ ] `examples/ui_layout.rs`: UI 布局示例
- [ ] 更新 README.md
  - [ ] 添加构建说明
  - [ ] 添加使用示例
  - [ ] 说明依赖关系
  - [ ] 列出已知限制

## 阶段 3: 验证和部署

### 3.1 完整测试验证
- [ ] 运行完整测试套件
  ```powershell
  # C++ 测试
  ctest --test-dir build --config Release

  # Rust 测试
  cargo test --release
  ```
- [ ] 验证所有平台
  - [ ] Windows (MSVC)
  - [ ] Linux (GCC/Clang)
  - [ ] macOS (Clang)
- [ ] 内存泄漏检测
  - [ ] Windows: Visual Studio 内存诊断
  - [ ] Linux: Valgrind
  - [ ] macOS: Instruments
- [ ] 生成测试报告
  - [ ] 测试覆盖率
  - [ ] 性能基准
  - [ ] 内存使用情况

### 3.2 持续集成配置
- [ ] 创建 CI 配置文件
  - [ ] `.github/workflows/rust-ci.yml`
- [ ] 配置自动构建
  - [ ] 编译 C++ 静态库
  - [ ] 编译 Rust 项目
  - [ ] 运行所有测试
- [ ] 配置代码质量检查
  - [ ] `cargo clippy`: Rust linter
  - [ ] `cargo fmt --check`: 代码格式检查
  - [ ] C++ clang-format 检查
- [ ] 配置自动发布
  - [ ] 版本标签触发
  - [ ] 生成 release artifacts
  - [ ] 发布到 crates.io (如需要)

### 3.3 文档完善
- [ ] 更新 `TODO.md` (本文件)
  - [ ] 标记已完成任务
  - [ ] 添加实际遇到的问题和解决方案
- [ ] 完善技术文档
  - [ ] C ABI 设计文档
  - [ ] FFI 绑定指南
  - [ ] 性能优化建议
  - [ ] 故障排查指南
- [ ] 创建迁移指南
  - [ ] 从纯 C++ 到 Rust + C ABI 的迁移步骤
  - [ ] API 映射表
  - [ ] 常见陷阱和最佳实践

## 优先级

### P0 (立即开始)
1. 设计 C ABI 接口 (1.1)
2. 实现动画系统包装器 (1.2.1)
3. 编写动画测试 (1.4.1)

### P1 (第二优先级)
4. 配置 CMake 构建静态库 (2.1)
5. 编译和验证静态库 (2.2)
6. 创建 Rust FFI 绑定 (2.4)

### P2 (后续任务)
7. 实现完整的 UI/精灵/游戏逻辑包装器 (1.2.2-1.2.4)
8. 性能测试和优化 (2.6)
9. 文档和示例 (2.7)

### P3 (最终任务)
10. 持续集成配置 (3.2)
11. 文档完善 (3.3)

## 风险和注意事项

### 技术风险
- **ABI 兼容性**: C++ 类无法直接暴露给 C/Rust，需要设计不透明指针
- **内存管理**: 跨语言边界的内存分配/释放需要明确所有权
- **字符串编码**: C++ 可能使用 UTF-16，Rust 使用 UTF-8，需要转换
- **异常处理**: C++ 异常无法穿越 C ABI，需转换为错误码
- **线程安全**: 如果 C++ 代码有全局状态，需要考虑线程安全

### 性能考虑
- **FFI 调用开销**: 频繁跨边界调用可能影响性能
- **数据复制**: 避免不必要的数据复制，考虑共享内存
- **优化级别**: 确保 C++ 和 Rust 都使用 Release 优化

### 维护成本
- **双重维护**: C++ 核心代码和 C ABI 包装器需要同步更新
- **测试负担**: 需要维护 C++ 和 Rust 两套测试
- **文档更新**: API 变更需要同步更新文档

## 里程碑

- [x] **M0**: 选择技术方案 (方案 1: C ABI 包装器)
- [x] **M1**: 创建 C++ 数据导出工具
- [x] **M2**: 生成测试数据 (400+ KB, 3000+ 用例)
- [ ] **M3**: 实现并验证动画系统 C ABI (1-2 天)
- [ ] **M4**: 编译静态库并创建 Rust 绑定 (1-2 天)
- [ ] **M5**: 通过所有动画测试 (1 天)
- [ ] **M6**: 实现完整的 C ABI 包装器 (1-2 周)
- [ ] **M7**: 通过所有测试和性能验证 (3-5 天)
- [ ] **M8**: 文档和 CI 配置完成 (2-3 天)

## 当前状态

- ✅ 已完成测试数据导出工具
- ✅ 已生成 442 KB 测试数据 (3000+ 用例)
- ⏸️ **下一步**: 开始设计和实现 C ABI 接口

---

**最后更新**: 2025年12月7日
**负责人**: gxh
**项目**: DevilutionX Rust 移植
