# meme-generator-xixi

`meme-generator-rs` 的额外表情仓库，提供三个表情：

| Key | 关键字 | 参数 |
| --- | --- | --- |
| `xixi_holdsign_1` | 西西举牌、西西举牌1 | 文本（默认：咕噜噜--） |
| `xixi_holdsign_2` | 西西举牌2 | 文本（默认：点亮语乂） |
| `xixi_goldpig` | 西西摸 | 一张图片 |

举牌表情是 30 fps 的 GIF 模板，举牌会按每帧的参考位置 + 角度摆动，牌面写用户传入的文本。字体使用 `Kingnammm Maiyuan 2`（荆南麦圆体 II），文字颜色 `#f8b860`。

`xixi_goldpig` 是 27 帧 16.7 fps 的 GIF 模板：左下角的圆形透明区域（半径 67px）用来放置传入的图片，图片按 `cover` 缩放填满圆窗、保持正立并跟随 `centers.json` 里每帧的圆心移动，圆形区域外的手指会盖在图片上层。

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

`resources/fonts/Kingnammm-Maiyuan.ttf` 是表情运行所需的字体。`meme-generator-rs` 的字体加载路径默认是 `~/.meme_generator/resources/fonts/`（可通过 `MEME_FONTS_DIR` 环境变量覆盖）。把这个 ttf 复制到那个目录里，再加载本仓库的 cdylib 就能正常出图。

## GitHub Actions

`.github/workflows/build.yml` 复用了上游 `meme-generator-contrib-rs` 的 build 流程，会在 windows / macos / linux / android 上编译 cdylib，并上传成 artifact。

## 声明

本仓库的表情素材等均来自网络，仅作学习交流使用，如有侵权请联系作者删除。
