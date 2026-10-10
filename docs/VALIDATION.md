# 验证状态

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

真实外部 API 没有被调用；测试中的 `cached_tokens` 是 mock 数据，用来验证 SDK 的统计字段读取。真实接口需要配置地址、模型、密钥后另行验证。

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
