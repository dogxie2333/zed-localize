# 验证

当前二进制适配限定 Windows x64 Zed 1.23.2：SHA-256 `d7afb758b045958838d843b95dd28c41554efd810c0f15012cf0069ff8046781`。官方磁盘程序保持原样。

13 种语言词典复用固定 LI-NA/zed-i18n 快照，通过上游占位符及受保护文本校验。Node 测试执行真实显示脚本的 14 档语言切换（含 English），核对词典数量与非法路径拒绝。Rust 检查语言持久化和安全字符串范围。

v0.3 本机通过 Rust 10 项测试、严格 Clippy、前端构建和 Node 词典切换测试。原生设置服务通过真实 async-openai SDK 的本机模拟请求：两条文案、两种语言分批翻译，重复运行四条缓存命中、无新增请求，密钥不写入配置或草稿。

真实隔离 Zed 使用日语词典启动与内部重启，分别观察到 22 和 103 条不同文案被替换。运行中切换 English、繁体、日语的词典数分别为 0、6494、6400。设置页切换简体、繁体、英文并刷新，目标语言仍保留；导航坐标未移动。最后恢复简体中文与原手动启动选择。

配置服务关闭后实测托盘工作集峰值 18.7 MiB、私有内存 2.1 MiB，单核平均 CPU 0.20%。官方 Zed 磁盘哈希保持原样。未测试的官方新版、全部控件和跨平台不视为已通过；显示引擎的输入/滚动帧耗时尚未测量。

GitHub Windows 测试曾因测试脚本默认 cp1252 读取中文 JSON 失败，已改为显式 UTF-8，并关闭 Python UTF-8 模式复测通过。CI 状态以对应提交的 Actions 为准。

后续 Windows CI 在干净的官方 sparse checkout 上出现 Dulwich 状态误判。维护端优先使用已安装 Git 的只读 status 检查，保留无 Git 时的 Dulwich 路线。新增真实 sparse-index、换行规范化、Windows 符号链接占位文件及实际修改拒绝测试；源码仍须干净。公开源码语境检查移至 Rust 构建前，便于尽早发现问题。

## v0.3.1

Windows 12 项 Rust 测试、严格 Clippy、原生 SDK/缓存测试通过。真实隔离 Zed 的显示钩子、重启与语言切换通过，新的缓存字段可读。托盘隐藏后继续运行，约 0.81 秒隐藏、0.95 秒恢复，同一 PID。PE 内的 PerMonitorV2 manifest 已验证；实际多屏系统菜单视觉仍需物理显示验收。

网页暗色下拉列表高度 274 px、内容 488 px，正常内部滚动，末尾 Türkçe 可选择。selected wrapper/placeholder 的 transform 均为 none；未编辑用户配置或凭据来执行网页测试。

Ubuntu 24.04 x86_64：官方 release 压缩包摘要与 GitHub digest 一致；简体 664、繁体 634、日语 16、德语 51 条候选通过自建进程故障回滚和还原验证。最终新增错误原文拒绝测试通过。X11/WSLg 隔离启动后工具退出、编辑器继续运行；未宣称全部 UI 视觉通过。原版 Wayland 在该 WSLg 环境同样启动异常，未对官方程序另做修补。详见 LINUX.md。

## 2026-10-11 Wayland 启动就绪修复

复用 Ubuntu wayland-info 1.2.0：冷启动 registry 没有 wl_seat；约三秒后出现 v7 seat。原版提前启动在官方 client.rs:923 崩溃，设备就绪后原版与补丁均可启动。WSLg 所用 Microsoft/weston 2318fecaeac1f1a2d5a7a042c34d931c71dae04c 的 RDP 激活路径创建 seat，与本机 Weston 键盘初始化时间一致。

启动器新增成熟 wayland-client 就绪检查和 auto/wayland/x11 选择。Linux 8 项单元测试、Windows 12 项通过；Linux Clippy 通过。真实隔离编辑器六项回归通过：冷启动自动等待后用 Wayland、强制 Wayland、强制 X11、缺失 compositor 回退、接受连接但不回应的 compositor 超时回退、强制 Wayland 超时报错且不启动编辑器。存活检查均在启动器退出后进行，测试只结束自己创建的编辑器。

冷启动自动模式约 3.28 秒；就绪后的强制 Wayland / X11 约 0.57 秒；故障回退约 5.5 秒。包括启动器规划和内存校验，不是帧耗时。本机仍选择 llvmpipe 软件渲染，未验收硬件 GPU 或完整 Ubuntu 桌面视觉。脚本：`tools/test_linux_launch.py`，需显式提供测试官方包、资源路径和隔离目录，以普通桌面用户执行。
