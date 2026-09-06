# meme-generator-xixi

`meme-generator-rs` 的额外表情仓库，提供二十一个表情：

| Key | 关键字 | 参数 |
| --- | --- | --- |
| `Sigrika_signholding1` | 西格莉卡举牌、西西举牌、xglkjp、xglk举牌、xxjp、xx举牌 | 文本 |
| `Sigrika_signholding2` | 西格莉卡举牌2、西西举牌2、xglkjp2、xglk举牌2、xxjp2、xx举牌2 | 文本 |
| `Sigrika_signholding3` | 西格莉卡举牌3、西西举牌3、xglkjp3、xglk举牌3、xxjp3、xx举牌3 | 文本 |
| `Sigrika_signholding4` | 西格莉卡举牌4、西西举牌4、xglkjp4、xglk举牌4、xxjp4、xx举牌4 | 文本 |
| `Sigrika_signholding5` | 西格莉卡举牌5、西西举牌5、xglkjp5、xglk举牌5、xxjp5、xx举牌5 | 文本 |
| `Sigrika_signholding6` | 西格莉卡举牌6、西西举牌6、xglkjp6、xglk举牌6、xxjp6、xx举牌6 | 文本 |
| `denia_jupai_1` | 娅娅举牌、娅娅举牌1、娅娅眨眼 | 文本（可选）+ 图片（可选） |
| `denia_jupai_2` | 娅娅举牌2、娅娅红温 | 同上 |
| `denia_jupai_3` | 娅娅举牌3、娅娅开心 | 同上 |
| `denia_jupai_4` | 娅娅举牌4、娅娅悲伤 | 同上 |
| `denia_jupai_5` | 娅娅举牌5、娅娅期待 | 同上 |
| `denia_jupai_6` | 娅娅举牌6、娅娅哭哭 | 同上 |
| `am_jupai_1` | 小爱举牌、小爱举牌1、小爱眨眼 | 同上 |
| `am_jupai_2` | 小爱举牌2、小爱红温 | 同上 |
| `am_jupai_3` | 小爱举牌3、小爱开心 | 同上 |
| `am_jupai_4` | 小爱举牌4、小爱悲伤 | 同上 |
| `am_jupai_5` | 小爱举牌5、小爱期待 | 同上 |
| `am_jupai_6` | 小爱举牌6、小爱哭哭 | 同上 |
| `xixi_goldpig` | 西西摸 | 一张图片（静图或 GIF） |
| `xixi_goldpig_2` | 西西展示 | 一张图片（静图或 GIF） |
| `kurogames_iuno_say` | 尤诺说 | 文本（默认：月亮游离世间） |

`denia_jupai_1-6`（娅娅举牌）、`am_jupai_1-6`（小爱举牌）移植自 AstrBot 插件 `astrbot_plugin_denia_jupai`。人物层与牌子层分离合成（`assets/base` + `assets/sign`），文字贴在白色牌面上、随牌逐帧摆动。默认字色「粉」`#f6c4c4`，字体 `Alimama FangYuanTi VF`（阿里妈妈方圆体）。动作编号一致：1 眨眼 / 2 红温 / 3 开心 / 4 悲伤 / 5 期待 / 6 哭哭。可只给文本，也可传一张图铺满牌面（图片按 `cover` 裁进标定四边形、只落白面、牌框与手盖在上层）；有文字则图上叠字，无文字则纯图上牌。GIF 保留每帧原始时长（30/40ms 交替），共享调色板无逐帧闪色，300×300 透明背景无限循环。正文末尾支持颜色标记 `#颜色` 或旧写法 `-c 颜色`（预设名、6 位 hex、`r,g,b` 皆可；`##` 转义字面 `#`，无法识别的标记原样保留），换行按 token 像素宽度自动折行并处理行首标点禁则，超长自动缩字号、到下限仍写不下则报错，彩色 emoji 正常显示。

`Sigrika_signholding1-6`（西格莉卡举牌）移植自官方来源的 Python 版 `Sigrika_signholding`，替换了原先自制的 `xixi_holdsign_1-4`：素材（`P1-P6.gif` 与标定文件 `calibration_inner_panel.json`）原样保留，牌面按每帧标定框的位置 + 底边倾角旋转放置文本，约 30 fps。文字默认颜色 `#ffae2e`，使用 `Lolita`（萝莉体）字体，彩色 emoji 正常显示。正文末尾支持颜色标记 `#颜色` / `##颜色`（预设名或 6 位 hex，如 `#红`、`#3498db`；两者等效作用于文字，同时出现以 `#` 为准；无法识别的标记如 `C#编程` 原样保留）。换行按字符宽度自动折行，每行最多 8 个宽度单位、最多 3 行，超出报错。GIF 编码使用每帧的原始时长（30/40ms 混排），比 Python 版统一取首帧时长更还原。

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

`resources/fonts/Lolita.ttf`（萝莉体）是 `Sigrika_signholding1-6` 运行所需的字体，`resources/fonts/Alimama-FangYuanTi-VF.ttf`（阿里妈妈方圆体）是 `denia_jupai_*` / `am_jupai_*` 所需字体。`meme-generator-rs` 的字体加载路径默认是 `~/.meme_generator/resources/fonts/`（可通过 `MEME_FONTS_DIR` 环境变量覆盖）。把这些 ttf 复制到那个目录里，再加载本仓库的 cdylib 就能正常出图。举牌素材 `resources/images/sigrika_signholding/`、`resources/images/denia_jupai/` 也需一并放到 `~/.meme_generator/resources/images/` 下。

`kurogames_iuno_say`（尤诺说）使用 `FZShaoEr-M11S`（方正少儿简体），该字体随 `meme-generator-rs` 自带，无需额外安装。

## GitHub Actions

`.github/workflows/build.yml` 复用了上游 `meme-generator-contrib-rs` 的 build 流程，会在 windows / macos / linux / android 上编译 cdylib，并上传成 artifact。

## 声明

本仓库的表情素材等均来自网络，仅作学习交流使用，如有侵权请联系作者删除。
