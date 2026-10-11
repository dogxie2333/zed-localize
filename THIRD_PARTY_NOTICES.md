# 第三方许可与来源

自有源代码 MIT。来源于 Zed 与社区的源文、译文、文案清单及衍生标志资源保留 GPL-3.0，完整文本在 licenses/GPL-3.0.txt。MIT 不重新授权这些资源，分发需继续满足其许可要求。

| 来源 | 固定版本/快照 | 上游许可 | 用途 |
|---|---|---|---|
| zed-industries/zed | c0199504b9cce79bc27524950022ed701bc3ebd5 / 1.23.2 | GPL-3.0；部分组件按其文件许可 | 公开文案、源码语境与标志参考 |
| LI-NA/zed-i18n | 99301f35d52a33a940ac9a6bfc1d8e5f5decf67b | 工具代码 MIT；Zed 衍生数据 GPL-3.0 | 词典、术语、提取器与校验器 |
| Microsoft/vscode-loc | 社区项目所引资料 | MIT | 本地化资料，保留社区来源 |
| Frida Gadget | 17.23.3 | wxWindows Library Licence 3.1，子组件按自身许可 | 官方显示运行时 |
| dll-syringe | 0.18.2 | MIT | 成熟 DLL 加载库 |
| Smithay/wayland-rs / wayland-client | 0.31.15 | MIT | Linux 单次启动的 Wayland registry 就绪检查 |
| Vue / Element Plus / Vite | 3.5.43 / 2.14.7 / 8.3.4 | MIT | 静态页面、语言包与构建 |
| Axum / tower-http / Tokio | Cargo.lock | MIT | 按需服务、静态资源与异步运行 |
| async-openai | 0.42.2 | MIT | OpenAI 兼容成熟 SDK，模型列表与翻译 |
| keyring / rfd / winresource | 3.6.3 / 0.17.2 / 0.1.31 | 各自 MIT 或 MIT/Apache-2.0 | 系统凭据、目录选择与图标编译 |
| redb / windows-sys / notify / tray-icon / muda | Cargo.lock | 各自上游许可 | 缓存、Windows API、事件与托盘 |
| Dulwich / tree-sitter / Pillow | 维护/构建环境 | 各自上游许可 | 维护端复用，不作为 Python 运行时发布 |

构建依赖的完整许可由 tools/collect_licenses.py 收集。前端公开组件许可保留于 web/package-lock.json 对应上游包，本地构建导出至 licenses/frontend。Frida 下载只使用官方 release，压缩包和 DLL 哈希固定在 data/gadget.lock.json。

Zed 与 Zed 标志属于 Zed Industries；本项目为独立社区补丁。详见 ACKNOWLEDGEMENTS.md 与 assets/LICENSE.md。
