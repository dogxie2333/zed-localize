<div align="center">
<img src="../brand.svg" alt="Zed Localize" width="520">

公式のメインラインに追従する Zed のインターフェース翻訳。

[![Release](https://img.shields.io/github/v/release/dogxie2333/zed-localize?color=black)](https://github.com/dogxie2333/zed-localize/releases/latest) [![CI](https://img.shields.io/github/actions/workflow/status/dogxie2333/zed-localize/ci.yml?branch=main&label=CI&color=black)](https://github.com/dogxie2333/zed-localize/actions/workflows/ci.yml) [![MIT](https://img.shields.io/badge/code-MIT-black)](../../LICENSE)

**[ダウンロード](https://github.com/dogxie2333/zed-localize/releases/latest) · [使い方](../LOCAL_USAGE.md) · [問題の報告](https://github.com/dogxie2333/zed-localize/issues)**
</div>

[简体中文](../../README.md) · [English](README.en.md) · [繁體中文](README.zh-TW.md) · [日本語](README.ja.md) · [한국어](README.ko.md) · [Deutsch](README.de.md) · [Français](README.fr.md) · [Español](README.es.md) · [Italiano](README.it.md) · [Português (Brasil)](README.pt-BR.md) · [Русский](README.ru.md) · [Polski](README.pl.md) · [Čeština](README.cs.md) · [Türkçe](README.tr.md)

## ダウンロード

| プラットフォーム | ダウンロード |
|---|---|
| Windows x64 | [Zed-Localize.exe](https://github.com/dogxie2333/zed-localize/releases/latest/download/Zed-Localize.exe) · [ZIP](https://github.com/dogxie2333/zed-localize/releases/latest/download/zed-localize-windows-x64.zip) |
| Ubuntu 24.04 x86_64 | [DEB](https://github.com/dogxie2333/zed-localize/releases/latest) · [tar.gz](https://github.com/dogxie2333/zed-localize/releases/latest/download/zed-localize-ubuntu24-x86_64.tar.gz) |

## 使い始める

**Windows** · `Zed-Localize.exe`

1. Zed のインストールフォルダーと翻訳言語を選択します。
2. 自動起動または手動起動を選択し、保存して Zed を起動します。
3. トレイアイコンを右クリックすると、設定、一時停止、終了を選択できます。

Windows 版は Zed 本体から独立した DLL 注入ツールです。Frida Gadget を使い、メモリ内の表示層で文字を翻訳します。Zed の拡張機能ではなく、公式プログラムのファイルを変更しません。

**Linux**

```sh
sudo apt install ./zed-localize_0.4.0_amd64.deb
/opt/zed-localize/zed-localize-linux --language ja-JP launch
```

Linux はコマンドラインと等長メモリパッチを使い、画面の一部を翻訳します。トレイと Web 設定はまだありません。対応範囲は利用ガイドを参照してください。 [使い方](../LINUX.md)

## 機能

- **Zed の翻訳言語 / 設定ページの言語** — 简体中文 · English · 繁體中文 · 日本語 · 한국어 · Deutsch · Français · Español · Italiano · Português (Brasil) · Русский · Polski · Čeština · Türkçe
- **起動方法** — サインイン時に起動 / 手動で起動; トレイアイコンを表示.
- **AI 翻訳** — DeepSeek / GLM / OpenAI; AI 翻訳の対象言語.

設定、ログイン、拡張機能を維持します。設定ページは必要なときだけ開き、通常の表示はローカル辞書だけを使用します。

公式の Windows 版 Zed に対応し、インストール構造を自動検出して公式の更新機能を維持します。AI はレビュー用の翻訳案を一括生成し、言語ごとにキャッシュします。パッチの文字数制限により、一部のテキストは英語のままです。

## 更新

Windows は起動時に表示呼び出し構造を自動検出し、バージョン番号や実行ファイルのハッシュで翻訳を制限しません。公式の更新を通常どおり利用できます。構造が認識できなくなった場合は、このツールを更新してください。

## ライセンスと謝辞

独自コードは MIT。コミュニティの翻訳と Zed 由来の素材は GPL-3.0 を維持します。 [ライセンスと謝辞](../../THIRD_PARTY_NOTICES.md)

Zed Industries · LI-NA/zed-i18n · Microsoft/vscode-loc · Frida · Smithay/wayland-rs · Vue · Element Plus · async-openai. [謝辞](../../ACKNOWLEDGEMENTS.md)
