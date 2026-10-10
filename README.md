# Zed 中文补丁

在**官方 Zed** 中加载中文界面，继续使用 **Zed 官方主线与官方更新渠道**。

## 下载与使用

从 GitHub Releases 下载 `Zed-Zh-Addon.exe`，放在固定、可写的文件夹，双击运行。

1. 选择包含 `Zed.exe` 的安装文件夹。
2. 选择开机自启或手动启动，点击「保存并启用托盘」。
3. 点击「启动中文 Zed」，以后照常使用原来的 Zed 快捷方式。

右下角托盘图标可以右键打开设置、暂停汉化或退出。设置页提供固定左侧导航，支持简体中文、繁体中文和 English；编辑器汉化目前为简体中文。

**AI 是可选功能。** 已有译文在本地加载，输入、滚动和界面显示不调用 AI。可选择 DeepSeek、GLM 或其他 OpenAI 兼容接口；密钥仅在内存或你选择的 Windows 凭据管理器中保存。

## 跟随官方主线更新

直接安装、更新官方 Zed，原有设置、登录和插件继续使用。补丁在启动和重启时重新加载，监控官方安装目录的变化。

新版本需要经过二进制适配验证；尚未适配时正常打开英文版，后续收到经过验证的适配再启用汉化。AI 翻译只生成待审文案，不能批准新二进制。当前验证目标为 Zed 1.23.2 的指定 Windows x64 官方包，不宣称每个新版本或全部界面都已覆盖。

## 轻量运行

日常入口、托盘和按需设置服务使用 Rust。前端复用 Vue / Element Plus，显示引擎复用官方 Frida Gadget。分发包**不含 Python、Gradio 或 LiteLLM 运行时**，不用另装 Python、pip、Rust 或 Git。

单 EXE 首次会在旁边创建 `zed-zh-data`，保存少量解包文件、配置和缓存；设置页关闭后释放服务，托盘继续运行。实际体积和本机测量见 [性能说明](docs/PERFORMANCE.md)。

## 开发与许可

- `src/`：Rust 入口、托盘、设置服务、AI SDK 与缓存。
- `web/`：Vue / Element Plus 页面和三个语言包。
- `assets/`、`data/`：图标、社区译文与经过审查的适配。
- `tools/`：复用社区的提取、校验、维护与测试工具。
- `packaging/`：单 EXE 打包；构建输出位于项目外的 `.build-cache`。
- `docs/`、`licenses/`：使用说明、测试边界和完整许可。

Windows 源码构建：安装 Rust 1.99 GNU、Node.js、Python（仅构建/维护用）和 MinGW，运行 `packaging/build-portable.ps1`。dll-syringe 需要上游不稳定接口，构建脚本仅对该 crate 启用 `RUSTC_BOOTSTRAP`；日常程序不需要这些开发工具。

**自有源代码：MIT。** 社区译文、Zed 衍生图标等资源保留 GPL-3.0 与上游声明，不能将整个资源包改标为纯 MIT。分发附带两类完整许可，详见 [第三方声明](THIRD_PARTY_NOTICES.md)。

感谢 **Zed Industries、LI-NA/zed-i18n、Microsoft/vscode-loc、Frida** 及所有底层组件维护者，完整名单见 [鸣谢](ACKNOWLEDGEMENTS.md)。

[完整使用说明](docs/LOCAL_USAGE.md) · [第三方组件与边界](docs/RUNTIME_DECISION.md)
