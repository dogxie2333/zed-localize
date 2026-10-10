# Zed 官方主线中文附件

保留官方 Zed 安装包和更新渠道，使用一个 Rust 小工具在进程内应用受限的中文补丁。AI 只参与版本更新后的批量翻译维护，编辑器启动、输入、搜索和绘制时不调用 AI。

**这是实验性的局部汉化附件，不是完整语言包。** 当前只批准一份官方 Windows x64 Zed **1.23.2** 安装包，找到 **215 条静态文案候选**。短菜单、动态文案、富文本和设置说明等尚未覆盖。官方更新后如安装包未适配，会正常打开英文版，不盲打补丁。

## 设计与复用

| 部分 | 复用项目 | 本项目负责的胶水 |
|---|---|---|
| 中文词典、文案提取、术语、语境、校验 | [LI-NA/zed-i18n](https://github.com/LI-NA/zed-i18n) | 导入已验证词典，适配官方发行包，生成待审更新 |
| Windows 进程和内存 | Microsoft Windows 官方 API、[windows-rs](https://github.com/microsoft/windows-rs) | 启动自有暂停进程、检查身份、数据写入与回滚 |
| PE 解析和扫描 | [goblin](https://github.com/m4b/goblin)、[Aho-Corasick](https://github.com/BurntSushi/aho-corasick)、memmap2 | 在 `.rdata` 查找唯一文案，控制可写范围 |
| OpenAI 兼容 API | [async-openai](https://github.com/64bit/async-openai)、reqwest | 更新时批量调用、固定提示前缀、预算与超时 |
| 持久翻译缓存 | [redb](https://github.com/cberner/redb) | 源文与语义上下文键、校验后入库 |
| 更新检查与待审 PR | GitHub Actions、GitHub CLI | 检查公开上游版本，复用社区工具，生成草稿 |

Python 原型已先验证；用户运行工具已迁移到 Rust。Python 只用于复用社区维护工具与测试夹具。没有重写社区提取器、占位符校验器、格式化翻译器、AI SDK、数据库或 DLL 注入器。

优先复用的执行要求写在 [AGENTS.md](AGENTS.md)；[agent.md](agent.md) 提供同一规则的入口。

## 使用运行版

从 Releases 获取 `zed-zh-addon.exe`。运行版不需要 Python、Frida 或 API 密钥。程序会尝试从常见安装目录和 PATH 找到 Zed；找不到时传 `--exe "你的官方 Zed.exe 路径"`。

```powershell
# 查看规划，完整文案与位置写入文件
.\zed-zh-addon.exe plan --output plan.json

# 先使用独立资料目录预览
.\zed-zh-addon.exe launch --experimental --isolated

# 保存工作并关闭已有 Zed 后，用现有用户资料目录启动
.\zed-zh-addon.exe launch --experimental

# 不启用实验后端，正常打开官方英文版
.\zed-zh-addon.exe launch

# 验证真实暂停进程中的应用、故障回滚和恢复
.\zed-zh-addon.exe test-memory --output memory-test.json

# 使用汉化启动时返回的 PID 恢复该进程，不结束进程
.\zed-zh-addon.exe restore --pid 12345
```

补丁只作用于本工具新建的暂停进程，不去汉化已有的其他 Zed 进程。原有英文 Zed 未关闭时，官方单实例转交可能使请求进入旧进程，先正常保存并关闭旧窗口。

启动器应用后退出，补丁在 Zed 进程里保留。最彻底的恢复方式是保存工作、关闭 Zed，再从官方入口重启；运行中恢复静态文字后，已缓存的布局可能需要重开窗口或重启清除。附件不会改动磁盘上的官方程序和原用户设置。

## 构建

Rust 1.90 以上。Windows x64 可使用 MSVC（需要 Visual C++ Build Tools）或 MinGW GNU 工具链。本机验证使用 GNU Rust 1.99.0；CI 另测 Windows 和 Linux 的 Rust 代码。

```powershell
# 默认构建轻量运行版，不包含 AI 客户端与数据库
cargo build --release --locked

# 维护版：启用 AI 与持久缓存
cargo build --release --locked --features ai

# 全部 Rust 测试
cargo test --locked --all-features
```

两次构建会生成同名二进制，请分别保存为 `zed-zh-addon.exe` 和 `zed-zh-maintain.exe`；Release 工作流已分别打包。维护版的网络和缓存代码只由明确的维护子命令执行，`launch` 不建立 Tokio 运行时、不检查更新、不读取 AI 缓存。

当前进程后端仅验证 Windows x64。Linux 可运行更新维护部分；没有宣称 Linux/macOS/ARM64 编辑器汉化可用。

## 更新时批量翻译

处理顺序是：**社区已有且校验通过的译文 → 本地语境缓存 → AI 批量请求 → 社区校验 → 待审草稿/PR**。

自动化每天检查官方稳定版和社区提交；未变化时只检查元数据，不编译维护工具、不克隆源码、不调用 AI。发生变化时读取公开官方源码，用社区提取器和 `prepare_translation_batches` 生成缺失文案、周围 12 行代码、控件类型、调用位置、关联设置标题/说明、现有译文和中文术语背景。

AI 输出不直接进入运行版。提取器的 `needs_review` 状态会保留在草稿；新的安装包哈希也不会被 AI 或更新脚本自动批准。翻译完成不等于二进制补丁兼容。

### 本地配置

维护工具支持 **OpenAI 兼容 Chat Completions** 接口。复制 `config.example.toml` 为已被 Git 忽略的 `config.local.toml`，填写 `api_base` 和 `model`；密钥只放环境变量 `ZED_I18N_API_KEY`。

```powershell
# 密钥通过交互输入进入当前进程环境，不写入配置或命令历史
$env:ZED_I18N_API_KEY = Read-Host 'AI API Key' -MaskInput

# 检查版本，不修改已安装的 Zed
.\zed-zh-maintain.exe check-update
```

PowerShell 5.1 没有 `-MaskInput`，请在 PowerShell 7 中运行示例，或通过系统环境变量/CI Secrets 配置。不同兼容接口可能需要调整 `json_mode`、`strict_schema`、`use_legacy_max_tokens`。`provider_prompt_cache_key` 默认关闭，确认服务支持后再开启。模型名称由你配置，没有硬编码特定供应商或“最新模型”。

首次维护需要 Python 3.12 以上及社区工具的 Tree-sitter 依赖：

```powershell
python -m pip install "tree-sitter>=0.25,<0.26" "tree-sitter-rust>=0.24,<0.25"

# 检查上游；当前版本不变时不会请求 AI
python tools/update.py --check-only

# 自动准备新增文案并调用维护工具
python tools/update.py --maintainer .\zed-zh-maintain.exe --config config.local.toml
```

也可以复用已有且干净的公开官方/社区 checkout，通过 `--toolkit` 和 `--source-root` 指定；版本必须与检测到的上游一致，不会重置用户 checkout。

若只想准备上下文、不调用 AI，运行 `python tools/update.py`，无需密钥。社区已提供全部译文时，本次任务同样不会调用 AI。

### GitHub 自动化

仓库已经包含 `.github/workflows/update-translations.yml`：每天 UTC 01:00（北京时间 09:00）检查，也可以手动运行。

在仓库 Settings → Secrets and variables → Actions 配置：

| 类型 | 名称 | 用途 |
|---|---|---|
| Secret | `AI_API_KEY` | API 密钥，仅在维护步骤读入环境 |
| Variable | `AI_API_BASE` | 兼容接口地址，未设置时使用 OpenAI 地址 |
| Variable | `AI_MODEL` | 服务支持的模型名称 |
| Variable | `AI_LEGACY_MAX_TOKENS` | 可选，填 `true` 适配仅支持 `max_tokens` 的服务 |

未配置接口时只同步社区信息、准备待审文案；不会调用收费 API。需要在 Actions 设置中允许工作流创建 pull request，自动 PR 才能打开。该流程不自动合并、不添加兼容哈希、不覆盖官方程序。

本地 redb 缓存会在 GitHub Actions 工作任务间恢复并保存，只包含已校验的公开文案译文，不保存密钥或原始 API 错误正文。

## 语境缓存

每条文案的缓存键包含：英文源文、控件类型、调用位置、去除行号的源码背景、上下文组、翻译提示/术语、语言、模型、接口、校验器指纹及策略版本。

行号、字节偏移、批次号、前后版本标记变化不会单独失效；真正的语境、术语、模型或校验规则变化会失效。缓存以文案为单位，批次重新分组也能复用；全部命中时可以不提供密钥，并且不请求 API。模型返回 `null` 的不确定项也保存为待审状态，避免反复请求同一语境。

通用编辑器背景、中文风格和术语放在固定的 system 提示前缀，变化条目放在后面的 user 消息；可选发送 `prompt_cache_key`，并统计服务返回的 `cached_tokens`。服务端是否实际缓存取决于供应商；本地缓存与其无关。参见 [OpenAI Prompt Caching](https://developers.openai.com/api/docs/guides/prompt-caching)。

维护请求有超时和每次最大批次数；成功批次逐批持久化，失败后可继续利用已完成缓存。占位符、受保护代码、键集合、值类型不合格的响应不入库；密钥回显也会被拒绝。

## 验证与实际限制

本地已经通过：

- Rust 与 Python 的 **215 条候选逐项一致**；真实官方进程写入、拒绝原文不匹配、注入失败回滚、恢复通过。
- Rust 启动器退出后，再由独立恢复进程还原 215 条；磁盘程序哈希保持不变。
- 真实社区语境生成器、真实 Rust SDK、真实社区校验器配合本地 HTTP 模拟服务：首次 1 批请求，重复任务无密钥时 0 请求；行号/版本变化命中，语境变化重译；缺失占位符及密钥回显被拒绝。

**真实外部 AI 服务尚未实测**，需要你配置接口和密钥。Mock 返回的缓存 token 数据只验证统计链路，不代表某个供应商已经产生缓存命中。

本机 GNU 运行版约 5.35 MiB；一次暖缓存观察中规划约 0.53 秒、启动约 0.91 秒。它不是普遍性能保证。完整验证与证据层级见 [docs/VALIDATION.md](docs/VALIDATION.md)。

当前用尾部空格保持 UTF-8 字节长度，只处理普通静态文案。没有完成所有 215 个候选的视觉、交互、截断、复制、无障碍或长期使用验收，也没有完成真实跨版本升级验收。未被社区清单收录的共享常量用途仍需进一步分析。

完整语言包还需要稳定的运行时本地化入口或官方扩展接口；单纯增加 AI 译文无法解决机器指令中的短文案和新的布局/ABI 变化。

## 开源与许可

项目采用 GPL-3.0，第三方源码与数据保留各自许可和来源，见 [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md)。不分发修改后的官方 Zed 或社区重新编译的 Zed。本项目与 Zed Industries 无关联，不是官方汉化插件。
