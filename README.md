# meme-generator-xixi

`meme-generator-rs` 的额外表情仓库，提供二十二个表情：

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
| `zhaoren_nongni` | 西西说 | 文本（默认：再发找人弄你） |
| `duilian_shanglian` | 西西上联、上联 | 文本（默认：恭喜发财） |
| `duilian_xialian` | 西西下联、下联 | 文本（默认：万事如意） |

`yaya_holdsign_1-6`（娅娅举牌）、`ams_holdsign_1-6`（小爱举牌）保留现有 `base` 人物 GIF，但牌子改为复用西西官方 `motion3 + moc3` 动态管线；不再读取旧 `sign` GIF 或旧标定数据。牌子样式、牌子颜色、文字颜色和文字均按每次请求实时生成，参数格式与西西举牌相同。

`xixi_holdsign_1-6`（西西举牌）使用官方 `motion3 + moc3` 网格动态合成，六个动作默认都使用牌子样式 1。参数写法为：`文字 | 牌子=1/2/3 | 牌子色=颜色 | 字色=颜色`，后三项均可省略，例如 `你好 | 牌子=2 | 牌子色=蓝 | 字色=白`。颜色支持色表名称、6 位 hex 或 `r,g,b`；旧的正文末尾 `#颜色` 写法仍表示文字颜色。牌面、文字和手部每次请求实时合成，GIF 使用官方 `30/40/30ms` 帧时序。

`xixi_goldpig` / `xixi_goldpig_2` 是 16.7 fps 的 GIF 模板（分别 27 帧 / 18 帧）：模板里有一个圆形透明区域（半径 67px / 65px）用来放置传入的图片，图片按 `cover` 缩放、裁成圆形填满圆窗、保持正立并跟随各自 `centers.json` 里每帧的圆心移动，圆形区域外的手指会盖在图片上层。传入的图可以是静图，也可以是 GIF——GIF 会在圆内循环播放，与模板动画一起循环（`FrameAlign::ExtendLoop`）。

`zhaoren_nongni`（西西说）参考尤诺说（`kurogames_iuno_say`）实现：以空白气泡图 `resources/images/zhaoren_nongni/0.jpg` 为底，文字黑色、`Kingnammm Maiyuan 2`（荆南麦圆体）字体并加同色描边加粗，通过对比原图与空白版的像素差测得文字块中心约 `(568.5, 181.5)`、倾角约 -1.2°（右高左低），在 1140×300 的未旋转文本区内自动缩字号（6 字默认文案下 208），整体旋转后贴到中心点。默认文案「再发找人弄你」，文字过长时自动缩号换行、放不下报错。

`duilian_shanglian` / `duilian_xialian`（对联上联 / 下联）把角色手持的竖幅做成可换字的表情：素材是 300x300、36 帧、每帧 60ms 的**空白竖幅**逐帧拆图，文字按标定参数逐帧实时绘制。每帧的倾角、墨迹中心横坐标与顶边纵坐标由「有字版 / 空白版」的像素差做刚体配准测得（数据见源码 `src/memes/duilian.rs` 里的 `SHANGLIAN_FRAMES` / `XIALIAN_FRAMES`）。排版锚点是文字块的**顶边**：字数变多时只往下延伸、超出画面底边被裁掉，和素材里最后一字被截断的表现一致。字号 50、单列字距 51px、不加描边（该字体 50 号无描边时笔画中位宽 4.0px，与原图一致）。上联默认文案「恭喜发财」，下联默认文案「万事如意」。

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

`resources/fonts/Kingnammm-Maiyuan.ttf`（荆南麦圆体，字族名 `Kingnammm Maiyuan 2`）是所有举牌表情运行所需的字体。举牌素材统一位于 `resources/images/holdsign/`：`person/xixi|yaya|ams/` 保存三套人物 GIF，`paizipng/` 保存三种牌面分层，`model/` 保存官方手部纹理。

`kurogames_iuno_say`（尤诺说）使用 `FZShaoEr-M11S`（方正少儿简体），该字体随 `meme-generator-rs` 自带，无需额外安装。

`resources/fonts/STXingkaiBold.ttf`（华文行楷，字族名 `STXingKai-SC`）是对联表情（`duilian_shanglian` / `duilian_xialian`）所需的字体，同样放进 `resources/fonts` 即可被加载。

## GitHub Actions

`.github/workflows/build.yml` 复用了上游 `meme-generator-contrib-rs` 的 build 流程，会在 windows / macos / linux / android 上编译 cdylib，并上传成 artifact。

## 声明

本仓库的表情素材等均来自网络，仅作学习交流使用，如有侵权请联系作者删除。
