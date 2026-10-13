<div align="center">
<img src="docs/brand.svg" alt="Zed Localize" width="520">

跟随官方主线的 Zed 界面本地化工具。

[![Release](https://img.shields.io/github/v/release/dogxie2333/zed-localize?color=black)](https://github.com/dogxie2333/zed-localize/releases/latest) [![CI](https://img.shields.io/github/actions/workflow/status/dogxie2333/zed-localize/ci.yml?branch=main&label=CI&color=black)](https://github.com/dogxie2333/zed-localize/actions/workflows/ci.yml) [![MIT](https://img.shields.io/badge/code-MIT-black)](LICENSE)

**[下载](https://github.com/dogxie2333/zed-localize/releases/latest) · [使用说明](docs/LOCAL_USAGE.md) · [问题反馈](https://github.com/dogxie2333/zed-localize/issues)**
</div>

[简体中文](README.md) · [English](docs/i18n/README.en.md) · [繁體中文](docs/i18n/README.zh-TW.md) · [日本語](docs/i18n/README.ja.md) · [한국어](docs/i18n/README.ko.md) · [Deutsch](docs/i18n/README.de.md) · [Français](docs/i18n/README.fr.md) · [Español](docs/i18n/README.es.md) · [Italiano](docs/i18n/README.it.md) · [Português (Brasil)](docs/i18n/README.pt-BR.md) · [Русский](docs/i18n/README.ru.md) · [Polski](docs/i18n/README.pl.md) · [Čeština](docs/i18n/README.cs.md) · [Türkçe](docs/i18n/README.tr.md)

## 下载

| 平台 | 下载文件 |
|---|---|
| Windows x64 | [Zed-Localize.exe](https://github.com/dogxie2333/zed-localize/releases/latest/download/Zed-Localize.exe) · [ZIP](https://github.com/dogxie2333/zed-localize/releases/latest/download/zed-localize-windows-x64.zip) |
| Ubuntu 24.04 x86_64 | [DEB](https://github.com/dogxie2333/zed-localize/releases/latest) · [tar.gz](https://github.com/dogxie2333/zed-localize/releases/latest/download/zed-localize-ubuntu24-x86_64.tar.gz) |

## 开始使用

**Windows** · `Zed-Localize.exe`

1. 选择 Zed 安装文件夹与翻译语言。
2. 选择自启或手动，保存并启动 Zed。
3. 托盘右键可打开设置、暂停或退出。

Windows 版是独立于 Zed 主程序的 DLL 注入工具，复用 Frida Gadget，在内存显示层翻译文案。不是 Zed 插件，不修改官方程序文件。

**Linux**

```sh
sudo apt install ./zed-localize_0.4.0_amd64.deb
/opt/zed-localize/zed-localize-linux --language zh-CN launch
```

Linux 包使用命令行与等长内存补丁，覆盖部分界面；暂不提供托盘和网页设置。Linux 支持范围见使用说明。 [使用说明](docs/LINUX.md)

## 功能

- **Zed 翻译语言 / 设置页语言** — 简体中文 · English · 繁體中文 · 日本語 · 한국어 · Deutsch · Français · Español · Italiano · Português (Brasil) · Русский · Polski · Čeština · Türkçe
- **启动方式** — 开机自启 / 手动启动; 显示托盘图标.
- **人工智能翻译** — DeepSeek / GLM / OpenAI; AI 翻译语言.

保留原设置、登录与插件。配置页按需打开，日常显示只查本地词典。

适用于官方 Windows Zed，自动检测安装结构并保留官方更新渠道。AI 批量生成待审译文，按语言缓存。受补丁长度限制，部分文案保留英文。

## 更新

Windows 启动时自动识别显示调用结构，不按版本号或程序哈希限制翻译。直接使用官方更新渠道；只有显示结构改变到无法识别时，才需要更新本工具。

## 许可与鸣谢

自有代码 MIT；社区译文与 Zed 衍生素材保留 GPL-3.0。 [许可与鸣谢](THIRD_PARTY_NOTICES.md)

Zed Industries · LI-NA/zed-i18n · Microsoft/vscode-loc · Frida · Smithay/wayland-rs · Vue · Element Plus · async-openai. [鸣谢](ACKNOWLEDGEMENTS.md)
