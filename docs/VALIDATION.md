# 验证状态

## 新增显示模式与设置页

本地 Windows x64 验证：社区离线词典筛选后 6499 条可匹配；此前用户确认实际显示汉化成功，菜单/设置探索曾观察 129 条不同文案。该数字不代表全部页面覆盖。

- 隔离资料目录中的两次关闭再开均观察到 23 条不同 UI 文案被替换，未结束现有用户编辑会话。
- Windows Job Object：强制结束测试控制器时，其自有暂停 cmd.exe 被内核清理；成功启动后撤销保护。
- 模拟官方更新将桌面 Zed 快捷方式重建为官方入口后，Rust 守护器自动修复为独立启动器；备份的原入口仍保留。Windows PowerShell 5.1 脚本使用 UTF-8 BOM 避免中文导致解析失败。
- Gradio 实际页面验证切换 GLM 地址、清空密钥；LiteLLM/OpenAI SDK 经本地回环 mock 完成模型查询和连接请求；配置保存未序列化密钥，未选择保存时未调用凭据写入。
- 新增运行层未知哈希测试返回英文后备；Rust 9 项测试、fmt、严格 clippy 通过。

未完成：所有界面/富文本/动态文案/无障碍验收、真实跨版本官方升级、真实 GLM 账号测试、长期运行压力测试。Frida 子进程门控已启用，但这不替代真实未来更新流程验收。当前运行控制器更新后重新挂接已有 Zed，不会终止编辑器。

一次 8 秒本机观察：Rust 守护器工作集约 4.7 MiB，Frida 控制器约 106 MiB，AI 页面约 286 MiB（不含解释器小型父进程）；这些不是通用性能保证。最终守护器进程扫描间隔已增加到不少于 5 秒，避免文件事件频繁触发扫描。

重现：`python tools/test_settings_ui.py`、`python tools/test_runtime_restart.py`、`python tools/test_runtime_guard.py`。后两者仅在有对应 Windows 本地环境时运行；设置测试使用模拟密钥，不需要真实账号。

2026-10-10，本地 Windows x64；Rust GNU 1.99.0。被测官方 Zed 1.23.2 对应源码提交 `c0199504b9cce79bc27524950022ed701bc3ebd5`。

| 范围 | 已验证 | 未代表的验收 |
|---|---|---|
| 原型迁移 | Rust / Python 的 215 条 source、translation、RVA、before、after、padding 字段完全一致 | 所有候选的视觉与交互 |
| 真实进程 | 215 条应用与恢复；原文不匹配拒绝；写入后注入故障回滚 | 其他版本或架构 |
| 生命周期 | 捕获输出调用能返回，Rust 启动器退出，4 秒后仍能从另一进程恢复 215 条 | 长期运行和用户工作流全覆盖 |
| 文件 | 真实内存测试前后官方 exe SHA256 保持不变 | 官方未来更新后仍兼容 |
| 社区工具 | 未修改的词典、语境分组、12 行源码背景与校验器 | AI 语义质量人工验收 |
| AI 协议 | 本地 HTTP mock + async-openai + 实际社区校验器；完整请求、结果验证、缓存、错误恢复 | 真实供应商、额度、费用或服务端缓存 |
| 缓存 | 行号和版本漂移命中；语境/术语/模型变化失效；缓存全命中可不提供密钥 | 任意语义变更都能自动识别 |
| 更新元数据 | 官方最新稳定标签为 v1.23.2，社区提交与固定快照一致，未发生新增文本更新 | 新发行版本的真实提取和兼容性验收 |

本地一次暖缓存观察：规划 0.527 秒，启动 0.911 秒，独立恢复 0.582 秒。运行版约 5.35 MiB，没有 Python/Frida 运行依赖。这些单次测量不构成跨机器或冷启动性能保证。

真实 DeepSeek 认证与 1 批译文已通过，选择 `/models` 实际返回的 `deepseek-flash`。译文通过社区校验，重复任务不提供密钥时为 1 条本地缓存命中、0 网络请求（约 0.02 秒）。该次供应商缓存 token 为 0。测试用凭据未保存到配置、缓存、报告、GitHub Secrets 或日志。

新增扩展补丁档：668 条纯文本候选，真实暂停进程应用/故障回滚/还原通过；启动器退出后独立恢复 668 条通过。一次发布版测量：规划约 0.57 秒、启动约 0.97 秒。没有声称所有 668 个控件已经完成视觉验收。

调试版的扩展测试曾超时，发现父进程被强制结束时会遗留暂停子进程；只清理了该测试明确创建的 PID。新增 Windows Job Object 内核保护后，“强制结束启动器清理自有暂停子进程”测试通过；正常启动释放保护，Zed 会继续存活。

CI 会运行 Rust 测试、社区桥接测试和本地 API mock，并打包 Windows 运行版/维护版；CI 状态应以 GitHub 实际运行结果为准。本记录不能替代 CI 或新版本视觉验收。

首轮 GitHub CI 已在 Windows 和 Ubuntu 成功完成；首轮手动 Update translations 工作流成功，确认当前上游没有变化，不需要 AI 请求。语境组行号 ID 与批次 target 标记另有 Rust 回归测试，避免社区分组导致无意义缓存失效。

## 重现

```powershell
cargo test --locked --all-features
cargo build --release --locked
python tools/test_native.py --binary target/release/zed-zh-addon.exe --exe "你的 Zed.exe 路径"

# 仅当本地留有原 Python 参考结果时，额外传 --python-reference 以核对逐条迁移
# 准备一个真实社区批次后运行 API mock，--python 指向维护环境的解释器
python tools/test_ai_pipeline.py --binary target/debug/zed-zh-addon.exe --jobs .cache/test-job/jobs.json --toolkit .cache/community --python python
```

API mock 需要先用 `cargo build --features ai` 构建维护工具，并用 `community_bridge.py prepare --force-source "Open Project Settings File"` 从匹配的公开官方源码生成单条测试批次。它会检查真实上游占位符规则，不会用简化正则替代生产校验器。
