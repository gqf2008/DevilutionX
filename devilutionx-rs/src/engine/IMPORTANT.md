# ⚠️ 移植规范

1、如果有外部依赖那么C++用什么模块rust也必须用什么模块
2、必须严格按照C++的逻辑移植，禁止发散和过度设计
3、禁止增加不必要的便捷函数和过度测试
4、禁止占位符或TODO，如果是外部依赖原因必须要写清楚

# ⛔ 已完成移植的文件 - 禁止再次移植！

以下文件已完成移植，每个文件头部都有警告标记，**禁止重复移植**：

## engine/ 目录
- actor_position.rs ✅
- animationinfo.rs ✅
- assets.rs ✅
- backbuffer_state.rs ✅
- circle.rs ✅
- clx_sprite.rs ✅
- demomode.rs ✅
- direction.rs ✅
- displacement.rs ✅
- dx.rs ✅
- events.rs ✅
- lighting_defs.rs ✅
- load_cel.rs ✅
- load_clx.rs ✅
- load_file.rs ✅
- load_pcx.rs ✅
- palette.rs ✅
- path.rs ✅
- point.rs ✅
- points_in_rectangle_range.rs ✅
- random.rs ✅
- rectangle.rs ✅
- size.rs ✅
- sound.rs ✅
- sound_defs.rs ✅
- sound_position.rs ✅
- sound_stubs.rs ✅
- surface.rs ✅
- ticks.rs ✅
- trn.rs ✅
- world_tile.rs ✅

## engine/render/ 目录
- automap_render.rs ✅
- blit_impl.rs ✅
- clx_render.rs ✅
- dun_render.rs ✅
- light_render.rs ✅
- primitive_render.rs ✅
- scrollrt.rs ✅
- text_render.rs ✅