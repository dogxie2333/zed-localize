# 项目结构

| 目录 | 内容 |
|---|---|
| src/ | Rust 启动器、托盘、按需服务和 AI SDK |
| web/ | Vue / Element Plus 设置页 |
| assets/locales/ | 13 种经过社区校验的离线显示词典 |
| data/ | 语言清单、公开语境任务和已验证适配 |
| tools/ | 社区提取/校验胶水与相关测试 |
| runtime/ | Windows 快捷方式、自启及恢复脚本 |
| packaging/ | Windows 便携发布构建 |
| docs/、licenses/ | 使用、验证记录和上游许可 |

下载、编译及开发依赖位于项目外 .build-cache。旧 Python 运行原型已从仓库移除；维护端继续使用社区 Python 工具。

运行数据 dist/zed-zh-data、API 缓存和本机报告不进入 Git 或 Release。保留旧目录和内部 crate 名称以兼容既有用户。
