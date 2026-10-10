# 项目结构

src/ 为原生程序；web/ 为设置页；assets/ 和 data/ 为公开资源；runtime/ 仅保留 Windows 入口脚本；tools/ 为构建、维护及测试胶水；packaging/ 为单文件打包；docs/ 和 licenses/ 为说明与许可；legacy/ 仅保存旧 Python 原型，不参与运行和打包。

开发缓存迁移至项目外 D:\EMB\.build-cache\zed-zh-addon：target 编译输出、cache 下载与临时文件、legacy-build 历史打包、python-maintenance 维护环境、frontend-dependencies 前端开发依赖。均保留恢复能力，没有删除源码或用户资料。

实际使用的 dist/Zed-Zh-Addon.exe 和 dist/zed-zh-data 保留原位置，避免破坏用户入口。运行数据、缓存、报告不进入 Git 或 Release；发布 ZIP 只包含 EXE、README、许可与来源记录。
