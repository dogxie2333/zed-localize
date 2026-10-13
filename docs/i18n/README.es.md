<div align="center">
<img src="../brand.svg" alt="Zed Localize" width="520">

Traducción de la interfaz de Zed que sigue las actualizaciones oficiales.

[![Release](https://img.shields.io/github/v/release/dogxie2333/zed-localize?color=black)](https://github.com/dogxie2333/zed-localize/releases/latest) [![CI](https://img.shields.io/github/actions/workflow/status/dogxie2333/zed-localize/ci.yml?branch=main&label=CI&color=black)](https://github.com/dogxie2333/zed-localize/actions/workflows/ci.yml) [![MIT](https://img.shields.io/badge/code-MIT-black)](../../LICENSE)

**[Descargar](https://github.com/dogxie2333/zed-localize/releases/latest) · [Guía](../LOCAL_USAGE.md) · [Informar de un problema](https://github.com/dogxie2333/zed-localize/issues)**
</div>

[简体中文](../../README.md) · [English](README.en.md) · [繁體中文](README.zh-TW.md) · [日本語](README.ja.md) · [한국어](README.ko.md) · [Deutsch](README.de.md) · [Français](README.fr.md) · [Español](README.es.md) · [Italiano](README.it.md) · [Português (Brasil)](README.pt-BR.md) · [Русский](README.ru.md) · [Polski](README.pl.md) · [Čeština](README.cs.md) · [Türkçe](README.tr.md)

## Descargar

| Plataforma | Descarga |
|---|---|
| Windows x64 | [Zed-Localize.exe](https://github.com/dogxie2333/zed-localize/releases/latest/download/Zed-Localize.exe) · [ZIP](https://github.com/dogxie2333/zed-localize/releases/latest/download/zed-localize-windows-x64.zip) |
| Ubuntu 24.04 x86_64 | [DEB](https://github.com/dogxie2333/zed-localize/releases/latest) · [tar.gz](https://github.com/dogxie2333/zed-localize/releases/latest/download/zed-localize-ubuntu24-x86_64.tar.gz) |

## Primeros pasos

**Windows** · `Zed-Localize.exe`

1. Elija la carpeta de instalación de Zed y el idioma de traducción.
2. Elija el inicio automático o manual, guarde y abra Zed.
3. Haga clic derecho en el icono de la bandeja para abrir la configuración, pausar o salir.

La edición de Windows es una herramienta independiente de inyección de DLL, separada de Zed. Utiliza Frida Gadget para traducir texto en la capa de visualización en memoria. No es una extensión de Zed y no modifica archivos oficiales.

**Linux**

```sh
sudo apt install ./zed-localize_0.4.0_amd64.deb
/opt/zed-localize/zed-localize-linux --language es-ES launch
```

Linux utiliza un lanzador por línea de comandos y parches de memoria de igual longitud para parte de la interfaz. Todavía no ofrece bandeja ni ajustes web. Consulta la guía para conocer su alcance. [Guía](../LINUX.md)

## Funciones

- **Idioma de traducción de Zed / Idioma de la configuración** — 简体中文 · English · 繁體中文 · 日本語 · 한국어 · Deutsch · Français · Español · Italiano · Português (Brasil) · Русский · Polski · Čeština · Türkçe
- **Modo de inicio** — Iniciar al acceder a Windows / Iniciar manualmente; Mostrar icono de la bandeja.
- **Traducción con IA** — DeepSeek / GLM / OpenAI; Idiomas de traducción con IA.

Conserva la configuración, la sesión y las extensiones. La página de ajustes se abre cuando se necesita; la visualización habitual solo usa diccionarios locales.

Funciona con Zed oficial para Windows, detecta automáticamente la estructura de instalación y conserva las actualizaciones oficiales. La IA genera borradores de traducción para revisión y los almacena en caché por idioma. Algunos textos permanecen en inglés por los límites de longitud de los parches.

## Actualizaciones

Windows detecta la estructura de llamadas de visualización al iniciar, sin limitar la traducción por versión o hash del ejecutable. Utiliza las actualizaciones oficiales como siempre. Actualiza esta herramienta si una estructura nueva deja de reconocerse.

## Licencia y agradecimientos

Código propio: MIT. Las traducciones comunitarias y los recursos derivados de Zed mantienen GPL-3.0. [Licencia y agradecimientos](../../THIRD_PARTY_NOTICES.md)

Zed Industries · LI-NA/zed-i18n · Microsoft/vscode-loc · Frida · Smithay/wayland-rs · Vue · Element Plus · async-openai. [Agradecimientos](../../ACKNOWLEDGEMENTS.md)
