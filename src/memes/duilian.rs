//! 对联表情：角色举着写有祝福语的竖幅轻微摆动。
//!
//! 素材是 300x300、36 帧、每帧 60ms 的空白竖幅 GIF。每帧文字块的倾角、墨迹中心的横坐标
//! 与墨迹顶边的纵坐标，由「有字版 / 空白版」的像素差逐帧标定（刚体配准）；标定值再用
//! 36 帧周期的单频正弦拟合，去掉整像素量化带来的抖动（残差 0.31px ≈ 量化噪声 1/√12）。
//!
//! 排版锚点是文字块的**顶边**：字数变多时只往下延伸、由画面底边裁掉，与素材里最后一字
//! 被截断的表现一致，不会往上顶出画面。
use std::path::PathBuf;

use skia_safe::{AlphaType, Codec, Color, ColorType, Data, FilterMode, Image, ImageInfo, MipmapMode,
    SamplingOptions, image::CachingHint, textlayout::TextAlign};

use meme_generator_core::error::Error;
use meme_generator_utils::{
    builder::InputImage,
    config::IMAGES_DIR,
    decoder::CodecExt,
    encoder::GifEncoder,
    image::ImageExt,
    text::Text2Image,
    text_params,
    tools::{local_date, new_paint, new_surface},
};

use crate::{options::NoOptions, register_meme};

/// 素材帧数与帧间隔（原 GIF 为 36 帧、每帧 60ms）。
const FRAME_NUM: usize = 36;
const FRAME_DURATION: f32 = 0.06;

/// 标定所用字体：`STXingkaiBold.ttf`（字族名 `STXingKai-SC` / `华文行楷-SC`），
/// 与内置字体一样放进 `resources/fonts` 即可被加载。
const FONT_FAMILIES: &[&str] = &["STXingKai-SC", "华文行楷-SC"];
/// 标定结果：字号 50、单列字距 51px、**不加描边**。
/// 该字体 50 号不加描边时笔画中位宽 4.0px，与原图一致；再加描边会明显偏粗。
const FONT_SIZE: f32 = 50.0;
const LINE_PITCH: f32 = 51.0;
/// 文字图层四周留白。
const LAYER_PAD: f32 = 4.0;

/// 每帧文字块参数：整块文字的倾角（度）、墨迹中心的横坐标、墨迹顶边的纵坐标。
struct Frame {
    angle: f32,
    center_x: f32,
    top: f32,
}

const fn frame(angle: f32, center_x: f32, top: f32) -> Frame {
    Frame {
        angle,
        center_x,
        top,
    }
}

/// 上联（角色在右、左侧竖幅）
const SHANGLIAN_FRAMES: [Frame; FRAME_NUM] = [
    frame(11.058, 81.15, 105.43),
    frame(11.055, 81.16, 104.74),
    frame(11.039, 81.17, 104.06),
    frame(11.012, 81.18, 103.41),
    frame(10.973, 81.19, 102.83),
    frame(10.925, 81.20, 102.32),
    frame(10.868, 81.21, 101.89),
    frame(10.805, 81.22, 101.58),
    frame(10.737, 81.22, 101.37),
    frame(10.666, 81.23, 101.28),
    frame(10.594, 81.23, 101.32),
    frame(10.524, 81.23, 101.47),
    frame(10.458, 81.23, 101.74),
    frame(10.398, 81.23, 102.12),
    frame(10.345, 81.23, 102.59),
    frame(10.302, 81.23, 103.15),
    frame(10.269, 81.22, 103.77),
    frame(10.247, 81.22, 104.43),
    frame(10.238, 81.21, 105.12),
    frame(10.241, 81.20, 105.82),
    frame(10.256, 81.19, 106.50),
    frame(10.284, 81.18, 107.14),
    frame(10.322, 81.17, 107.73),
    frame(10.371, 81.16, 108.24),
    frame(10.427, 81.15, 108.66),
    frame(10.491, 81.14, 108.98),
    frame(10.559, 81.14, 109.19),
    frame(10.630, 81.13, 109.27),
    frame(10.701, 81.13, 109.24),
    frame(10.771, 81.13, 109.08),
    frame(10.837, 81.13, 108.82),
    frame(10.898, 81.13, 108.44),
    frame(10.950, 81.13, 107.97),
    frame(10.994, 81.13, 107.41),
    frame(11.027, 81.14, 106.79),
    frame(11.049, 81.15, 106.12),
];

/// 下联（角色在左、右侧竖幅，与上联镜像）
const XIALIAN_FRAMES: [Frame; FRAME_NUM] = [
    frame(-11.177, 212.28, 113.48),
    frame(-11.131, 212.28, 112.78),
    frame(-11.074, 212.28, 112.09),
    frame(-11.007, 212.29, 111.44),
    frame(-10.932, 212.31, 110.83),
    frame(-10.852, 212.34, 110.28),
    frame(-10.769, 212.37, 109.83),
    frame(-10.685, 212.40, 109.47),
    frame(-10.604, 212.44, 109.22),
    frame(-10.527, 212.48, 109.08),
    frame(-10.457, 212.53, 109.07),
    frame(-10.397, 212.57, 109.18),
    frame(-10.347, 212.61, 109.41),
    frame(-10.309, 212.65, 109.74),
    frame(-10.285, 212.69, 110.18),
    frame(-10.275, 212.72, 110.70),
    frame(-10.280, 212.74, 111.30),
    frame(-10.299, 212.76, 111.95),
    frame(-10.333, 212.77, 112.63),
    frame(-10.378, 212.78, 113.33),
    frame(-10.436, 212.77, 114.02),
    frame(-10.503, 212.76, 114.68),
    frame(-10.578, 212.74, 115.29),
    frame(-10.658, 212.72, 115.83),
    frame(-10.741, 212.69, 116.28),
    frame(-10.824, 212.65, 116.64),
    frame(-10.906, 212.62, 116.89),
    frame(-10.982, 212.57, 117.03),
    frame(-11.052, 212.53, 117.04),
    frame(-11.113, 212.49, 116.93),
    frame(-11.163, 212.45, 116.71),
    frame(-11.200, 212.41, 116.37),
    frame(-11.224, 212.37, 115.93),
    frame(-11.234, 212.34, 115.41),
    frame(-11.229, 212.31, 114.81),
    frame(-11.210, 212.30, 114.16),
];

fn asset_path(relative: &str) -> PathBuf {
    let deployed = IMAGES_DIR.join(relative);
    if deployed.is_file() {
        deployed
    } else {
        // 开发时回退到本仓库的 resources 目录
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("resources/images")
            .join(relative)
    }
}

/// 把整段文字排成单列竖排图层：逐字按固定字距居中堆叠，不做缩放。
fn make_text_layer(text: &str) -> Result<Image, Error> {
    let chars: Vec<char> = text.chars().filter(|c| !c.is_whitespace()).collect();
    if chars.is_empty() {
        return Err(Error::MemeFeedback("还没有写字哦".to_string()));
    }
    let params = text_params!(
        text_align = TextAlign::Center,
        font_families = FONT_FAMILIES,
        paint = new_paint(Color::BLACK),
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

/// 取出文字图层里所有墨迹像素的坐标。旋转后的外接框直接由这些点算，避免用
/// 包围盒四角旋转带来的偏差（墨迹并不铺满包围盒）。
fn ink_points(image: &Image) -> Result<Vec<(f32, f32)>, Error> {
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
    let mut points = Vec::new();
    for y in 0..image.height() as usize {
        let row = y * row_bytes;
        for x in 0..image.width() as usize {
            if data[row + x * 4 + 3] > 8 {
                points.push((x as f32, y as f32));
            }
        }
    }
    if points.is_empty() {
        return Err(Error::MemeFeedback("还没有写字哦".to_string()));
    }
    Ok(points)
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
    let points = ink_points(&text_layer)?;
    let sampling = SamplingOptions::new(FilterMode::Linear, MipmapMode::Linear);

    let gif_path = asset_path(&format!("{template}/0.gif"));
    if !gif_path.is_file() {
        return Err(Error::ImageAssetMissing(format!("{template}/0.gif")));
    }
    let data = Data::from_filename(&gif_path)
        .ok_or_else(|| Error::ImageDecodeError("Failed to read gif".to_string()))?;
    let mut codec = Codec::from_data(data)
        .ok_or_else(|| Error::ImageDecodeError("Failed to decode gif".to_string()))?;
    let frame_count = codec.get_frame_count().min(FRAME_NUM);

    let mut encoder = GifEncoder::new();
    for (f, i) in frames.iter().zip(0..frame_count) {
        let frame = codec.get_frame(i)?;
        // 绕图层左上角旋转（Skia 正角度为顺时针），算出旋转后墨迹外接框，
        // 再平移让「中心横坐标 = center_x、顶边 = top」。
        let phi = -f.angle.to_radians();
        let (sin_phi, cos_phi) = phi.sin_cos();
        let (mut min_x, mut max_x, mut min_y) = (f32::INFINITY, f32::NEG_INFINITY, f32::INFINITY);
        for &(x, y) in &points {
            let rx = cos_phi * x - sin_phi * y;
            let ry = sin_phi * x + cos_phi * y;
            min_x = min_x.min(rx);
            max_x = max_x.max(rx);
            min_y = min_y.min(ry);
        }
        let offset_x = f.center_x - (min_x + max_x) / 2.0;
        let offset_y = f.top - min_y;

        let mut surface = frame.to_surface();
        {
            let canvas = surface.canvas();
            canvas.save();
            canvas.translate((offset_x, offset_y));
            canvas.rotate(-f.angle, None);
            canvas.draw_image_with_sampling_options(&text_layer, (0, 0), sampling, None);
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
