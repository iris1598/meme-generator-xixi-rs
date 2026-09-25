//! 对联表情：角色举着写有祝福语的竖幅轻微摆动。
//!
//! 素材为 300x300、36 帧、每帧 60ms 的空白竖幅 GIF 逐帧拆图；每帧文字块的
//! 倾角与中心由「有字版 / 空白版」的像素差逐帧标定（刚体配准，见 calibration.json）。
use skia_safe::{
    AlphaType, Color, ColorType, FilterMode, Image, ImageInfo, MipmapMode, SamplingOptions,
    image::CachingHint, textlayout::TextAlign,
};

use meme_generator_core::error::Error;
use meme_generator_utils::{
    builder::InputImage,
    encoder::GifEncoder,
    image::ImageExt,
    text::Text2Image,
    text_params,
    tools::{load_image, local_date, new_paint, new_stroke_paint, new_surface},
};

use crate::{options::NoOptions, register_meme};

/// 素材帧数与帧间隔（原 GIF 为 36 帧、每帧 60ms）。
const FRAME_NUM: usize = 36;
const FRAME_DURATION: f32 = 0.06;

/// 标定所用字体：`STXingkaiBold.ttf`（字族名 `STXingKai-SC` / `华文行楷-SC`），
/// 需与内置字体一样放进 `resources/fonts`。
const FONT_FAMILIES: &[&str] = &["STXingKai-SC", "华文行楷-SC"];
/// 标定结果：字号 50、描边 2px、单列字距 51px（原图笔画中位宽约 4px，
/// 该字体 50 号无描边约 2.8px，故加 2px 描边补齐字重）。
const FONT_SIZE: f32 = 50.0;
const STROKE_WIDTH: f32 = 2.0;
const LINE_PITCH: f32 = 51.0;
/// 描边会向外扩张，防止文字图层贴边被切。
const LAYER_PAD: f32 = 4.0;

/// 每帧文字块参数：整块文字的倾角（度）与文字块（墨迹外接框）中心。
struct Frame {
    angle: f32,
    center_x: f32,
    center_y: f32,
}

const fn frame(angle: f32, center_x: f32, center_y: f32) -> Frame {
    Frame {
        angle,
        center_x,
        center_y,
    }
}

/// 上联（角色在右、左侧竖幅）
const SHANGLIAN_FRAMES: [Frame; FRAME_NUM] = [
    frame(11.00, 81.0, 201.5),
    frame(11.11, 80.5, 201.0),
    frame(11.02, 81.0, 200.5),
    frame(11.00, 81.5, 200.0),
    frame(10.99, 81.0, 199.5),
    frame(10.93, 81.5, 198.5),
    frame(10.84, 81.5, 198.0),
    frame(10.78, 81.5, 198.0),
    frame(10.82, 81.0, 197.5),
    frame(10.61, 81.0, 197.5),
    frame(10.56, 81.0, 197.5),
    frame(10.56, 81.0, 197.5),
    frame(10.52, 81.5, 198.0),
    frame(10.38, 81.0, 198.5),
    frame(10.27, 81.5, 199.0),
    frame(10.33, 81.0, 199.5),
    frame(10.26, 81.5, 200.5),
    frame(10.27, 81.5, 201.5),
    frame(10.25, 81.0, 201.5),
    frame(10.23, 81.5, 202.5),
    frame(10.32, 81.5, 203.0),
    frame(10.24, 80.5, 203.5),
    frame(10.30, 81.5, 204.5),
    frame(10.41, 81.0, 204.5),
    frame(10.45, 81.5, 205.0),
    frame(10.38, 80.5, 205.5),
    frame(10.51, 81.0, 205.5),
    frame(10.70, 81.0, 205.5),
    frame(10.72, 81.0, 205.5),
    frame(10.76, 81.0, 205.5),
    frame(10.86, 81.5, 205.0),
    frame(10.90, 81.5, 205.0),
    frame(10.99, 81.5, 204.0),
    frame(10.96, 81.0, 203.5),
    frame(11.05, 81.5, 203.0),
    frame(11.04, 81.0, 202.5),
];

/// 下联（角色在左、右侧竖幅，与上联镜像）
const XIALIAN_FRAMES: [Frame; FRAME_NUM] = [
    frame(-11.05, 212.5, 207.0),
    frame(-11.01, 212.0, 206.5),
    frame(-10.99, 212.5, 205.5),
    frame(-10.91, 212.0, 205.5),
    frame(-10.98, 212.5, 205.0),
    frame(-10.81, 212.5, 203.5),
    frame(-10.88, 212.5, 204.0),
    frame(-10.81, 212.0, 203.5),
    frame(-10.72, 212.5, 202.5),
    frame(-10.54, 212.5, 202.5),
    frame(-10.52, 212.5, 202.5),
    frame(-10.53, 212.5, 202.5),
    frame(-10.41, 213.0, 203.5),
    frame(-10.34, 212.5, 203.5),
    frame(-10.17, 213.0, 204.5),
    frame(-10.23, 212.5, 204.5),
    frame(-10.09, 213.0, 205.0),
    frame(-10.29, 212.5, 206.0),
    frame(-10.33, 212.5, 206.5),
    frame(-10.29, 213.0, 207.5),
    frame(-10.37, 212.0, 208.0),
    frame(-10.51, 213.0, 208.5),
    frame(-10.55, 213.0, 209.0),
    frame(-10.76, 212.5, 210.0),
    frame(-10.68, 213.0, 210.5),
    frame(-10.86, 213.0, 210.5),
    frame(-10.93, 212.5, 210.5),
    frame(-11.05, 212.5, 211.0),
    frame(-11.12, 212.5, 210.5),
    frame(-11.25, 212.5, 211.0),
    frame(-11.27, 213.0, 210.5),
    frame(-11.22, 212.0, 210.0),
    frame(-11.22, 212.0, 209.5),
    frame(-11.13, 212.5, 208.5),
    frame(-11.22, 212.0, 208.5),
    frame(-11.14, 212.5, 208.0),
];

/// 把整段文字排成单列竖排图层：逐字按固定字距居中堆叠，不做缩放，
/// 文字过长时会在贴图阶段自然被画面裁掉（与原素材一致）。
fn make_text_layer(text: &str) -> Result<Image, Error> {
    let chars: Vec<char> = text.chars().filter(|c| !c.is_whitespace()).collect();
    if chars.is_empty() {
        return Err(Error::MemeFeedback("还没有写字哦".to_string()));
    }
    let params = text_params!(
        text_align = TextAlign::Center,
        font_families = FONT_FAMILIES,
        paint = new_paint(Color::BLACK),
        stroke_paint = new_stroke_paint(Color::BLACK, STROKE_WIDTH),
    );
    let mut glyphs = Vec::with_capacity(chars.len());
    let mut max_width = 1.0_f32;
    let mut line_height = 1.0_f32;
    for ch in chars {
        let glyph = Text2Image::from_text(ch.to_string(), FONT_SIZE, params.clone());
        max_width = max_width.max(glyph.longest_line());
        line_height = line_height.max(glyph.height());
        glyphs.push(glyph);
    }
    let count = glyphs.len();
    let stack_height = (count - 1) as f32 * LINE_PITCH + line_height;
    let width = (max_width + 2.0 * LAYER_PAD).ceil() as i32;
    let height = (stack_height + 2.0 * LAYER_PAD).ceil() as i32;
    let mut surface = new_surface((width.max(1), height.max(1)));
    surface.canvas().clear(Color::TRANSPARENT);
    let center_y = LAYER_PAD + stack_height / 2.0;
    for (i, glyph) in glyphs.iter().enumerate() {
        let y = center_y + (i as f32 - (count as f32 - 1.0) / 2.0) * LINE_PITCH
            - glyph.height() / 2.0;
        glyph.draw_on_canvas(surface.canvas(), ((width as f32 - glyph.longest_line()) / 2.0, y));
    }
    Ok(surface.image_snapshot())
}

/// 文字图层的墨迹外接框中心，用于把图层按「墨迹中心」对齐到每帧标定位置。
fn ink_center(image: &Image) -> Result<(f32, f32), Error> {
    let info = ImageInfo::new(
        image.dimensions(),
        ColorType::RGBA8888,
        AlphaType::Unpremul,
        None,
    );
    let row_bytes = info.min_row_bytes();
    let mut data = vec![0u8; info.compute_min_byte_size()];
    if !image.read_pixels(&info, &mut data, row_bytes, (0, 0), CachingHint::Allow) {
        return Err(Error::ImageDecodeError("读取文字图层失败".to_string()));
    }
    let (mut min_x, mut min_y) = (usize::MAX, usize::MAX);
    let (mut max_x, mut max_y) = (0_usize, 0_usize);
    for y in 0..image.height() as usize {
        let row = y * row_bytes;
        for x in 0..image.width() as usize {
            if data[row + x * 4 + 3] > 8 {
                min_x = min_x.min(x);
                max_x = max_x.max(x);
                min_y = min_y.min(y);
                max_y = max_y.max(y);
            }
        }
    }
    if min_x == usize::MAX {
        return Err(Error::MemeFeedback("还没有写字哦".to_string()));
    }
    Ok((
        (min_x + max_x) as f32 / 2.0,
        (min_y + max_y) as f32 / 2.0,
    ))
}

fn render_couplet(
    template: &str,
    default_text: &str,
    frames: &[Frame; FRAME_NUM],
    text: &str,
) -> Result<Vec<u8>, Error> {
    let text = if text.trim().is_empty() {
        default_text
    } else {
        text
    };
    let text_layer = make_text_layer(text)?;
    let (ink_x, ink_y) = ink_center(&text_layer)?;
    let sampling = SamplingOptions::new(FilterMode::Linear, MipmapMode::Linear);

    let mut encoder = GifEncoder::new();
    for (i, f) in frames.iter().enumerate() {
        let frame = load_image(format!("{template}/{i}.png"))?;
        let mut surface = frame.to_surface();
        {
            let canvas = surface.canvas();
            canvas.save();
            canvas.translate((f.center_x, f.center_y));
            canvas.rotate(-f.angle, None);
            canvas.draw_image_with_sampling_options(
                &text_layer,
                (-ink_x, -ink_y),
                sampling,
                None,
            );
            canvas.restore();
        }
        encoder.add_frame(surface.image_snapshot(), FRAME_DURATION)?;
    }
    encoder.finish()
}

fn duilian_shanglian(
    _: Vec<InputImage>,
    texts: Vec<String>,
    _: NoOptions,
) -> Result<Vec<u8>, Error> {
    let text = texts.first().map(String::as_str).unwrap_or("");
    render_couplet("duilian_shanglian", "恭喜发财", &SHANGLIAN_FRAMES, text)
}

fn duilian_xialian(
    _: Vec<InputImage>,
    texts: Vec<String>,
    _: NoOptions,
) -> Result<Vec<u8>, Error> {
    let text = texts.first().map(String::as_str).unwrap_or("");
    render_couplet("duilian_xialian", "万事如意", &XIALIAN_FRAMES, text)
}

register_meme!(
    "duilian_shanglian",
    duilian_shanglian,
    min_texts = 0,
    max_texts = 1,
    default_texts = &["恭喜发财"],
    keywords = &["对联上联", "上联", "恭喜发财"],
    date_created = local_date(2026, 9, 25),
    date_modified = local_date(2026, 9, 25),
);

register_meme!(
    "duilian_xialian",
    duilian_xialian,
    min_texts = 0,
    max_texts = 1,
    default_texts = &["万事如意"],
    keywords = &["对联下联", "下联", "万事如意"],
    date_created = local_date(2026, 9, 25),
    date_modified = local_date(2026, 9, 25),
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_couplets_default_text() {
        let out = std::path::PathBuf::from(std::env::var("DUILIAN_OUT").expect("DUILIAN_OUT"));
        for (key, result) in [
            (
                "duilian_shanglian",
                duilian_shanglian(Vec::new(), Vec::new(), NoOptions {}),
            ),
            (
                "duilian_xialian",
                duilian_xialian(Vec::new(), Vec::new(), NoOptions {}),
            ),
        ] {
            let gif = result.expect("render couplet");
            std::fs::write(out.join(format!("{key}.gif")), &gif).expect("write gif");
            println!("{key}: {} bytes", gif.len());
        }
    }
}
