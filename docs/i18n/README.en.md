<div align="center">
<img src="../brand.svg" alt="Zed Localize" width="520">

Interface localization for Zed, following the official mainline.

[![Release](https://img.shields.io/github/v/release/dogxie2333/zed-localize?color=black)](https://github.com/dogxie2333/zed-localize/releases/latest) [![CI](https://img.shields.io/github/actions/workflow/status/dogxie2333/zed-localize/ci.yml?branch=main&label=CI&color=black)](https://github.com/dogxie2333/zed-localize/actions/workflows/ci.yml) [![MIT](https://img.shields.io/badge/code-MIT-black)](../../LICENSE)

**[Download](https://github.com/dogxie2333/zed-localize/releases/latest) · [Guide](../LOCAL_USAGE.md) · [Issues](https://github.com/dogxie2333/zed-localize/issues)**
</div>

[简体中文](../../README.md) · [English](README.en.md) · [繁體中文](README.zh-TW.md) · [日本語](README.ja.md) · [한국어](README.ko.md) · [Deutsch](README.de.md) · [Français](README.fr.md) · [Español](README.es.md) · [Italiano](README.it.md) · [Português (Brasil)](README.pt-BR.md) · [Русский](README.ru.md) · [Polski](README.pl.md) · [Čeština](README.cs.md) · [Türkçe](README.tr.md)

## Download

| Platform | Download |
|---|---|
| Windows x64 | [Zed-Localize.exe](https://github.com/dogxie2333/zed-localize/releases/latest/download/Zed-Localize.exe) · [ZIP](https://github.com/dogxie2333/zed-localize/releases/latest/download/zed-localize-windows-x64.zip) |
| Ubuntu 24.04 x86_64 | [DEB](https://github.com/dogxie2333/zed-localize/releases/latest) · [tar.gz](https://github.com/dogxie2333/zed-localize/releases/latest/download/zed-localize-ubuntu24-x86_64.tar.gz) |

## Quick start

**Windows** · `Zed-Localize.exe`

1. Select the Zed folder and translation language.
2. Choose automatic or manual startup, save, then open Zed.
3. Right-click the tray icon for settings, pause or exit.

The Windows edition is a standalone DLL injection tool, independent of Zed. It uses Frida Gadget to translate text in the display layer in memory. It is not a Zed extension and does not modify official program files.

**Linux**

```sh
sudo apt install ./zed-localize_0.4.0_amd64.deb
/opt/zed-localize/zed-localize-linux --language en launch
```

Linux uses a command-line launcher and equal-length memory patches with partial interface coverage. Tray and web settings are not available on Linux. See the usage guide for its supported scope. [Guide](../LINUX.md)

## Features

- **Zed translation language / Settings language** — 简体中文 · English · 繁體中文 · 日本語 · 한국어 · Deutsch · Français · Español · Italiano · Português (Brasil) · Русский · Polski · Čeština · Türkçe
- **Startup mode** — Start at sign-in / Start manually; Show tray icon.
- **AI translation** — DeepSeek / GLM / OpenAI; AI translation languages.

Keep your settings, sign-in and extensions. Settings open on demand; everyday rendering uses local dictionaries only.

Works with official Zed for Windows, automatically detects the installation structure and keeps official updates. AI generates translation drafts for review, cached per language. Some text remains English because of patch length limits.

## Updates

Windows detects the display call structure at startup, without version or binary-hash gates. Use official Zed updates as usual; update this tool when a changed display structure can no longer be recognized.

## License & acknowledgements

Original code: MIT. Community translations and Zed-derived assets retain GPL-3.0. [License & acknowledgements](../../THIRD_PARTY_NOTICES.md)

Zed Industries · LI-NA/zed-i18n · Microsoft/vscode-loc · Frida · Smithay/wayland-rs · Vue · Element Plus · async-openai. [Acknowledgements](../../ACKNOWLEDGEMENTS.md)
