# 本地运行层

Rust GUI 启动器与快捷方式守护进程负责入口。Frida 官方 Python 绑定作为运行时桥接，保持父子进程事件会话以覆盖编辑器自身的重启/更新助手；不重写注入协议。

AI 设置页面复用 Gradio、LiteLLM 和 OpenAI SDK，仅手动打开时监听本机回环地址。密钥不写配置；用户可在页面上自行选择保存到 Windows 凭据管理器（keyring），不自动保存聊天中提供的密钥。

具体操作与适配边界见 `docs/LOCAL_USAGE.md`。
