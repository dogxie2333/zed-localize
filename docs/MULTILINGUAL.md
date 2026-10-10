# 多语言维护

`tools/export_locales.py --toolkit <固定社区目录>` 复用上游校验器，导出离线显示词典。`tools/community_bridge.py prepare --locale ja-JP ...` 复用上游语境分组、语言术语和批处理。

社区已有译文优先使用。`tools/update.py` 跟随官方与社区变化，为每种语言生成公开新增文案任务；AI 输出独立待审草稿。审核后的文案可重新导出，不能自动扩大已验证二进制清单。

运行配置位于 EXE 旁 `zed-zh-data/settings.json`，示例见 examples/settings.json。AI 缓存位于 draft-translations.redb，各语言草稿位于 drafts/<语言>.json；缓存按语义、语言、模型和校验策略隔离。
