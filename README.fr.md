<div align="center">
<img src="docs/brand.svg" alt="Zed Localize" width="520">

Traduction de l’interface de Zed, en suivant les mises à jour officielles.

[![Release](https://img.shields.io/github/v/release/dogxie2333/zed-localize?color=black)](https://github.com/dogxie2333/zed-localize/releases/latest) [![CI](https://img.shields.io/github/actions/workflow/status/dogxie2333/zed-localize/ci.yml?branch=main&label=CI&color=black)](https://github.com/dogxie2333/zed-localize/actions/workflows/ci.yml) [![MIT](https://img.shields.io/badge/code-MIT-black)](LICENSE)

**[Télécharger](https://github.com/dogxie2333/zed-localize/releases/latest) · [Guide](docs/LOCAL_USAGE.md) · [Signaler un problème](https://github.com/dogxie2333/zed-localize/issues)**
</div>

[简体中文](README.md) · [English](README.en.md) · [繁體中文](README.zh-TW.md) · [日本語](README.ja.md) · [한국어](README.ko.md) · [Deutsch](README.de.md) · [Français](README.fr.md) · [Español](README.es.md) · [Italiano](README.it.md) · [Português (Brasil)](README.pt-BR.md) · [Русский](README.ru.md) · [Polski](README.pl.md) · [Čeština](README.cs.md) · [Türkçe](README.tr.md)

## Télécharger

| Plateforme | Téléchargement |
|---|---|
| Windows x64 | [Zed-Localize.exe](https://github.com/dogxie2333/zed-localize/releases/latest/download/Zed-Localize.exe) · [ZIP](https://github.com/dogxie2333/zed-localize/releases/latest/download/zed-localize-windows-x64.zip) |
| Ubuntu 24.04 x86_64 | [DEB](https://github.com/dogxie2333/zed-localize/releases/latest) · [tar.gz](https://github.com/dogxie2333/zed-localize/releases/latest/download/zed-localize-ubuntu24-x86_64.tar.gz) |

## Bien démarrer

**Windows** · `Zed-Localize.exe`

1. Choisissez le dossier d’installation de Zed et la langue de traduction.
2. Choisissez le démarrage automatique ou manuel, enregistrez, puis ouvrez Zed.
3. Faites un clic droit sur l’icône de notification pour ouvrir les paramètres, mettre en pause ou quitter.

La version Windows est un outil autonome d’injection de DLL, indépendant de Zed. Frida Gadget traduit le texte dans la couche d’affichage en mémoire. Ce n’est pas une extension Zed et les fichiers officiels ne sont pas modifiés.

**Linux**

```sh
sudo apt install ./zed-localize_0.4.0_amd64.deb
/opt/zed-localize/zed-localize-linux --language fr-FR launch
```

Linux utilise un lanceur en ligne de commande et des correctifs mémoire de même longueur pour une partie de l’interface. La zone de notification et la configuration web ne sont pas encore disponibles. Consultez le guide pour connaître le périmètre pris en charge. [Guide](docs/LINUX.md)

## Fonctionnalités

- **Langue de traduction de Zed / Langue des paramètres** — 简体中文 · English · 繁體中文 · 日本語 · 한국어 · Deutsch · Français · Español · Italiano · Português (Brasil) · Русский · Polski · Čeština · Türkçe
- **Mode de démarrage** — Démarrer à la connexion / Démarrer manuellement; Afficher l’icône de notification.
- **Traduction IA** — DeepSeek / GLM / OpenAI; Langues de traduction IA.

Vos paramètres, votre connexion et vos extensions sont conservés. La page de configuration s’ouvre à la demande ; l’affichage courant utilise uniquement les dictionnaires locaux.

Compatible avec Zed officiel pour Windows : détection automatique de la structure d’installation et maintien des mises à jour officielles. L’IA génère des brouillons de traduction à vérifier, mis en cache par langue. Certains textes restent en anglais en raison des limites de longueur des correctifs.

## Mises à jour

Windows détecte la structure des appels d’affichage au démarrage, sans filtre par version ou empreinte du programme. Continuez à utiliser les mises à jour officielles. Mettez cet outil à jour si une nouvelle structure ne peut plus être reconnue.

## Licence et remerciements

Code original : MIT. Les traductions communautaires et les ressources dérivées de Zed conservent GPL-3.0. [Licence et remerciements](THIRD_PARTY_NOTICES.md)

Zed Industries · LI-NA/zed-i18n · Microsoft/vscode-loc · Frida · Smithay/wayland-rs · Vue · Element Plus · async-openai. [Remerciements](ACKNOWLEDGEMENTS.md)
