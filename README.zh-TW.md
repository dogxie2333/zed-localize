<div align="center">
<img src="docs/brand.svg" alt="Zed Localize" width="520">

跟隨官方主線的 Zed 介面本地化工具。

[![Release](https://img.shields.io/github/v/release/dogxie2333/zed-localize?color=black)](https://github.com/dogxie2333/zed-localize/releases/latest) [![CI](https://img.shields.io/github/actions/workflow/status/dogxie2333/zed-localize/ci.yml?branch=main&label=CI&color=black)](https://github.com/dogxie2333/zed-localize/actions/workflows/ci.yml) [![MIT](https://img.shields.io/badge/code-MIT-black)](LICENSE)

**[下載](https://github.com/dogxie2333/zed-localize/releases/latest) · [使用說明](docs/LOCAL_USAGE.md) · [問題回報](https://github.com/dogxie2333/zed-localize/issues)**
</div>

[简体中文](README.md) · [English](README.en.md) · [繁體中文](README.zh-TW.md) · [日本語](README.ja.md) · [한국어](README.ko.md) · [Deutsch](README.de.md) · [Français](README.fr.md) · [Español](README.es.md) · [Italiano](README.it.md) · [Português (Brasil)](README.pt-BR.md) · [Русский](README.ru.md) · [Polski](README.pl.md) · [Čeština](README.cs.md) · [Türkçe](README.tr.md)

## 下載

| 平台 | 下載檔案 |
|---|---|
| Windows x64 | [Zed-Localize.exe](https://github.com/dogxie2333/zed-localize/releases/latest/download/Zed-Localize.exe) · [ZIP](https://github.com/dogxie2333/zed-localize/releases/latest/download/zed-localize-windows-x64.zip) |
| Ubuntu 24.04 x86_64 | [DEB](https://github.com/dogxie2333/zed-localize/releases/latest) · [tar.gz](https://github.com/dogxie2333/zed-localize/releases/latest/download/zed-localize-ubuntu24-x86_64.tar.gz) |

## 開始使用

**Windows** · `Zed-Localize.exe`

1. 選擇 Zed 安裝資料夾與翻譯語言。
2. 選擇自啟或手動，儲存並啟動 Zed。
3. 系統匣右鍵可開啟設定、暫停或結束。

Windows 版是獨立於 Zed 主程式的 DLL 注入工具，復用 Frida Gadget，在記憶體顯示層翻譯文字。不是 Zed 擴充套件，不修改官方程式檔案。

**Linux**

```sh
sudo apt install ./zed-localize_0.4.0_amd64.deb
/opt/zed-localize/zed-localize-linux --language zh-TW launch
```

Linux 套件使用命令列與等長記憶體補丁，覆蓋部分介面；暫不提供系統匣和網頁設定。支援範圍見使用說明。 [使用說明](docs/LINUX.md)

## 功能

- **Zed 翻譯語言 / 設定頁語言** — 简体中文 · English · 繁體中文 · 日本語 · 한국어 · Deutsch · Français · Español · Italiano · Português (Brasil) · Русский · Polski · Čeština · Türkçe
- **啟動方式** — 開機自啟 / 手動啟動; 顯示系統匣圖示.
- **人工智慧翻譯** — DeepSeek / GLM / OpenAI; AI 翻譯語言.

保留原設定、登入與擴充套件。設定頁按需開啟，日常顯示只查本機字典。

適用於官方 Windows Zed，自動偵測安裝結構並保留官方更新管道。AI 批次產生待審譯文，依語言快取。受補丁長度限制，部分文字保留英文。

## 更新

Windows 啟動時自動辨識顯示呼叫結構，不以版本號或程式雜湊限制翻譯。照常使用官方更新；只有顯示結構無法辨識時，才需更新本工具。

## 授權與致謝

自有程式碼 MIT；社群譯文與 Zed 衍生素材保留 GPL-3.0。 [授權與致謝](THIRD_PARTY_NOTICES.md)

Zed Industries · LI-NA/zed-i18n · Microsoft/vscode-loc · Frida · Smithay/wayland-rs · Vue · Element Plus · async-openai. [致謝](ACKNOWLEDGEMENTS.md)
