# ⚠️ 移植规范

1、如果有外部依赖那么C++用什么模块rust也必须用什么模块
2、必须严格按照C++的逻辑移植，禁止发散和过度设计
3、禁止增加不必要的便捷函数和过度测试
4、禁止占位符或TODO，如果是外部依赖原因必须要写清楚
5、已移植完成的文件在文件头部标注禁止变更

# ⛔ 已完成移植的文件 - 禁止再次移植！

以下文件已完成移植，每个文件头部都有警告标记，**禁止重复移植**：

- automap_render.rs ✅
- blit_impl.rs ✅
- clx_render.rs ✅
- dun_render.rs ✅
- light_render.rs ✅
- primitive_render.rs ✅
- scrollrt.rs ✅
- text_render.rs ✅