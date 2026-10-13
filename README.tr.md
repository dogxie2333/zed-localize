<div align="center">
<img src="docs/brand.svg" alt="Zed Localize" width="520">

Resmî güncellemeleri takip eden Zed arayüz çevirisi.

[![Release](https://img.shields.io/github/v/release/dogxie2333/zed-localize?color=black)](https://github.com/dogxie2333/zed-localize/releases/latest) [![CI](https://img.shields.io/github/actions/workflow/status/dogxie2333/zed-localize/ci.yml?branch=main&label=CI&color=black)](https://github.com/dogxie2333/zed-localize/actions/workflows/ci.yml) [![MIT](https://img.shields.io/badge/code-MIT-black)](LICENSE)

**[İndir](https://github.com/dogxie2333/zed-localize/releases/latest) · [Kılavuz](docs/LOCAL_USAGE.md) · [Sorun bildir](https://github.com/dogxie2333/zed-localize/issues)**
</div>

[简体中文](README.md) · [English](README.en.md) · [繁體中文](README.zh-TW.md) · [日本語](README.ja.md) · [한국어](README.ko.md) · [Deutsch](README.de.md) · [Français](README.fr.md) · [Español](README.es.md) · [Italiano](README.it.md) · [Português (Brasil)](README.pt-BR.md) · [Русский](README.ru.md) · [Polski](README.pl.md) · [Čeština](README.cs.md) · [Türkçe](README.tr.md)

## İndir

| Platform | İndirme |
|---|---|
| Windows x64 | [Zed-Localize.exe](https://github.com/dogxie2333/zed-localize/releases/latest/download/Zed-Localize.exe) · [ZIP](https://github.com/dogxie2333/zed-localize/releases/latest/download/zed-localize-windows-x64.zip) |
| Ubuntu 24.04 x86_64 | [DEB](https://github.com/dogxie2333/zed-localize/releases/latest) · [tar.gz](https://github.com/dogxie2333/zed-localize/releases/latest/download/zed-localize-ubuntu24-x86_64.tar.gz) |

## Başlarken

**Windows** · `Zed-Localize.exe`

1. Zed kurulum klasörünü ve çeviri dilini seçin.
2. Otomatik veya elle başlatmayı seçin, kaydedin ve Zed’i açın.
3. Ayarları açmak, duraklatmak veya çıkmak için sistem tepsisi simgesine sağ tıklayın.

Windows sürümü, Zed’den bağımsız bir DLL enjeksiyon aracıdır. Frida Gadget ile bellekteki görüntüleme katmanında metinleri çevirir. Zed eklentisi değildir ve resmî program dosyalarını değiştirmez.

**Linux**

```sh
sudo apt install ./zed-localize_0.4.0_amd64.deb
/opt/zed-localize/zed-localize-linux --language tr-TR launch
```

Linux, arayüzün bir kısmını çevirmek için komut satırı başlatıcısı ve eşit uzunluklu bellek yamaları kullanır. Tepsi simgesi ve web ayarları henüz yoktur. Destek kapsamı için kılavuza bakın. [Kılavuz](docs/LINUX.md)

## Özellikler

- **Zed çeviri dili / Ayar sayfası dili** — 简体中文 · English · 繁體中文 · 日本語 · 한국어 · Deutsch · Français · Español · Italiano · Português (Brasil) · Русский · Polski · Čeština · Türkçe
- **Başlatma biçimi** — Oturum açıldığında başlat / Elle başlat; Sistem tepsisi simgesini göster.
- **Yapay zekâ çevirisi** — DeepSeek / GLM / OpenAI; Yapay zekâ çeviri dilleri.

Ayarlar, oturum ve eklentiler korunur. Ayarlar sayfası gerektiğinde açılır; normal görüntüleme yalnızca yerel sözlükleri kullanır.

Resmî Windows Zed sürümüyle çalışır, kurulum yapısını otomatik algılar ve resmî güncellemeleri korur. Yapay zekâ, incelenmek üzere toplu çeviri taslakları oluşturur ve bunları her dil için ayrı önbelleğe alır. Yama uzunluğu sınırları nedeniyle bazı metinler İngilizce kalır.

## Güncellemeler

Windows başlangıçta görüntüleme çağrı yapısını algılar; sürüm numarası veya dosya özetiyle çeviriyi sınırlandırmaz. Resmî güncellemeleri normal biçimde kullanın. Yeni yapı artık tanınmıyorsa bu aracı güncelleyin.

## Lisans ve teşekkürler

Özgün kod MIT lisanslıdır. Topluluk çevirileri ve Zed’den türetilen kaynaklar GPL-3.0 lisansını korur. [Lisans ve teşekkürler](THIRD_PARTY_NOTICES.md)

Zed Industries · LI-NA/zed-i18n · Microsoft/vscode-loc · Frida · Smithay/wayland-rs · Vue · Element Plus · async-openai. [Teşekkürler](ACKNOWLEDGEMENTS.md)
