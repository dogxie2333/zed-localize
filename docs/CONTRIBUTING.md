# 开发与检查

修改后运行相关检查：

```sh
cargo fmt --all --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all-features
node tools/test_display_locales.js
node tools/test_ui_locales.mjs
```

前端构建：在 `web/` 中运行 `npm ci` 与 `npm run build`。

Windows 使用 `packaging/build-portable.ps1` 构建单 EXE。Linux 使用 `packaging/package-linux.py` 生成 DEB 与 tar.gz。维护脚本需要 Python，用户分发包不包含 Python 运行时。

新增显示结构检测规则必须检查唯一匹配、加载代码校验、界面翻译、非界面调用拒绝和重启恢复。不得通过移除内容保护来扩大翻译范围。

测试只使用隔离资料与自建编辑器；不结束用户实例，不提交私人日志或凭据。正式发布附带校验文件、源码来源与完整许可。

# 架构

Windows 的 Rust 启动器负责启动、托盘、快捷方式和更新恢复。显示层复用 Frida Gadget 与 dll-syringe，在内存中翻译 UI 文案。启动时自动识别 PE 可执行段中的显示调用结构，并在加载后再次校验；不按版本号或文件哈希决定是否启用。

显示替换保留字体、UI 调用栈和文件名保护。只读本地字典，不调用网络或 AI。无法识别唯一结构时保留原文，不影响 Zed 的官方更新渠道。

设置服务复用 Axum、Vue、Element Plus、rfd 和 keyring，按需打开。AI 接口复用 async-openai，redb 保存按语境和语言隔离的翻译缓存。社区词典、术语、提取和校验复用 LI-NA/zed-i18n。

Linux 使用独立命令行启动器和等长内存数据补丁；功能范围见 [Linux 使用说明](LINUX.md)。

## 仓库目录

`src/` 是 Rust 代码，`web/` 是设置页，`assets/` 是运行资源，`data/` 是公开语境和语言清单，`tools/` 与 `packaging/` 负责维护、检查与打包。译文 README 集中在 `docs/i18n/`。第三方依赖许可在打包时生成，完整文本随发行包提供。
