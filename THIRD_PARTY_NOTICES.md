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

新增显示补丁运行层使用 Frida 官方 Python 绑定；原静态补丁 CLI 仍可独立运行。

| 来源 | 固定版本 | 上游许可 | 使用范围 |
|---|---|---|---|
| Frida | 17.23.3 | wxWindows Library Licence 3.1（各子组件按自身许可） | 官方挂接、父子进程事件与 JS 运行时 |
| pefile / psutil / filelock / json5 | requirements-runtime.txt | MIT / BSD-3-Clause / Unlicense / Apache-2.0 | PE、进程身份、锁与用户 UI 字体设置 |
| Gradio | 6.30.0 | Apache-2.0 | 按需启动的设置页面组件，不自行实现供应商表单框架 |
| LiteLLM | 1.104.2 | MIT（仅使用 Python SDK） | 供应商兼容层与连接测试，不使用代理企业功能 |
| OpenAI Python SDK | requirements-ui.txt | Apache-2.0 | OpenAI 兼容模型列表 |
| keyring | 25.7.0 | MIT | 用户自行选择保存的 Windows 凭据管理器条目 |
| notify | 8.2.0 | MIT OR Apache-2.0 | Rust 文件系统事件监控 |

依赖从 PyPI/Cargo 官方注册表安装，发行包不重新打包 Python wheels。固定直接依赖后升级时须重新跑接口 mock 与实际启动测试。支持范围、成本与回滚记录见 `docs/RUNTIME_DECISION.md`。
