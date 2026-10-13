# 架构

Windows 的 Rust 启动器负责启动、托盘、快捷方式和更新恢复。显示层复用 Frida Gadget 与 dll-syringe，在内存中翻译 UI 文案。启动时自动识别 PE 可执行段中的显示调用结构，并在加载后再次校验；不按版本号或文件哈希决定是否启用。

显示替换保留字体、UI 调用栈和文件名保护。只读本地字典，不调用网络或 AI。无法识别唯一结构时保留原文，不影响 Zed 的官方更新渠道。

设置服务复用 Axum、Vue、Element Plus、rfd 和 keyring，按需打开。AI 接口复用 async-openai，redb 保存按语境和语言隔离的翻译缓存。社区词典、术语、提取和校验复用 LI-NA/zed-i18n。

Linux 使用独立命令行启动器和等长内存数据补丁；功能范围见 [Linux 使用说明](LINUX.md)。
