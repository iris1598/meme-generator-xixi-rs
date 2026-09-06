use std::{collections::HashMap, sync::LazyLock};

use serde::Deserialize;
use skia_safe::{Color, Data, textlayout::TextAlign};
use unicode_properties::{GeneralCategory, UnicodeGeneralCategory};
use unicode_width::UnicodeWidthChar;

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

const CALIBRATION_JSON: &str = include_str!("../data/calibration_inner_panel.json");

const DEFAULT_TEXT_COLOR: Color = Color::from_rgb(255, 174, 46);
const MAX_UNITS_PER_LINE: f32 = 8.0;
const MAX_LINES: usize = 3;
const MIN_FONT_SIZE: f32 = 10.0;
const MAX_FONT_SIZE: f32 = 44.0;
const FONT_FAMILIES: &str = "Lolita";
const FALLBACK_FRAME_DURATION: f32 = 0.03;

#[derive(Deserialize)]
struct Calibration {
    templates: HashMap<String, Template>,
}

#[derive(Deserialize)]
struct Template {
    frames: Vec<FrameRect>,
}

#[derive(Deserialize)]
struct FrameRect {
    corners: [[f32; 2]; 4],
    center: [f32; 2],
    width: f32,
    height: f32,
}

static CALIBRATION: LazyLock<Calibration> = LazyLock::new(|| {
    serde_json::from_str(CALIBRATION_JSON).expect("invalid calibration_inner_panel.json")
});

// ---- 文字颜色：正文末尾「#预设名」或「#rrggbb」指定文字颜色，与 Python 版一致 ----

const COLOR_PRESETS: [(&str, &str); 26] = [
    ("红", "#e74c3c"), ("深红", "#c0392b"), ("浅红", "#ff6b6b"),
    ("橙", "#e67e22"), ("深橙", "#d35400"), ("浅橙", "#f39c12"),
    ("黄", "#f1c40f"), ("深黄", "#d4ac0d"), ("浅黄", "#fff176"),
    ("绿", "#2ecc71"), ("深绿", "#27ae60"), ("浅绿", "#82e0aa"),
    ("青", "#1abc9c"), ("深青", "#16a085"), ("浅青", "#7fffd4"),
    ("蓝", "#3498db"), ("深蓝", "#2874a6"), ("浅蓝", "#85c1e2"),
    ("紫", "#9b59b6"), ("深紫", "#7d3c98"), ("浅紫", "#d2b4de"),
    ("天蓝", "#87ceeb"), ("金", "#ffd700"), ("粉", "#ff7fbf"),
    ("黑", "#000000"), ("白", "#ffffff"),
];

const COLOR_PRESETS2: [(&str, &str); 4] = [
    ("灰", "#95a5a6"), ("深灰", "#34495e"), ("浅灰", "#bdc3c7"), ("棕", "#8b5a2b"),
];

fn match_color_value(token: &str) -> Option<String> {
    for (name, value) in COLOR_PRESETS.into_iter().chain(COLOR_PRESETS2) {
        if name == token {
            return Some(value.to_string());
        }
    }
    let bytes = token.as_bytes();
    if bytes.len() == 6 && bytes.iter().all(|b| b.is_ascii_hexdigit()) {
        return Some(format!("#{}", token.to_ascii_lowercase()));
    }
    None
}

fn hex_to_color(hex: &str) -> Color {
    let b = hex.as_bytes();
    let parse = |i: usize| u8::from_str_radix(std::str::from_utf8(&b[i..i + 2]).unwrap(), 16).unwrap();
    Color::from_rgb(parse(1), parse(3), parse(5))
}

/// 从正文末尾解析颜色标记，返回 (剥色后的正文, 文字颜色)。
/// 有「#颜色」标记时优先使用正文最靠后的一个，否则回退到「##颜色」；
/// 颜色为预设名或恰好 6 位 hex。连续 3 个及以上 # 不识别；
/// 不合法的标记原样保留在正文里，如 `C#编程`、`价格#5`、`你好###红`。
fn parse_text_color(text: &str) -> (String, Color) {
    let mut remaining = text.to_string();
    let mut tokens: Vec<(String, u32)> = Vec::new(); // 剥离顺序与正文顺序相反
    for _ in 0..2 {
        let idx = match remaining.rfind('#') {
            Some(i) if i > 0 => i,
            _ => break,
        };
        let mut start = idx;
        while start > 0 && remaining.as_bytes()[start - 1] == b'#' {
            start -= 1;
        }
        let run = (idx - start + 1) as u32; // 末尾连续 # 的个数
        if run > 2 {
            break;
        }
        let matched = match_color_value(&remaining[idx + 1..]);
        let Some(matched) = matched else { break };
        tokens.push((matched, run));
        remaining.truncate(start);
    }
    let color = match tokens.iter().find(|(_, run)| *run == 1) {
        Some((value, _)) => hex_to_color(value),
        None => match tokens.first() {
            Some((value, _)) => hex_to_color(value),
            None => DEFAULT_TEXT_COLOR,
        },
    };
    (remaining, color)
}

fn char_units(c: char) -> f32 {
    let category = c.general_category();
    if matches!(
        category,
        GeneralCategory::NonspacingMark | GeneralCategory::SpacingMark | GeneralCategory::EnclosingMark
    ) {
        return 0.0;
    }
    if matches!(category, GeneralCategory::DecimalNumber | GeneralCategory::OtherNumber) {
        return 0.4;
    }
    if c.is_ascii_alphabetic() {
        return 0.6;
    }
    if c.is_whitespace() {
        return 0.4;
    }
    if c.width() == Some(2) { 1.0 } else { 0.6 }
}

fn wrap_text(text: &str) -> Result<Vec<String>, Error> {
    let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
    let mut result: Vec<String> = Vec::new();
    for paragraph in normalized.split('\n') {
        let mut line = String::new();
        let mut units = 0.0f32;
        for c in paragraph.chars() {
            let width = char_units(c);
            if !line.is_empty() && units + width > MAX_UNITS_PER_LINE {
                result.push(std::mem::replace(&mut line, c.to_string()));
                units = width;
            } else {
                line.push(c);
                units += width;
            }
        }
        result.push(line);
    }
    if result.len() > MAX_LINES {
        return Err(Error::TextOverLength("文字太长，最多三行".to_string()));
    }
    Ok(result)
}

/// 文字层：以首帧标定框为画布，自动字号把整段文字（含彩色 emoji）画在框中央。
fn make_text_layer(wrapped: &str, w: i32, h: i32, color: Color) -> Result<skia_safe::Image, Error> {
    let box_w = w as f32 * 0.9;
    let box_h = h as f32 * 0.66;
    let paint = new_paint(color);
    let mut size = MAX_FONT_SIZE;
    loop {
        let text2image = Text2Image::from_text(
            wrapped,
            size,
            text_params!(
                text_align = TextAlign::Center,
                font_families = &[FONT_FAMILIES],
                paint = paint.clone(),
            ),
        );
        if text2image.longest_line() <= box_w && text2image.height() <= box_h {
            let mut surface = new_surface((w, h));
            surface.canvas().clear(Color::TRANSPARENT);
            text2image.draw_on_canvas(
                surface.canvas(),
                (
                    (w as f32 - text2image.longest_line()) / 2.0,
                    (h as f32 - text2image.height()) / 2.0,
                ),
            );
            return Ok(surface.image_snapshot());
        }
        if size <= MIN_FONT_SIZE + f32::EPSILON {
            break;
        }
        size -= 1.0;
    }
    Err(Error::TextOverLength(wrapped.to_string()))
}

/// 标定框底边的倾角（度），与 Python 版 _bottom_edge_angle 一致。
fn bottom_edge_angle(corners: &[[f32; 2]; 4]) -> f32 {
    let mut bottom = corners.iter().copied().collect::<Vec<[f32; 2]>>();
    bottom.sort_by(|a, b| b[1].total_cmp(&a[1]));
    let (mut left, mut right) = (bottom[0], bottom[1]);
    if left[0] > right[0] {
        std::mem::swap(&mut left, &mut right);
    }
    (right[1] - left[1]).atan2(right[0] - left[0]).to_degrees()
}

fn render(template_id: u32, text: &str) -> Result<Vec<u8>, Error> {
    let name = format!("P{template_id}");
    let template = CALIBRATION
        .templates
        .get(&name)
        .ok_or_else(|| Error::ImageAssetMissing(name.clone()))?;
    let rects = &template.frames;

    let gif_path = format!("sigrika_signholding/{name}.gif");
    let image_path = IMAGES_DIR.join(&gif_path);
    if !(image_path.exists() && image_path.is_file()) {
        return Err(Error::ImageAssetMissing(gif_path));
    }
    let data = Data::from_filename(&image_path)
        .ok_or_else(|| Error::ImageDecodeError("Failed to read gif".to_string()))?;
    let mut codec = skia_safe::Codec::from_data(data)
        .ok_or_else(|| Error::ImageDecodeError("Failed to decode gif".to_string()))?;

    let (body, color) = parse_text_color(text);
    let wrapped = wrap_text(&body)?.join("\n");
    let rect0 = &rects[0];
    let w = rect0.width.max(1.0).round() as i32;
    let h = rect0.height.max(1.0).round() as i32;
    let layer = make_text_layer(&wrapped, w, h, color)?;

    let frame_count = codec.get_frame_count() as usize;
    let mut encoder = GifEncoder::new();
    for (i, rect) in (0..frame_count).zip(rects.iter()) {
        let frame = codec.get_frame(i)?;
        let duration = codec
            .get_frame_info(i)
            .map(|info| info.duration as f32 / 1000.0)
            .filter(|d| *d > 0.0)
            .unwrap_or(FALLBACK_FRAME_DURATION);
        let mut surface = frame.to_surface();
        let canvas = surface.canvas();
        canvas.save();
        canvas.translate((rect.center[0], rect.center[1]));
        canvas.rotate(bottom_edge_angle(&rect.corners), None);
        canvas.draw_image(
            &layer,
            (-w as f32 / 2.0, -h as f32 / 2.0),
            None,
        );
        canvas.restore();
        encoder.add_frame(surface.image_snapshot(), duration)?;
    }
    Ok(encoder.finish()?)
}

macro_rules! sigrika_signholding_meme {
    ($id:literal, $ids:literal, $suffix:literal, $fname:ident) => {
        fn $fname(
            _: Vec<InputImage>,
            texts: Vec<String>,
            _: NoOptions,
        ) -> Result<Vec<u8>, Error> {
            render($id, texts.first().map(String::as_str).unwrap_or(""))
        }

        register_meme!(
            concat!("Sigrika_signholding", $ids),
            $fname,
            min_texts = 1,
            max_texts = 1,
            keywords = &[
                concat!("西格莉卡举牌", $suffix),
                concat!("西西举牌", $suffix),
                concat!("Sigrika_signholding", $ids),
                concat!("xglkjp", $suffix),
                concat!("xglk举牌", $suffix),
                concat!("xxjp", $suffix),
                concat!("xx举牌", $suffix),
            ],
            date_created = local_date(2026, 9, 5),
            date_modified = local_date(2026, 9, 6),
        );
    };
}

sigrika_signholding_meme!(1, "1", "", sigrika_signholding_1);
sigrika_signholding_meme!(2, "2", "2", sigrika_signholding_2);
sigrika_signholding_meme!(3, "3", "3", sigrika_signholding_3);
sigrika_signholding_meme!(4, "4", "4", sigrika_signholding_4);
sigrika_signholding_meme!(5, "5", "5", sigrika_signholding_5);
sigrika_signholding_meme!(6, "6", "6", sigrika_signholding_6);
