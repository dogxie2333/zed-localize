# Zed Localize

简体中文 · [English](README.en.md) · [繁體中文](README.zh-TW.md)

为 **官方 Zed** 加载多语言界面，**跟随官方主线，保留官方更新渠道**。

**Windows 版是独立于 Zed 主程序的 DLL 注入工具。** 启动器将官方 Frida Gadget DLL 注入 Zed 进程，在内存中的显示层替换文案；不修改磁盘上的官方 Zed.exe，也不是 Zed 插件。

## 使用

在 [Releases](https://github.com/dogxie2333/zed-localize/releases) 下载 `Zed-Localize.exe` 或便携 ZIP，放在固定、可写的文件夹。

1. 双击运行，选择 Zed 安装文件夹。
2. 选择 **Zed 翻译语言**、自启或手动，保存设置。
3. 启动 Zed，以后照常使用原快捷方式。

托盘右键可打开设置、暂停翻译或退出。配置页仅首次初始化或手动打开时运行，关闭后释放服务。原设置、登录与插件继续使用。

## 语言与 AI

复用社区 13 种译文：简体中文、繁體中文、日本語、한국어、Deutsch、Français、Español、Italiano、Português、Русский、Polski、Čeština、Türkçe。English 恢复原文。

**Zed 翻译语言**控制编辑器；左下角**设置页语言**控制网页，支持简体、繁体、English。切换后重启 Zed 完整生效，选择会保留。

补丁受 UTF-16 长度和安全范围限制，部分文字保留英文，欧美语言覆盖较少。[语言统计](data/locales.json) 是安全候选数量，并非整个界面的覆盖率。

可选 DeepSeek、GLM 或 OpenAI 兼容接口，复用 async-openai SDK。可一次选择多个目标语言，按语境批量翻译新增文案；缓存包含源文、语言、模型、术语与校验版本。

日常启动、输入和显示只查本地词典。AI 在更新维护时生成待审草稿，不读取用户项目，也不批准新二进制。密钥只在内存或自行选择的 Windows 凭据管理器保存。

## 官方更新与文件

照常更新官方 Zed。已验证版本在启动与重启时加载补丁；未知版本先使用英文，等待经过验证的适配。当前验证目标为指定 Windows x64 Zed 1.23.2 包。

入口、托盘与按需设置服务采用 Rust，网页复用 Vue / Element Plus，无需另装运行环境。EXE 旁的 `zed-zh-data/` 保存配置、语言资源与缓存；更新工具时保留此目录，历史名称用于兼容旧版。

[详细使用方法](docs/LOCAL_USAGE.md) · [性能](docs/PERFORMANCE.md) · [项目结构](docs/PROJECT_STRUCTURE.md)

## 开发与许可

构建需 Rust 1.99 GNU、Node、Python（仅维护/构建）与 MinGW：`packaging/build-portable.ps1`。构建缓存在项目外 `.build-cache`。

自有代码 **MIT**；社区译文及 Zed 衍生素材保留 **GPL-3.0** 和上游声明。[许可](THIRD_PARTY_NOTICES.md) · [鸣谢](ACKNOWLEDGEMENTS.md)

感谢 Zed Industries、LI-NA/zed-i18n、Microsoft/vscode-loc、Frida 与各组件维护者。

## Ubuntu 24.04 实验包

提供 x86_64 `.deb` 与便携 `.tar.gz`。Linux 使用独立 Rust 启动器和 ptrace 等长内存数据补丁，当前为指定 Zed 1.23.2 包提供 664 条简体中文安全候选；各语言覆盖不同。该后端不等同于 Windows 显示钩子，暂不含 Linux 托盘/网页设置，编辑器内部重启后需重新从工具启动。

[Linux 使用说明](docs/LINUX.md)。
