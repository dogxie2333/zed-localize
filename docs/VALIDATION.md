# 开发与检查

修改后运行相关检查：

```sh
cargo fmt --all --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all-features
node tools/test_display_locales.js
node tools/test_ui_locales.mjs
```

前端构建：在 `web/` 中运行 `npm ci` 与 `npm run build`。

Windows 使用 `packaging/build-portable.ps1` 构建单 EXE。Linux 使用 `packaging/package-linux.py` 生成 DEB 与 tar.gz。维护脚本需要 Python，用户分发包不包含 Python 运行时。

新增显示结构检测规则必须检查唯一匹配、加载代码校验、界面翻译、非界面调用拒绝和重启恢复。不得通过移除内容保护来扩大翻译范围。

测试只使用隔离资料与自建编辑器；不结束用户实例，不提交私人日志或凭据。正式发布附带校验文件、源码来源与完整许可。
