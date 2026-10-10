# 复用来源与许可

项目自有代码采用 GPL-3.0-only；运行时包含社区提取的 UI 文案及译文，因此选择与社区发行物一致的 GPL-3.0，不对所有数据来源作 MIT 的笼统声明。

| 来源 | 当前版本 | 许可 | 使用边界 |
|---|---|---|---|
| LI-NA/zed-i18n | `99301f35d52a33a940ac9a6bfc1d8e5f5decf67b` | 源工具 MIT；发行物 GPL-3.0 | 7430 条中文译文、提取清单、语境和验证工具 |
| zed-industries/zed | `c0199504b9cce79bc27524950022ed701bc3ebd5` / 1.23.2 | Zed GPL-3.0；GPUI 等按上游文件许可 | UI 原文及公开源码位置；不分发官方可执行文件 |
| windows-sys | 0.61.2 | MIT OR Apache-2.0 | 官方 Windows Rust 绑定，不自制注入基础设施 |
| async-openai | 0.42.2 | MIT | 可选维护工具，OpenAI 兼容 SDK，不替换其实现 |
| redb | 4.3.0 | MIT OR Apache-2.0 | 可选维护工具中的嵌入式缓存，无数据库服务 |
| goblin / Aho-Corasick / memmap2 / reqwest / Tokio / serde / sha2 / clap | 见 Cargo.lock | 各自上游许可 | PE、扫描、映射、HTTP、异步和序列化基础能力 |

`assets/catalog.json` 由 `tools/community_bridge.py export` 从固定社区快照生成，保留每条源位置、原文和译文；其 `validator_fingerprint` 对应上游三个校验文件。数据更新必须再次通过未修改的社区校验器。

社区 MIT 原文保留在 `licenses/zed-i18n-MIT.txt`，完整 GPL 文本在项目根 `LICENSE`。Release 工作流还从 Cargo metadata 收集第三方依赖许可文本，随二进制一起发布。

Frida 和 pefile 仅用于此前未迁移的本地 Python 原型对照，不是本仓库 Rust 运行工具的依赖，也不包含在 Release 产物中。
