# 中文字体目录

`fonts.rs` 会按以下优先级解析中文字体：

1. 环境变量 `MNEMO_FONT` 指向的字体文件；
2. 本目录下的 `NotoSansSC-Regular.ttf`（打包字体）；
3. 系统常见中文字体（Windows 微软雅黑/黑体、macOS 苹方、Linux Noto CJK / 文泉驿）。

## 打包字体（推荐用于发布）

为保证跨平台渲染一致，将 Noto Sans SC 放入本目录：

```text
assets/fonts/NotoSansSC-Regular.ttf
```

下载地址（SIL OFL 1.1 许可，可随程序分发）：
- https://fonts.google.com/noto/specimen/Noto+Sans+SC
- 或 https://github.com/notofonts/noto-cjk/releases

放置后无需改代码，`fonts.rs` 会自动优先加载该字体。发布时需把
`assets/fonts/NotoSansSC-Regular.ttf` 复制到可执行文件同级的 `assets/fonts/` 目录。

> 若未放置字体，程序仍可运行，但会回退到系统字体；系统字体缺失时中文显示为方框。
