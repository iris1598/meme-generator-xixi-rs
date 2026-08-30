# meme-generator-xixi

`meme-generator-rs` 的额外表情仓库，提供两个表情：

| Key | 关键字 | 默认文本 |
| --- | --- | --- |
| `xixi_holdsign_1` | 西西举牌、西西举牌1 | 咕噜噜-- |
| `xixi_holdsign_2` | 西西举牌2 | 点亮语乂 |

两个表情都是 30 fps 的 GIF 模板，举牌会按每帧的参考位置 + 角度摆动，牌面写用户传入的文本。字体使用 `Kingnammm Maiyuan 2`（荆南麦圆体 II），文字颜色 `#f8b860`。

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
