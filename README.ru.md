<div align="center">
<img src="docs/brand.svg" alt="Zed Localize" width="520">

Перевод интерфейса Zed с сохранением официальных обновлений.

[![Release](https://img.shields.io/github/v/release/dogxie2333/zed-localize?color=black)](https://github.com/dogxie2333/zed-localize/releases/latest) [![CI](https://img.shields.io/github/actions/workflow/status/dogxie2333/zed-localize/ci.yml?branch=main&label=CI&color=black)](https://github.com/dogxie2333/zed-localize/actions/workflows/ci.yml) [![MIT](https://img.shields.io/badge/code-MIT-black)](LICENSE)

**[Скачать](https://github.com/dogxie2333/zed-localize/releases/latest) · [Руководство](docs/LOCAL_USAGE.md) · [Сообщить о проблеме](https://github.com/dogxie2333/zed-localize/issues)**
</div>

[简体中文](README.md) · [English](README.en.md) · [繁體中文](README.zh-TW.md) · [日本語](README.ja.md) · [한국어](README.ko.md) · [Deutsch](README.de.md) · [Français](README.fr.md) · [Español](README.es.md) · [Italiano](README.it.md) · [Português (Brasil)](README.pt-BR.md) · [Русский](README.ru.md) · [Polski](README.pl.md) · [Čeština](README.cs.md) · [Türkçe](README.tr.md)

## Скачать

| Платформа | Скачать |
|---|---|
| Windows x64 | [Zed-Localize.exe](https://github.com/dogxie2333/zed-localize/releases/latest/download/Zed-Localize.exe) · [ZIP](https://github.com/dogxie2333/zed-localize/releases/latest/download/zed-localize-windows-x64.zip) |
| Ubuntu 24.04 x86_64 | [DEB](https://github.com/dogxie2333/zed-localize/releases/latest) · [tar.gz](https://github.com/dogxie2333/zed-localize/releases/latest/download/zed-localize-ubuntu24-x86_64.tar.gz) |

## Начало работы

**Windows** · `Zed-Localize.exe`

1. Выберите папку установки Zed и язык перевода.
2. Выберите автоматический или ручной запуск, сохраните настройки и откройте Zed.
3. Нажмите правой кнопкой мыши на значок в трее, чтобы открыть настройки, приостановить перевод или выйти.

Версия для Windows — самостоятельный инструмент внедрения DLL, независимый от Zed. Frida Gadget переводит текст в слое отображения в памяти. Это не расширение Zed, и официальные файлы программы не изменяются.

**Linux**

```sh
sudo apt install ./zed-localize_0.4.0_amd64.deb
/opt/zed-localize/zed-localize-linux --language ru-RU launch
```

Linux использует запуск из командной строки и патчи памяти одинаковой длины для части интерфейса. Значок в трее и веб-настройки пока недоступны. Поддерживаемые возможности описаны в руководстве. [Руководство](docs/LINUX.md)

## Возможности

- **Язык перевода Zed / Язык настроек** — 简体中文 · English · 繁體中文 · 日本語 · 한국어 · Deutsch · Français · Español · Italiano · Português (Brasil) · Русский · Polski · Čeština · Türkçe
- **Режим запуска** — Запускать при входе в Windows / Запускать вручную; Показывать значок в трее.
- **Перевод с ИИ** — DeepSeek / GLM / OpenAI; Языки перевода с ИИ.

Настройки, вход в аккаунт и расширения сохраняются. Страница настроек открывается по запросу; обычное отображение использует только локальные словари.

Работает с официальным Zed для Windows, автоматически определяет структуру установки и сохраняет официальные обновления. ИИ создаёт черновики перевода для проверки и кэширует их отдельно для каждого языка. Из-за ограничений длины патчей некоторые тексты остаются на английском.

## Обновления

В Windows структура вызовов отображения определяется при запуске без ограничения по версии или хешу программы. Используйте официальные обновления как обычно. Обновите этот инструмент, если новая структура перестанет распознаваться.

## Лицензия и благодарности

Собственный код: MIT. Переводы сообщества и материалы на основе Zed сохраняют GPL-3.0. [Лицензия и благодарности](THIRD_PARTY_NOTICES.md)

Zed Industries · LI-NA/zed-i18n · Microsoft/vscode-loc · Frida · Smithay/wayland-rs · Vue · Element Plus · async-openai. [Благодарности](ACKNOWLEDGEMENTS.md)
