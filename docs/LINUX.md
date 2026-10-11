# Ubuntu 24.04 / x86_64 实验版

安装官方 Zed 后，下载 `.deb` 或便携 `.tar.gz`。本工具独立于 Zed，不替换官方程序。Linux 后端使用 ptrace，只改自建进程中的安全等长 UI 文案；Windows 的 DLL 显示钩子没有移植到此版本。

## 使用

```sh
# Debian 包
sudo apt install ./zed-localize_0.3.1-alpha.1_amd64.deb

# 保留原设置、登录与插件
/opt/zed-localize/zed-localize-linux --zed "$HOME/.local/zed.app/libexec/zed-editor" --language zh-CN launch

# 查看候选，不修改进程
/opt/zed-localize/zed-localize-linux --zed "$HOME/.local/zed.app/libexec/zed-editor" plan
```

便携版解压后运行目录中的 `zed-localize-linux`；可用 `--zed` 指定官方 `zed-editor`，`--language` 指定 13 种目标语言，English 使用 `en`。资源默认从工具目录读取。Ubuntu 菜单中也有独立的 Zed Localize 入口；原 Zed 入口保留。

中文建议安装 `fonts-noto-cjk`。正常用户执行，不使用 root。WSLg 下本次官方 1.23.2 的 Wayland 启动异常，X11 对照及补丁启动通过；可用 `env -u WAYLAND_DISPLAY ... launch` 选择 X11。普通 Ubuntu Wayland 桌面尚未验收。

## 范围

当前验证指定官方 Zed 1.23.2 ELF（SHA-256 `9c732953934a4b8e3f9523401091a13ae8d5b5662c3ee38dd789fe4a2034357b`）。简体中文 664 条安全候选通过真实进程写入、故障回滚、还原测试；X11 隔离编辑器启动后保持存活，工具可退出。官方文件不变。其他语言的候选数量不同，未宣称整个界面或所有语言完成视觉验收。

未知版本使用原文，原官方更新渠道继续保留。内部重启后需重新由工具启动。当前 Linux 包提供命令行入口，尚未提供托盘、配置网页及自动适配下载；这些能力仍是 Windows 版功能。

卸载工具可直接删除便携目录或卸载 Debian 包，官方 Zed 和用户资料保留。源码 MIT，社区数据 GPL-3.0，完整声明随包提供。
