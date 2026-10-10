# 原生设置与显示补丁

主入口和托盘使用 Rust、notify、tray-icon 与官方 Windows API。显示引擎使用官方 Frida Gadget 与 dll-syringe，保留指定官方包哈希的字体、调用位置和文件名保护，不修改官方磁盘文件。

设置服务从 Gradio/Python 迁移到独立的 Rust 按需组件。HTTP、静态文件、凭据、文件夹选择和图标编译分别复用 Axum、tower-http、keyring、rfd、winresource。页面复用 Vue 3.5.43、Element Plus 2.14.7、Vite 8.3.4；设置服务不与日常托盘一起加载，不包含嵌入式浏览器或 Python。组件仍由单个分发 EXE 首次解包，界面关闭后释放服务。

AI 的 OpenAI 兼容传输、模型查询和完成请求复用 async-openai 0.42.2。语义缓存复用既有 Rust + redb 实现，缓存包含公开源文、稳定语境、术语、模型、接口和校验指纹，行号或版本漂移不会单独失效。前端语言包是静态数据，不用 AI 实时翻译。

社区提取器和校验器仍在维护端与 CI 使用 Python/Dulwich/tree-sitter，保留固定快照。维护流程导出公开 `data/translation-jobs.json`；本地原生服务消费该文案和语境生成待审草稿，使用独立 draft-translations.redb。原生草稿不标记为社区已校验，不进入经过校验的缓存，不自动安装或扩大运行时支持哈希。社区校验与二进制适配仍分别审核。

服务只绑定 127.0.0.1，修改操作校验本机 Host、Origin 和会话 nonce。API 密钥不写入配置、报告、日志或提交，不输出供应商错误正文；旧系统凭据按原名称读取，新保存使用独立条目。文件夹选择只在用户点击后调用系统对话框。

依赖固定于 Cargo.lock 与 web/package-lock.json。直接组件许可保留在 THIRD_PARTY_NOTICES.md；自有代码 MIT，社区 GPL 资源保留原许可。当前只验证 Windows x64，跨平台 SDK 能力不代表汉化已支持 Linux/macOS/ARM64。

回滚：退出托盘，恢复备份 EXE，保留 zed-zh-data。恢复原快捷方式使用解包目录中的 install-startup.ps1 -Uninstall；官方 Zed 文件保持原样。
