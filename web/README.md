# 设置页面

Vue 与 Element Plus 组件，固定左侧导航；简体中文、繁体中文、English 静态语言包。全部资源随包提供，不从 CDN 加载。HTTP/API 由 src/configuration.rs 的按需 Rust 服务提供。

npm ci && npm run build 生成 dist。发布由 packaging/build-portable.ps1 完成。不得在源码、语言包或构建产物中加入用户 API 密钥。
