# 黑白图标

图案依据官方 Zed 公开应用图标中的折线结构重绘：去除底板、渐变和最内侧细节，保留斜向主线与两侧嵌套折线。该图标表示独立社区汉化补丁。

- mark-light.svg / mark-dark.svg：可编辑矢量图，黑线 / 白线，透明背景。
- app.ico：16、20、24、32、48、64、128、256 像素；透明背景，细白描边提高桌面背景上的可见性。
- tray-*.rgba：系统浅色任务栏用黑线，深色任务栏用白线。
- 状态标记：空心圆待命，实心圆已启用，双竖线暂停，感叹号未适配，叉号监控异常。托盘文字提示提供完整状态。
- PNG 与 preview.png 是构建工具生成的本地预览，不提交到仓库。

构建工具 tools/build_icons.py 复用 Pillow，只在构建时运行；托盘直接读取嵌入的 RGBA，不引入图像解码器或常驻 Python。

原始参考：official/crates/zed/resources/app-icon@2x.png，公开源仓库 zed-industries/zed，参考检出 c0199504b9cce79bc27524950022ed701bc3ebd5。图标资源保留 GPL-3.0；自有程序代码 MIT；图案不表示由 Zed 官方制作或支持。
