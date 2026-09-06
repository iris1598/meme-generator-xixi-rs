use std::{collections::HashMap, sync::LazyLock};

use serde::Deserialize;
use skia_safe::{Color, Data};

use meme_generator_core::error::Error;
use meme_generator_utils::{
    builder::InputImage,
    config::IMAGES_DIR,
    decoder::CodecExt,
    encoder::GifEncoder,
    image::ImageExt,
    tools::{local_date, new_paint, new_surface},
};

use crate::{
    holdsign::{parse_color, split_color_tail, FONT_FAMILIES, PAD},
    options::NoOptions,
    register_meme,
    textfit::{fit_sign_text, measure_text},
};

const CALIBRATION_JSON: &str = include_str!("../data/xixi_holdsign.json");

const DEFAULT_COLOR_NAME: &str = "橙";
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
    serde_json::from_str(CALIBRATION_JSON).expect("invalid xixi_holdsign.json")
});

/// 文字层：以首帧标定框为画布，统一字号排版把整段文字（含彩色 emoji）画在框中央。
/// 安全区沿用老版 xixi_holdsign 逻辑：标定框宽高各内缩 2×PAD。
fn make_text_layer(text: &str, w: i32, h: i32, color: Color) -> Result<skia_safe::Image, Error> {
    let box_w = (w - 2 * PAD as i32).max(1) as f32;
    let box_h = (h - 2 * PAD as i32).max(1) as f32;
    let paint = new_paint(color);
    let fitted = fit_sign_text(text, box_w, box_h, FONT_FAMILIES, &paint)?;
    let t2i = measure_text(&fitted.wrapped, fitted.size, FONT_FAMILIES, &paint);
    let mut surface = new_surface((w, h));
    surface.canvas().clear(Color::TRANSPARENT);
    t2i.draw_on_canvas(
        surface.canvas(),
        (
            (w as f32 - t2i.longest_line()) / 2.0,
            (h as f32 - t2i.height()) / 2.0,
        ),
    );
    Ok(surface.image_snapshot())
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

fn render(template_id: u32, text: &str, default_text: &str) -> Result<Vec<u8>, Error> {
    let name = format!("P{template_id}");
    let template = CALIBRATION
        .templates
        .get(&name)
        .ok_or_else(|| Error::ImageAssetMissing(name.clone()))?;
    let rects = &template.frames;

    let gif_path = format!("xixi_holdsign/{name}.gif");
    let image_path = IMAGES_DIR.join(&gif_path);
    if !(image_path.exists() && image_path.is_file()) {
        return Err(Error::ImageAssetMissing(gif_path));
    }
    let data = Data::from_filename(&image_path)
        .ok_or_else(|| Error::ImageDecodeError("Failed to read gif".to_string()))?;
    let mut codec = skia_safe::Codec::from_data(data)
        .ok_or_else(|| Error::ImageDecodeError("Failed to decode gif".to_string()))?;

    // 文字：剥色（与娅娅/小爱同一语义）-> 空则用默认文字
    let (body, color_token) = split_color_tail(text);
    let body = if body.is_empty() {
        default_text.to_string()
    } else {
        body
    };
    let color = parse_color(&color_token.unwrap_or_else(|| DEFAULT_COLOR_NAME.to_string()))?;
    let rect0 = &rects[0];
    let w = rect0.width.max(1.0).round() as i32;
    let h = rect0.height.max(1.0).round() as i32;
    let layer = make_text_layer(&body, w, h, color)?;

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

macro_rules! xixi_holdsign_meme {
    ($id:literal, $ids:literal, $fname:ident, $default_text:literal, [$($kw:literal),+]) => {
        fn $fname(
            _: Vec<InputImage>,
            texts: Vec<String>,
            _: NoOptions,
        ) -> Result<Vec<u8>, Error> {
            render($id, texts.first().map(String::as_str).unwrap_or(""), $default_text)
        }

        register_meme!(
            concat!("xixi_holdsign_", $ids),
            $fname,
            min_texts = 0,
            max_texts = 1,
            default_texts = &[$default_text],
            keywords = &[$($kw),+],
            date_created = local_date(2026, 9, 5),
            date_modified = local_date(2026, 9, 6),
        );
    };
}

// 无文字时的默认文案（1 继承老版 xixi_holdsign，2 取自老版点亮语义，3-6 按动作命名）
xixi_holdsign_meme!(1, "1", xixi_holdsign_1, "咕噜噜––", ["西西举牌", "西西举牌1"]);
xixi_holdsign_meme!(2, "2", xixi_holdsign_2, "点亮语义！", ["西西举牌2"]);
xixi_holdsign_meme!(3, "3", xixi_holdsign_3, "开心", ["西西举牌3"]);
xixi_holdsign_meme!(4, "4", xixi_holdsign_4, "悲伤", ["西西举牌4"]);
xixi_holdsign_meme!(5, "5", xixi_holdsign_5, "得意", ["西西举牌5"]);
xixi_holdsign_meme!(6, "6", xixi_holdsign_6, "哭哭", ["西西举牌6"]);
