# Zed Localize

[简体中文](README.md) · [English](README.en.md) · 繁體中文

為 **官方 Zed** 載入多語言介面，**跟隨官方主線，保留官方更新管道**。

**Windows 版是獨立於 Zed 主程式的 DLL 注入工具。** 啟動器將官方 Frida Gadget DLL 注入 Zed 處理程序，在記憶體顯示層替換文字，不修改磁碟上的官方 Zed.exe，也不是 Zed 擴充套件。

## 使用

在 [Releases](https://github.com/dogxie2333/zed-localize/releases) 下載 `Zed-Localize.exe` 或可攜 ZIP，放在固定、可寫的資料夾。

1. 執行工具，選擇 Zed 安裝資料夾。
2. 選擇翻譯語言、自啟或手動，儲存設定。
3. 啟動 Zed，之後照常使用原捷徑。

系統匣右鍵可開啟設定、暫停或結束。設定頁僅首次初始化或手動開啟時執行，關閉後釋放服務。原設定、登入與擴充套件繼續使用。

## 語言與 AI

復用 13 種社群譯文：簡體／繁體中文、日語、韓語、德語、法語、西班牙語、義大利語、巴西葡萄牙語、俄語、波蘭語、捷克語、土耳其語。English 恢復原文；切換後重新啟動 Zed 完整生效。

Zed 翻譯語言與設定頁語言各自切換；設定頁支援簡體、繁體與 English。受 UTF-16 長度和安全範圍限制，部分文字保留英文，歐美語言覆蓋較少。[語言統計](data/locales.json) 是安全候選數量。

可選 DeepSeek、GLM 或 OpenAI 相容介面，復用 async-openai SDK。一次選擇多種目標語言，依原始碼語境與術語批次翻譯新增文字，按語言、模型及校驗版本快取。日常顯示只讀本機詞典；AI 產生待審草稿。金鑰只保存在記憶體或自行選擇的 Windows 認證管理員。

## 更新與檔案

照常更新官方 Zed；已驗證版本在啟動與重啟時載入補丁。未知版本先使用英文，等待驗證適配。目前驗證目標為指定 Windows x64 Zed 1.23.2 包。

Rust 負責入口、系統匣與按需設定服務，網頁復用 Vue / Element Plus，不需另裝執行環境。更新工具時保留旁邊的 `zed-zh-data/`，歷史名稱用於相容既有設定。

[使用說明](docs/LOCAL_USAGE.md) · [效能](docs/PERFORMANCE.md)

## 開發與授權

構建需 Rust 1.99 GNU、Node、Python（僅維護／構建）與 MinGW：`packaging/build-portable.ps1`。

自有程式碼 **MIT**；社群譯文與 Zed 衍生素材保留 **GPL-3.0** 和上游聲明。[授權](THIRD_PARTY_NOTICES.md) · [致謝](ACKNOWLEDGEMENTS.md)

## Ubuntu 24.04 實驗版

提供 x86_64 `.deb` 與可攜 `.tar.gz`，採獨立 Rust 啟動器及受限 ptrace 等長記憶體資料補丁。指定 Zed 1.23.2 包有 664 條簡體中文候選，各語言覆蓋不同。暫不含 Linux 系統匣和網頁設定；編輯器內部重新啟動後需由工具再次啟動。

啟動前自動等待 Wayland 輸入裝置就緒，逾時且有 X11 時退回 X11。可用 `--display-server wayland` 或 `--display-server x11` 手動指定。見 [Linux 使用說明](docs/LINUX.md)。
