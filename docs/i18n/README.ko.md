<div align="center">
<img src="../brand.svg" alt="Zed Localize" width="520">

공식 메인라인 업데이트를 따르는 Zed 인터페이스 번역 도구입니다.

[![Release](https://img.shields.io/github/v/release/dogxie2333/zed-localize?color=black)](https://github.com/dogxie2333/zed-localize/releases/latest) [![CI](https://img.shields.io/github/actions/workflow/status/dogxie2333/zed-localize/ci.yml?branch=main&label=CI&color=black)](https://github.com/dogxie2333/zed-localize/actions/workflows/ci.yml) [![MIT](https://img.shields.io/badge/code-MIT-black)](../../LICENSE)

**[다운로드](https://github.com/dogxie2333/zed-localize/releases/latest) · [사용 안내](../LOCAL_USAGE.md) · [문제 보고](https://github.com/dogxie2333/zed-localize/issues)**
</div>

[简体中文](../../README.md) · [English](README.en.md) · [繁體中文](README.zh-TW.md) · [日本語](README.ja.md) · [한국어](README.ko.md) · [Deutsch](README.de.md) · [Français](README.fr.md) · [Español](README.es.md) · [Italiano](README.it.md) · [Português (Brasil)](README.pt-BR.md) · [Русский](README.ru.md) · [Polski](README.pl.md) · [Čeština](README.cs.md) · [Türkçe](README.tr.md)

## 다운로드

| 플랫폼 | 다운로드 |
|---|---|
| Windows x64 | [Zed-Localize.exe](https://github.com/dogxie2333/zed-localize/releases/latest/download/Zed-Localize.exe) · [ZIP](https://github.com/dogxie2333/zed-localize/releases/latest/download/zed-localize-windows-x64.zip) |
| Ubuntu 24.04 x86_64 | [DEB](https://github.com/dogxie2333/zed-localize/releases/latest) · [tar.gz](https://github.com/dogxie2333/zed-localize/releases/latest/download/zed-localize-ubuntu24-x86_64.tar.gz) |

## 시작하기

**Windows** · `Zed-Localize.exe`

1. Zed 설치 폴더와 번역 언어를 선택합니다.
2. 자동 또는 수동 시작을 선택하고 저장한 뒤 Zed를 실행합니다.
3. 트레이 아이콘을 마우스 오른쪽 버튼으로 클릭하여 설정, 일시 중지 또는 종료를 선택합니다.

Windows 버전은 Zed 본체와 독립된 DLL 주입 도구입니다. Frida Gadget으로 메모리의 표시 계층에서 문구를 번역합니다. Zed 확장 기능이 아니며 공식 프로그램 파일을 수정하지 않습니다.

**Linux**

```sh
sudo apt install ./zed-localize_0.4.0_amd64.deb
/opt/zed-localize/zed-localize-linux --language ko-KR launch
```

Linux는 명령줄 실행기와 동일 길이 메모리 패치를 사용해 일부 화면을 번역합니다. 트레이와 웹 설정은 아직 제공되지 않습니다. 지원 범위는 사용 안내를 확인하세요. [사용 안내](../LINUX.md)

## 기능

- **Zed 번역 언어 / 설정 페이지 언어** — 简体中文 · English · 繁體中文 · 日本語 · 한국어 · Deutsch · Français · Español · Italiano · Português (Brasil) · Русский · Polski · Čeština · Türkçe
- **시작 방식** — 로그인 시 자동 시작 / 수동 시작; 트레이 아이콘 표시.
- **AI 번역** — DeepSeek / GLM / OpenAI; AI 번역 언어.

기존 설정, 로그인 및 확장을 유지합니다. 설정 페이지는 필요할 때만 열리고 일반 화면 표시는 로컬 사전만 사용합니다.

공식 Windows용 Zed에 적용되며 설치 구조를 자동 감지하고 공식 업데이트 기능을 유지합니다. AI는 검토용 번역 초안을 일괄 생성하고 언어별로 캐시합니다. 패치 길이 제한으로 일부 문구는 영어로 유지됩니다.

## 업데이트

Windows는 시작할 때 표시 호출 구조를 자동 감지하며 버전 번호나 실행 파일 해시로 번역을 제한하지 않습니다. 공식 업데이트를 그대로 사용하세요. 구조를 더 이상 인식할 수 없을 때만 이 도구를 업데이트하면 됩니다.

## 라이선스 및 감사

자체 코드는 MIT이며 커뮤니티 번역과 Zed 파생 자료는 GPL-3.0을 유지합니다. [라이선스 및 감사](../../THIRD_PARTY_NOTICES.md)

Zed Industries · LI-NA/zed-i18n · Microsoft/vscode-loc · Frida · Smithay/wayland-rs · Vue · Element Plus · async-openai. [감사의 말](../../ACKNOWLEDGEMENTS.md)
