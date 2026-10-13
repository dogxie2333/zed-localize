<div align="center">
<img src="docs/brand.svg" alt="Zed Localize" width="520">

Tłumaczenie interfejsu Zed zgodne z oficjalnym cyklem aktualizacji.

[![Release](https://img.shields.io/github/v/release/dogxie2333/zed-localize?color=black)](https://github.com/dogxie2333/zed-localize/releases/latest) [![CI](https://img.shields.io/github/actions/workflow/status/dogxie2333/zed-localize/ci.yml?branch=main&label=CI&color=black)](https://github.com/dogxie2333/zed-localize/actions/workflows/ci.yml) [![MIT](https://img.shields.io/badge/code-MIT-black)](LICENSE)

**[Pobierz](https://github.com/dogxie2333/zed-localize/releases/latest) · [Instrukcja](docs/LOCAL_USAGE.md) · [Zgłoś problem](https://github.com/dogxie2333/zed-localize/issues)**
</div>

[简体中文](README.md) · [English](README.en.md) · [繁體中文](README.zh-TW.md) · [日本語](README.ja.md) · [한국어](README.ko.md) · [Deutsch](README.de.md) · [Français](README.fr.md) · [Español](README.es.md) · [Italiano](README.it.md) · [Português (Brasil)](README.pt-BR.md) · [Русский](README.ru.md) · [Polski](README.pl.md) · [Čeština](README.cs.md) · [Türkçe](README.tr.md)

## Pobierz

| Platforma | Pobieranie |
|---|---|
| Windows x64 | [Zed-Localize.exe](https://github.com/dogxie2333/zed-localize/releases/latest/download/Zed-Localize.exe) · [ZIP](https://github.com/dogxie2333/zed-localize/releases/latest/download/zed-localize-windows-x64.zip) |
| Ubuntu 24.04 x86_64 | [DEB](https://github.com/dogxie2333/zed-localize/releases/latest) · [tar.gz](https://github.com/dogxie2333/zed-localize/releases/latest/download/zed-localize-ubuntu24-x86_64.tar.gz) |

## Pierwsze kroki

**Windows** · `Zed-Localize.exe`

1. Wybierz folder instalacyjny Zed i język tłumaczenia.
2. Wybierz uruchamianie automatyczne lub ręczne, zapisz ustawienia i otwórz Zed.
3. Kliknij prawym przyciskiem myszy ikonę w zasobniku, aby otworzyć ustawienia, wstrzymać tłumaczenie lub zakończyć działanie.

Wersja Windows jest niezależnym narzędziem do wstrzykiwania DLL, działającym obok Zed. Frida Gadget tłumaczy tekst w warstwie wyświetlania w pamięci. Nie jest to rozszerzenie Zed i nie zmienia oficjalnych plików programu.

**Linux**

```sh
sudo apt install ./zed-localize_0.4.0_amd64.deb
/opt/zed-localize/zed-localize-linux --language pl-PL launch
```

Linux korzysta z programu uruchamianego w terminalu i poprawek pamięci o tej samej długości dla części interfejsu. Ikona zasobnika i ustawienia webowe nie są jeszcze dostępne. Zakres obsługi opisuje instrukcja. [Instrukcja](docs/LINUX.md)

## Funkcje

- **Język tłumaczenia Zed / Język ustawień** — 简体中文 · English · 繁體中文 · 日本語 · 한국어 · Deutsch · Français · Español · Italiano · Português (Brasil) · Русский · Polski · Čeština · Türkçe
- **Tryb uruchamiania** — Uruchamiaj przy logowaniu / Uruchamiaj ręcznie; Pokaż ikonę w zasobniku.
- **Tłumaczenie AI** — DeepSeek / GLM / OpenAI; Języki tłumaczenia AI.

Ustawienia, logowanie i rozszerzenia pozostają zachowane. Strona ustawień otwiera się na żądanie, a zwykłe wyświetlanie korzysta wyłącznie z lokalnych słowników.

Działa z oficjalnym Zed dla Windows, automatycznie wykrywa strukturę instalacji i zachowuje oficjalne aktualizacje. AI tworzy szkice tłumaczeń do sprawdzenia i zapisuje je w pamięci podręcznej osobno dla każdego języka. Niektóre teksty pozostają po angielsku ze względu na ograniczenia długości poprawek.

## Aktualizacje

Windows wykrywa strukturę wywołań wyświetlania podczas uruchamiania, bez blokowania według wersji lub skrótu pliku. Korzystaj z oficjalnych aktualizacji jak zwykle. Zaktualizuj narzędzie, jeśli nowa struktura przestanie być rozpoznawana.

## Licencja i podziękowania

Własny kod: MIT. Tłumaczenia społeczności i zasoby pochodzące z Zed zachowują GPL-3.0. [Licencja i podziękowania](THIRD_PARTY_NOTICES.md)

Zed Industries · LI-NA/zed-i18n · Microsoft/vscode-loc · Frida · Smithay/wayland-rs · Vue · Element Plus · async-openai. [Podziękowania](ACKNOWLEDGEMENTS.md)
