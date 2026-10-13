# Linux 使用说明

支持 Ubuntu 24.04 / x86_64。下载 DEB 或便携 tar.gz，先安装官方 Zed。

## 安装与启动

```sh
sudo apt install ./zed-localize_0.4.0_amd64.deb
/opt/zed-localize/zed-localize-linux --language zh-CN launch
```

便携包解压后直接运行目录中的 `zed-localize-linux`。以普通用户运行，不需要 root 启动。中文字体建议安装 `fonts-noto-cjk`。

如未找到 Zed，可指定官方编辑器路径：

```sh
/opt/zed-localize/zed-localize-linux   --zed "$HOME/.local/zed.app/libexec/zed-editor"   --language zh-CN launch
```

可选显示协议：

```sh
/opt/zed-localize/zed-localize-linux --display-server wayland launch
/opt/zed-localize/zed-localize-linux --display-server x11 launch
```

默认自动选择：优先 Wayland；输入设备未就绪时最多等待五秒，有 X11 时回退。手动指定 Wayland 时不会静默切换。

## 功能范围

Linux 使用等长内存补丁，翻译部分界面，保留官方程序文件与原设置。支持的目标语言与 Windows 相同，覆盖范围因语言和平台而异。当前 Linux 数据布局支持官方 Zed 1.23.2；Windows 的显示结构自动识别不适用于此后端。

Linux 暂不提供托盘、网页配置与自动翻译服务。Zed 内部重新启动后，需要从工具再次启动。官方更新后，尚未支持的数据布局会保留原文。

## 卸载

DEB：`sudo apt remove zed-localize`。便携包：删除工具目录。官方 Zed、登录、设置和插件继续保留。
