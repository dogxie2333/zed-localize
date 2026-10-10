# 补丁范围与插件边界

项目保留 GPL-3.0，按用户明确要求只维护现有官方发行包的进程补丁。没有添加自编译 Zed、社区发行版安装器或源码构建工作流。

## 译文与可应用补丁

已复用的社区目录中，7430 条 accepted 源文都有中文译文。DeepSeek 负责新增/缺失文案及语境维护，不能单独改变二进制的内联常量、排版入口或机器码引用。

被测官方 Windows x64 1.23.2：保守档 215 条，`--extended` 档 668 条数据补丁候选。扩展档加入纯文本设置标题、说明和通知，继续排除格式化、富文本、译文更长、重复/共享/重叠和未定位的文案。

668 条均通过真实进程内存应用、原文错误拒绝、故障回滚和恢复；启动器退出后也能从另一进程还原。没有将这些结果称为 668 个界面的视觉验收。

短常量经 Rust/SmolStr 优化后不一定是连续静态字符串；如“File”原文为 4 字节、中文为 6 字节，不能通过等长数据覆盖。动态命令名称、占位符和富文本索引也需要额外的定位及上下文处理。本后端未实现这些能力，因此目前仍无法保证“全部界面中文”。

## 官方插件能否承载

核对官方扩展文档与 `crates/extension_api/wit/since_v0.8.0/extension.wit` 后，接口覆盖语言服务、补全/符号代码标签、调试、上下文服务等，没有主界面菜单/设置/GPUI 文案的全局替换回调。code-label 是代码补全内容，不是整个应用界面的标签入口。

插件可以研究用作管理入口，调用一个外部工具或展示翻译状态，但不能因此摆脱进程补丁后端。已经运行的编辑器不能通过该接口回到“创建后、执行前的暂停进程”。没有发布假称全局汉化的占位插件。

当前推荐仍是独立 Rust 程序：没有常驻 Python/Frida/翻译服务，不修改官方磁盘程序；应用后退出，失败回到英文。完整插件化需要官方新增受支持的本地化接口，或稳定、验证充分的运行时挂接后端。

## 来源

- 官方扩展能力：https://zed.dev/docs/extensions/capabilities
- 官方扩展开发与 WASM 边界：https://zed.dev/docs/extensions/developing-extensions
- 被测版本扩展接口：https://github.com/zed-industries/zed/blob/c0199504b9cce79bc27524950022ed701bc3ebd5/crates/extension_api/wit/since_v0.8.0/extension.wit
- 社区现有翻译与限制：https://github.com/LI-NA/zed-i18n
- DeepSeek 官方 JSON 模式：https://api-docs.deepseek.com/guides/json_mode
- DeepSeek 官方前缀缓存：https://api-docs.deepseek.com/guides/kv_cache
