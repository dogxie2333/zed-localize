<div align="center">
<img src="docs/brand.svg" alt="Zed Localize" width="520">

Tradução da interface do Zed que acompanha as atualizações oficiais.

[![Release](https://img.shields.io/github/v/release/dogxie2333/zed-localize?color=black)](https://github.com/dogxie2333/zed-localize/releases/latest) [![CI](https://img.shields.io/github/actions/workflow/status/dogxie2333/zed-localize/ci.yml?branch=main&label=CI&color=black)](https://github.com/dogxie2333/zed-localize/actions/workflows/ci.yml) [![MIT](https://img.shields.io/badge/code-MIT-black)](LICENSE)

**[Baixar](https://github.com/dogxie2333/zed-localize/releases/latest) · [Guia](docs/LOCAL_USAGE.md) · [Relatar problema](https://github.com/dogxie2333/zed-localize/issues)**
</div>

[简体中文](README.md) · [English](README.en.md) · [繁體中文](README.zh-TW.md) · [日本語](README.ja.md) · [한국어](README.ko.md) · [Deutsch](README.de.md) · [Français](README.fr.md) · [Español](README.es.md) · [Italiano](README.it.md) · [Português (Brasil)](README.pt-BR.md) · [Русский](README.ru.md) · [Polski](README.pl.md) · [Čeština](README.cs.md) · [Türkçe](README.tr.md)

## Baixar

| Plataforma | Download |
|---|---|
| Windows x64 | [Zed-Localize.exe](https://github.com/dogxie2333/zed-localize/releases/latest/download/Zed-Localize.exe) · [ZIP](https://github.com/dogxie2333/zed-localize/releases/latest/download/zed-localize-windows-x64.zip) |
| Ubuntu 24.04 x86_64 | [DEB](https://github.com/dogxie2333/zed-localize/releases/latest) · [tar.gz](https://github.com/dogxie2333/zed-localize/releases/latest/download/zed-localize-ubuntu24-x86_64.tar.gz) |

## Primeiros passos

**Windows** · `Zed-Localize.exe`

1. Selecione a pasta de instalação do Zed e o idioma de tradução.
2. Escolha a inicialização automática ou manual, salve e abra o Zed.
3. Clique com o botão direito no ícone da bandeja para abrir as configurações, pausar ou sair.

A edição para Windows é uma ferramenta independente de injeção de DLL, separada do Zed. Usa Frida Gadget para traduzir textos na camada de exibição em memória. Não é uma extensão do Zed e não modifica os arquivos oficiais.

**Linux**

```sh
sudo apt install ./zed-localize_0.4.0_amd64.deb
/opt/zed-localize/zed-localize-linux --language pt-BR launch
```

No Linux, um inicializador por linha de comando e patches de memória de mesmo tamanho traduzem parte da interface. Ainda não há bandeja nem configurações web. Consulte o guia para saber o escopo suportado. [Guia](docs/LINUX.md)

## Recursos

- **Idioma de tradução do Zed / Idioma das configurações** — 简体中文 · English · 繁體中文 · 日本語 · 한국어 · Deutsch · Français · Español · Italiano · Português (Brasil) · Русский · Polski · Čeština · Türkçe
- **Modo de inicialização** — Iniciar ao entrar no Windows / Iniciar manualmente; Mostrar ícone da bandeja.
- **Tradução com IA** — DeepSeek / GLM / OpenAI; Idiomas de tradução com IA.

Preserva configurações, login e extensões. A página de configurações abre sob demanda; a exibição normal usa somente dicionários locais.

Funciona com o Zed oficial para Windows, detecta automaticamente a estrutura de instalação e mantém as atualizações oficiais. A IA gera rascunhos de tradução para revisão, armazenados em cache por idioma. Alguns textos permanecem em inglês devido aos limites de tamanho dos patches.

## Atualizações

O Windows detecta a estrutura das chamadas de exibição ao iniciar, sem limitar a tradução por versão ou hash do executável. Continue usando as atualizações oficiais. Atualize esta ferramenta se uma nova estrutura não puder ser reconhecida.

## Licença e agradecimentos

Código próprio: MIT. As traduções da comunidade e os recursos derivados do Zed mantêm GPL-3.0. [Licença e agradecimentos](THIRD_PARTY_NOTICES.md)

Zed Industries · LI-NA/zed-i18n · Microsoft/vscode-loc · Frida · Smithay/wayland-rs · Vue · Element Plus · async-openai. [Agradecimentos](ACKNOWLEDGEMENTS.md)
