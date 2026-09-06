# meme-generator-xixi

`meme-generator-rs` 的额外表情仓库，提供二十一个表情：

| Key | 关键字 | 参数 |
| --- | --- | --- |
| `xixi_holdsign_1` | 西西举牌、西西举牌1 | 文本（可选） |
| `xixi_holdsign_2` | 西西举牌2 | 同上 |
| `xixi_holdsign_3` | 西西举牌3 | 同上 |
| `xixi_holdsign_4` | 西西举牌4 | 同上 |
| `xixi_holdsign_5` | 西西举牌5 | 同上 |
| `xixi_holdsign_6` | 西西举牌6 | 同上 |
| `yaya_holdsign_1` | 娅娅举牌、娅娅举牌1 | 文本（可选） |
| `yaya_holdsign_2` | 娅娅举牌2 | 同上 |
| `yaya_holdsign_3` | 娅娅举牌3 | 同上 |
| `yaya_holdsign_4` | 娅娅举牌4 | 同上 |
| `yaya_holdsign_5` | 娅娅举牌5 | 同上 |
| `yaya_holdsign_6` | 娅娅举牌6 | 同上 |
| `ams_holdsign_1` | 小爱举牌、小爱举牌1 | 同上 |
| `ams_holdsign_2` | 小爱举牌2 | 同上 |
| `ams_holdsign_3` | 小爱举牌3 | 同上 |
| `ams_holdsign_4` | 小爱举牌4 | 同上 |
| `ams_holdsign_5` | 小爱举牌5 | 同上 |
| `ams_holdsign_6` | 小爱举牌6 | 同上 |
| `xixi_goldpig` | 西西摸 | 一张图片（静图或 GIF） |
| `xixi_goldpig_2` | 西西展示 | 一张图片（静图或 GIF） |
| `kurogames_iuno_say` | 尤诺说 | 文本（默认：月亮游离世间） |

`yaya_holdsign_1-6`（娅娅举牌）、`ams_holdsign_1-6`（小爱举牌）移植自 AstrBot 插件 `astrbot_plugin_denia_jupai`。人物层与牌子层分离合成（`assets/base` + `assets/sign`），文字贴在白色牌面上、随牌逐帧摆动。默认字色「粉」`#f6c4c4`，字体 `Kingnammm Maiyuan 2`（荆南麦圆体）。动作编号一致：1 眨眼 / 2 红温 / 3 开心 / 4 悲伤 / 5 期待 / 6 哭哭。GIF 保留每帧原始时长（30/40ms 交替），共享调色板无逐帧闪色，300×300 透明背景无限循环。正文末尾支持颜色标记 `#颜色` 或旧写法 `-c 颜色`（30 色预设名、6 位 hex、`r,g,b` 皆可，全角 `＃` 等效；`#`/`##` 均可标色、同时出现以最右单个 `#` 为准；无法识别的标记原样保留；语义与西西举牌完全一致，见 `holdsign::split_color_tail`），换行与字号使用全举牌统一逻辑（`src/textfit.rs`：40 起逐档减到 18、按像素宽度折行、ASCII 词组不拆行、最多 3 行、放不下报错），彩色 emoji 正常显示。

`xixi_holdsign_1-6`（西西举牌）移植自官方来源的 Python 版 `Sigrika_signholding`，替换了原先自制的 `xixi_holdsign_1-4`：素材（`P1-P6.gif` 与标定文件 `xixi_holdsign.json`）已按新顺序重排（原 `P1` 与 `P3` 对调、标定数据同步），表情 1-6 依次对应 `P1-P6`，牌面按每帧标定框的位置 + 底边倾角旋转放置文本，约 30 fps。文字默认颜色为色表中的「橙」（`#ffae2e`，老版默认色，已替换原橙 #e67e22；如需原值可用 `#e67e22`），使用 `Kingnammm Maiyuan 2`（荆南麦圆体）字体，彩色 emoji 正常显示。正文末尾支持颜色标记 `#颜色` / `##颜色`（30 色预设名、6 位 hex 或 `r,g,b` 三元组，全角 `＃` 等效；同时出现以最右单个 `#` 为准；无法识别的标记如 `C#编程` 原样保留；亦支持旧写法 `-c 颜色`；语义与娅娅/小爱举牌完全一致，见 `holdsign::split_color_tail`）。文字安全区沿用老版逻辑：标定框宽高各内缩 12px。换行与字号使用全举牌统一逻辑（`src/textfit.rs`：40 起逐档减到 18、按像素宽度折行、ASCII 词组不拆行、最多 3 行、超出报错）。不传文字时使用各表情默认文案：1 咕噜噜–– / 2 点亮语义！ / 3 开心 / 4 悲伤 / 5 得意 / 6 哭哭。GIF 编码使用每帧的原始时长（30/40ms 混排），比 Python 版统一取首帧时长更还原。

`xixi_goldpig` / `xixi_goldpig_2` 是 16.7 fps 的 GIF 模板（分别 27 帧 / 18 帧）：模板里有一个圆形透明区域（半径 67px / 65px）用来放置传入的图片，图片按 `cover` 缩放、裁成圆形填满圆窗、保持正立并跟随各自 `centers.json` 里每帧的圆心移动，圆形区域外的手指会盖在图片上层。传入的图可以是静图，也可以是 GIF——GIF 会在圆内循环播放，与模板动画一起循环（`FrameAlign::ExtendLoop`）。

## 编译

```sh
cargo build --release
```

产物：

- Windows: `target/release/meme_generator_xixi.dll`
- Linux: `target/release/libmeme_generator_xixi.so`
- macOS: `target/release/libmeme_generator_xixi.dylib`

按 [meme-generator-rs 加载其他表情的方式](https://github.com/MemeCrafters/meme-generator-rs/wiki/%E5%8A%A0%E8%BD%BD%E5%85%B6%E4%BB%96%E8%A1%A8%E6%83%85) 加载即可。

## 字体

`resources/fonts/Kingnammm-Maiyuan.ttf`（荆南麦圆体，字族名 `Kingnammm Maiyuan 2`）是所有举牌表情（`xixi_holdsign_1-6`、`yaya_holdsign_*` / `ams_holdsign_*`）运行所需的字体。`meme-generator-rs` 的字体加载路径默认是 `~/.meme_generator/resources/fonts/`（可通过 `MEME_FONTS_DIR` 环境变量覆盖）。把这些 ttf 复制到那个目录里，再加载本仓库的 cdylib 就能正常出图。举牌素材 `resources/images/xixi_holdsign/`、`resources/images/yaya_holdsign/`、`resources/images/ams_holdsign/` 也需一并放到 `~/.meme_generator/resources/images/` 下（目录已整体改名：原 `sigrika_signholding`→`xixi_holdsign`、`denia_jupai`→`yaya_holdsign`、`am_jupai`→`ams_holdsign`；若沿用旧 `denia_jupai/` 部署请删除其中 `am_*` 文件，小爱素材现独立在 `ams_holdsign/` 且无 `am_` 前缀）。

`kurogames_iuno_say`（尤诺说）使用 `FZShaoEr-M11S`（方正少儿简体），该字体随 `meme-generator-rs` 自带，无需额外安装。

## GitHub Actions

`.github/workflows/build.yml` 复用了上游 `meme-generator-contrib-rs` 的 build 流程，会在 windows / macos / linux / android 上编译 cdylib，并上传成 artifact。

## 声明

本仓库的表情素材等均来自网络，仅作学习交流使用，如有侵权请联系作者删除。
