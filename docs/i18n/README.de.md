<div align="center">
<img src="../brand.svg" alt="Zed Localize" width="520">

Oberflächenübersetzung für Zed, mit Updates aus der offiziellen Hauptentwicklung.

[![Release](https://img.shields.io/github/v/release/dogxie2333/zed-localize?color=black)](https://github.com/dogxie2333/zed-localize/releases/latest) [![CI](https://img.shields.io/github/actions/workflow/status/dogxie2333/zed-localize/ci.yml?branch=main&label=CI&color=black)](https://github.com/dogxie2333/zed-localize/actions/workflows/ci.yml) [![MIT](https://img.shields.io/badge/code-MIT-black)](../../LICENSE)

**[Download](https://github.com/dogxie2333/zed-localize/releases/latest) · [Anleitung](../LOCAL_USAGE.md) · [Probleme melden](https://github.com/dogxie2333/zed-localize/issues)**
</div>

[简体中文](../../README.md) · [English](README.en.md) · [繁體中文](README.zh-TW.md) · [日本語](README.ja.md) · [한국어](README.ko.md) · [Deutsch](README.de.md) · [Français](README.fr.md) · [Español](README.es.md) · [Italiano](README.it.md) · [Português (Brasil)](README.pt-BR.md) · [Русский](README.ru.md) · [Polski](README.pl.md) · [Čeština](README.cs.md) · [Türkçe](README.tr.md)

## Download

| Plattform | Download |
|---|---|
| Windows x64 | [Zed-Localize.exe](https://github.com/dogxie2333/zed-localize/releases/latest/download/Zed-Localize.exe) · [ZIP](https://github.com/dogxie2333/zed-localize/releases/latest/download/zed-localize-windows-x64.zip) |
| Ubuntu 24.04 x86_64 | [DEB](https://github.com/dogxie2333/zed-localize/releases/latest) · [tar.gz](https://github.com/dogxie2333/zed-localize/releases/latest/download/zed-localize-ubuntu24-x86_64.tar.gz) |

## Erste Schritte

**Windows** · `Zed-Localize.exe`

1. Wählen Sie den Installationsordner von Zed und die Übersetzungssprache.
2. Wählen Sie den automatischen oder manuellen Start, speichern Sie und öffnen Sie Zed.
3. Klicken Sie mit der rechten Maustaste auf das Symbol im Infobereich, um Einstellungen zu öffnen, zu pausieren oder zu beenden.

Die Windows-Ausgabe ist ein eigenständiges Werkzeug zur DLL-Injektion, unabhängig von Zed. Frida Gadget übersetzt Texte in der Anzeigeebene im Arbeitsspeicher. Es ist keine Zed-Erweiterung und verändert keine offiziellen Programmdateien.

**Linux**

```sh
sudo apt install ./zed-localize_0.4.0_amd64.deb
/opt/zed-localize/zed-localize-linux --language de-DE launch
```

Linux verwendet einen Kommandozeilenstarter und gleich lange Speicherpatches für Teile der Oberfläche. Systemablage und Web-Einstellungen sind dort noch nicht verfügbar. Den unterstützten Umfang beschreibt die Anleitung. [Anleitung](../LINUX.md)

## Funktionen

- **Übersetzungssprache von Zed / Sprache der Einstellungen** — 简体中文 · English · 繁體中文 · 日本語 · 한국어 · Deutsch · Français · Español · Italiano · Português (Brasil) · Русский · Polski · Čeština · Türkçe
- **Startmodus** — Bei der Anmeldung starten / Manuell starten; Symbol im Infobereich anzeigen.
- **KI-Übersetzung** — DeepSeek / GLM / OpenAI; KI-Übersetzungssprachen.

Einstellungen, Anmeldung und Erweiterungen bleiben erhalten. Die Einstellungsseite öffnet sich bei Bedarf; die laufende Anzeige verwendet nur lokale Wörterbücher.

Für das offizielle Zed für Windows. Die Installationsstruktur wird automatisch erkannt und offizielle Updates bleiben erhalten. Die KI erstellt Übersetzungsentwürfe zur Prüfung und speichert sie je Sprache zwischen. Wegen der Längenbegrenzung der Patches bleiben einige Texte auf Englisch.

## Updates

Unter Windows wird die Struktur der Anzeigeaufrufe beim Start automatisch erkannt, ohne Freigabe nach Version oder Datei-Hash. Nutzen Sie die offiziellen Updates wie gewohnt. Dieses Werkzeug muss aktualisiert werden, wenn eine geänderte Struktur nicht mehr erkannt wird.

## Lizenz und Danksagungen

Eigener Code: MIT. Übersetzungen der Community und von Zed abgeleitete Ressourcen behalten GPL-3.0. [Lizenz und Danksagungen](../../THIRD_PARTY_NOTICES.md)

Zed Industries · LI-NA/zed-i18n · Microsoft/vscode-loc · Frida · Smithay/wayland-rs · Vue · Element Plus · async-openai. [Danksagung](../../ACKNOWLEDGEMENTS.md)
