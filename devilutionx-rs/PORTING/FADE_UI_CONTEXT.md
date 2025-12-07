# 调色板式淡入接线备忘

背景：C++ 使用 `PaletteFadeIn/Out` 按 32 帧(~533ms) 调制调色板。Rust 侧原先在 `main.rs` 里用局部 `fade_value()` 叠加黑色遮罩，现在改为复用 `UiContext` 的淡入计时并对纹理/文本做颜色调制，避免重复实现。

完成的接线
- 渲染函数 `render_title_screen`/`render_main_menu`/`render_selhero` 统一从调用方接收 `fade: u8`，不再内部计算。
- 标题/主菜单/角色选择循环创建 `UiContext`，调用 `start_fade_in(0)`，每帧用 `update_fade(now_ms)` 获取淡入值并传给渲染。
- `fade_value` 取 `min(255)`，与 C++ 256 基准兼容，超过时自动饱和（相当于淡入完成的回退值）。

回退与验证要点
- 若淡入被跳过（例如需立即显示），可直接将 `fade_value` 设为 256 或将渲染时的 `fade` 设为 255，效果等同全亮。
- 533ms 目标：`update_fade` 的步进基于 `current_time` 毫秒 / 2.083，实际渲染帧率不同也会收敛到 255。需要精确匹配时，可改为基于帧计数（32 帧）但保持接口不变。
- 已运行 `cargo check` 确认签名变更与接线编译通过。

后续扩展提示
- 若要接入全局调色板/亮度，沿用同一 `fade` 数值驱动调色板表，而非重新定义淡入计时。
- 硬件光标或背景缺失时，应回退到 `fade=255` 常亮渲染，避免黑屏。
