<div align="center">
<img src="docs/brand.svg" alt="Zed Localize" width="520">

Traduzione dell’interfaccia di Zed che segue gli aggiornamenti ufficiali.

[![Release](https://img.shields.io/github/v/release/dogxie2333/zed-localize?color=black)](https://github.com/dogxie2333/zed-localize/releases/latest) [![CI](https://img.shields.io/github/actions/workflow/status/dogxie2333/zed-localize/ci.yml?branch=main&label=CI&color=black)](https://github.com/dogxie2333/zed-localize/actions/workflows/ci.yml) [![MIT](https://img.shields.io/badge/code-MIT-black)](LICENSE)

**[Scarica](https://github.com/dogxie2333/zed-localize/releases/latest) · [Guida](docs/LOCAL_USAGE.md) · [Segnala un problema](https://github.com/dogxie2333/zed-localize/issues)**
</div>

[简体中文](README.md) · [English](README.en.md) · [繁體中文](README.zh-TW.md) · [日本語](README.ja.md) · [한국어](README.ko.md) · [Deutsch](README.de.md) · [Français](README.fr.md) · [Español](README.es.md) · [Italiano](README.it.md) · [Português (Brasil)](README.pt-BR.md) · [Русский](README.ru.md) · [Polski](README.pl.md) · [Čeština](README.cs.md) · [Türkçe](README.tr.md)

## Scarica

| Piattaforma | Download |
|---|---|
| Windows x64 | [Zed-Localize.exe](https://github.com/dogxie2333/zed-localize/releases/latest/download/Zed-Localize.exe) · [ZIP](https://github.com/dogxie2333/zed-localize/releases/latest/download/zed-localize-windows-x64.zip) |
| Ubuntu 24.04 x86_64 | [DEB](https://github.com/dogxie2333/zed-localize/releases/latest) · [tar.gz](https://github.com/dogxie2333/zed-localize/releases/latest/download/zed-localize-ubuntu24-x86_64.tar.gz) |

## Primi passi

**Windows** · `Zed-Localize.exe`

1. Scegli la cartella di installazione di Zed e la lingua di traduzione.
2. Scegli l’avvio automatico o manuale, salva e apri Zed.
3. Fai clic con il pulsante destro sull’icona di notifica per aprire le impostazioni, mettere in pausa o uscire.

La versione Windows è uno strumento autonomo di iniezione DLL, indipendente da Zed. Usa Frida Gadget per tradurre il testo nel livello di visualizzazione in memoria. Non è un’estensione Zed e non modifica i file ufficiali.

**Linux**

```sh
sudo apt install ./zed-localize_0.4.0_amd64.deb
/opt/zed-localize/zed-localize-linux --language it-IT launch
```

Linux usa un avviatore da riga di comando e patch di memoria della stessa lunghezza per parte dell’interfaccia. Non offre ancora l’icona nell’area di notifica o le impostazioni web. Consulta la guida per l’ambito supportato. [Guida](docs/LINUX.md)

## Funzioni

- **Lingua di traduzione di Zed / Lingua delle impostazioni** — 简体中文 · English · 繁體中文 · 日本語 · 한국어 · Deutsch · Français · Español · Italiano · Português (Brasil) · Русский · Polski · Čeština · Türkçe
- **Modalità di avvio** — Avvia all’accesso a Windows / Avvia manualmente; Mostra icona di notifica.
- **Traduzione IA** — DeepSeek / GLM / OpenAI; Lingue di traduzione IA.

Impostazioni, accesso ed estensioni vengono mantenuti. La pagina delle impostazioni si apre quando serve; la visualizzazione ordinaria usa solo dizionari locali.

Funziona con Zed ufficiale per Windows, rileva automaticamente la struttura di installazione e mantiene gli aggiornamenti ufficiali. L’IA genera bozze di traduzione da revisionare, memorizzate nella cache per lingua. Alcuni testi rimangono in inglese per i limiti di lunghezza delle patch.

## Aggiornamenti

Windows riconosce la struttura delle chiamate di visualizzazione all’avvio, senza vincoli di versione o hash dell’eseguibile. Continua a usare gli aggiornamenti ufficiali. Aggiorna questo strumento se una nuova struttura non viene più riconosciuta.

## Licenza e ringraziamenti

Codice originale: MIT. Le traduzioni della comunità e le risorse derivate da Zed mantengono GPL-3.0. [Licenza e ringraziamenti](THIRD_PARTY_NOTICES.md)

Zed Industries · LI-NA/zed-i18n · Microsoft/vscode-loc · Frida · Smithay/wayland-rs · Vue · Element Plus · async-openai. [Ringraziamenti](ACKNOWLEDGEMENTS.md)
