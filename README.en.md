# Zed Localize

[简体中文](README.md) · English · [繁體中文](README.zh-TW.md)

Localize **official Zed** while keeping its **mainline releases and official updater**.

**The Windows edition is a standalone DLL injection tool, separate from Zed itself.** It injects the official Frida Gadget DLL into the Zed process and translates the in-memory display path. It leaves the official Zed.exe on disk unchanged and is not a Zed extension.

## Getting started

Download `Zed-Localize.exe` or the portable ZIP from [Releases](https://github.com/dogxie2333/zed-localize/releases). Keep it in a writable, permanent folder.

1. Open the tool and select the Zed installation folder.
2. Choose the Zed translation language and automatic or manual startup, then save.
3. Open Zed and keep using your existing shortcuts.

Right-click the tray icon for settings, pause or exit. Settings run only for initialization or when opened manually. Existing settings, sign-in and extensions are preserved.

## Languages and AI

Reuses 13 community catalogs: Simplified/Traditional Chinese, Japanese, Korean, German, French, Spanish, Italian, Brazilian Portuguese, Russian, Polish, Czech and Turkish. English restores original text. Restart Zed after switching to refresh all cached text.

Zed translation language and settings-page language are separate. The page supports Simplified Chinese, Traditional Chinese and English. UTF-16 length and safety restrictions leave some text in English; European-language coverage is limited. [Catalog counts](data/locales.json) describe eligible entries, not full UI coverage.

Optional DeepSeek, GLM and OpenAI-compatible APIs use async-openai. Select multiple languages to batch new public UI text with source context and language-specific terminology. Persistent caches separate languages, models and validation policies. Rendering never calls AI. Generated translations remain review drafts.

## Updates and files

Update official Zed normally. Verified binaries load the patch on launch and restart; unverified versions use English until an adapter is verified. The currently verified target is the specific Windows x64 Zed 1.23.2 package.

Rust handles startup, tray and on-demand settings; Vue / Element Plus handle the page. No extra runtime installation is required. Keep the adjacent `zed-zh-data/` when updating the tool; its legacy name preserves existing installations.

[Usage](docs/LOCAL_USAGE.md) · [Performance](docs/PERFORMANCE.md)

## Development and license

Build with Rust 1.99 GNU, Node, Python for maintenance/build only, and MinGW: `packaging/build-portable.ps1`.

Original code: **MIT**. Community translations and Zed-derived assets retain **GPL-3.0** and upstream notices. [Licenses](THIRD_PARTY_NOTICES.md) · [Acknowledgements](ACKNOWLEDGEMENTS.md)

## Ubuntu 24.04 preview

x86_64 `.deb` and portable `.tar.gz` use a standalone Rust launcher with a restricted ptrace memory-data backend. The verified Zed 1.23.2 binary has 664 Simplified Chinese candidates. Coverage differs per language. Linux does not yet include the Windows display hook, tray or web settings; relaunch through this tool after an internal editor restart. See [Linux usage](docs/LINUX.md).
