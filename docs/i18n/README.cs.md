<div align="center">
<img src="../brand.svg" alt="Zed Localize" width="520">

Překlad rozhraní Zed se zachováním oficiálních aktualizací.

[![Release](https://img.shields.io/github/v/release/dogxie2333/zed-localize?color=black)](https://github.com/dogxie2333/zed-localize/releases/latest) [![CI](https://img.shields.io/github/actions/workflow/status/dogxie2333/zed-localize/ci.yml?branch=main&label=CI&color=black)](https://github.com/dogxie2333/zed-localize/actions/workflows/ci.yml) [![MIT](https://img.shields.io/badge/code-MIT-black)](../../LICENSE)

**[Stáhnout](https://github.com/dogxie2333/zed-localize/releases/latest) · [Návod](../LOCAL_USAGE.md) · [Nahlásit problém](https://github.com/dogxie2333/zed-localize/issues)**
</div>

[简体中文](../../README.md) · [English](README.en.md) · [繁體中文](README.zh-TW.md) · [日本語](README.ja.md) · [한국어](README.ko.md) · [Deutsch](README.de.md) · [Français](README.fr.md) · [Español](README.es.md) · [Italiano](README.it.md) · [Português (Brasil)](README.pt-BR.md) · [Русский](README.ru.md) · [Polski](README.pl.md) · [Čeština](README.cs.md) · [Türkçe](README.tr.md)

## Stáhnout

| Platforma | Stažení |
|---|---|
| Windows x64 | [Zed-Localize.exe](https://github.com/dogxie2333/zed-localize/releases/latest/download/Zed-Localize.exe) · [ZIP](https://github.com/dogxie2333/zed-localize/releases/latest/download/zed-localize-windows-x64.zip) |
| Ubuntu 24.04 x86_64 | [DEB](https://github.com/dogxie2333/zed-localize/releases/latest) · [tar.gz](https://github.com/dogxie2333/zed-localize/releases/latest/download/zed-localize-ubuntu24-x86_64.tar.gz) |

## Začínáme

**Windows** · `Zed-Localize.exe`

1. Vyberte instalační složku Zed a jazyk překladu.
2. Zvolte automatické nebo ruční spouštění, uložte nastavení a otevřete Zed.
3. Kliknutím pravým tlačítkem na ikonu v oznamovací oblasti otevřete nastavení, pozastavíte překlad nebo ukončíte aplikaci.

Verze pro Windows je samostatný nástroj pro injektování DLL, nezávislý na Zed. Frida Gadget překládá text ve vrstvě zobrazení v paměti. Nejde o rozšíření Zed a oficiální soubory programu se nemění.

**Linux**

```sh
sudo apt install ./zed-localize_0.4.0_amd64.deb
/opt/zed-localize/zed-localize-linux --language cs-CZ launch
```

Linux používá spouštěč z příkazové řádky a paměťové záplaty stejné délky pro část rozhraní. Ikona v oznamovací oblasti ani webové nastavení zatím nejsou k dispozici. Rozsah podpory uvádí návod. [Návod](../LINUX.md)

## Funkce

- **Jazyk překladu Zed / Jazyk nastavení** — 简体中文 · English · 繁體中文 · 日本語 · 한국어 · Deutsch · Français · Español · Italiano · Português (Brasil) · Русский · Polski · Čeština · Türkçe
- **Způsob spouštění** — Spouštět při přihlášení / Spouštět ručně; Zobrazit ikonu v oznamovací oblasti.
- **Překlad pomocí AI** — DeepSeek / GLM / OpenAI; Jazyky překladu AI.

Nastavení, přihlášení a rozšíření zůstávají zachovány. Stránka nastavení se otevírá na vyžádání; běžné zobrazení používá pouze místní slovníky.

Funguje s oficiálním Zed pro Windows, automaticky rozpoznává strukturu instalace a zachovává oficiální aktualizace. AI vytváří návrhy překladů ke kontrole a ukládá je do mezipaměti pro každý jazyk zvlášť. Některé texty zůstávají anglicky kvůli omezení délky oprav.

## Aktualizace

Windows při spuštění rozpoznává strukturu volání zobrazení bez omezení podle verze či otisku souboru. Používejte oficiální aktualizace jako obvykle. Tento nástroj aktualizujte, pokud novou strukturu už nelze rozpoznat.

## Licence a poděkování

Vlastní kód: MIT. Komunitní překlady a prostředky odvozené od Zed si ponechávají GPL-3.0. [Licence a poděkování](../../THIRD_PARTY_NOTICES.md)

Zed Industries · LI-NA/zed-i18n · Microsoft/vscode-loc · Frida · Smithay/wayland-rs · Vue · Element Plus · async-openai. [Poděkování](../../ACKNOWLEDGEMENTS.md)
