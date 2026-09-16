use std::collections::HashMap;
use std::path::PathBuf;

use serde::Deserialize;
use skia_safe::{
    BlendMode, Color, ColorFilter, Data, Image, Paint, Point, SamplingOptions, Vertices,
    color_filters, vertices::VertexMode,
};

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
    options::NoOptions,
    register_meme,
    textfit::{fit_sign_text, measure_text},
};

const FONT_FAMILIES: &[&str] = &["Kingnammm Maiyuan 2", "荆南麦圆 2"];
const PAD: f32 = 12.0;
const FALLBACK_FRAME_DURATION: f32 = 0.03;
const DRAWABLES_JSON: &str = include_str!("../data/live2d/cubism_drawables.json");
const ALIGN_JSON: &str = include_str!("../data/live2d/live2d_brand_align.json");

const NAMED_COLORS: &[(&str, &str)] = &[
    ("红", "e74c3c"),
    ("深红", "c0392b"),
    ("浅红", "ff6b6b"),
    ("橙", "ffae2e"),
    ("深橙", "d35400"),
    ("浅橙", "f39c12"),
    ("黄", "f1c40f"),
    ("深黄", "d4ac0d"),
    ("浅黄", "fff176"),
    ("绿", "2ecc71"),
    ("深绿", "27ae60"),
    ("浅绿", "82e0aa"),
    ("青", "1abc9c"),
    ("深青", "16a085"),
    ("浅青", "7fffd4"),
    ("蓝", "3498db"),
    ("深蓝", "2874a6"),
    ("浅蓝", "85c1e2"),
    ("紫", "9b59b6"),
    ("深紫", "7d3c98"),
    ("浅紫", "d2b4de"),
    ("天蓝", "87ceeb"),
    ("金", "ffd700"),
    ("粉", "f6c4c4"),
    ("黑", "000000"),
    ("白", "ffffff"),
    ("灰", "95a5a6"),
    ("深灰", "34495e"),
    ("浅灰", "bdc3c7"),
    ("棕", "8b5a2b"),
];

fn parse_color(value: &str) -> Result<Color, Error> {
    let raw = value.trim().trim_start_matches('#');
    let hex = NAMED_COLORS
        .iter()
        .find(|(name, _)| *name == raw)
        .map(|(_, hex)| *hex)
        .unwrap_or(raw);
    if hex.len() == 6 && hex.bytes().all(|b| b.is_ascii_hexdigit()) {
        let byte = |i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap();
        return Ok(Color::from_rgb(byte(0), byte(2), byte(4)));
    }
    let rgb: Vec<_> = raw
        .replace('，', ",")
        .split(',')
        .map(|s| s.trim().parse::<u8>())
        .collect();
    if let [Ok(r), Ok(g), Ok(b)] = rgb.as_slice() {
        return Ok(Color::from_rgb(*r, *g, *b));
    }
    Err(Error::MemeFeedback(format!("颜色参数不认识：{value}")))
}

fn split_color_tail(text: &str) -> (String, Option<String>) {
    let text = text.trim();
    if let Some((body, token)) = text.rsplit_once(['#', '＃']) {
        if !token.is_empty() && parse_color(token).is_ok() {
            return (
                body.trim_end_matches(['#', '＃']).trim_end().to_string(),
                Some(token.trim().to_string()),
            );
        }
    }
    for marker in [" -c ", " –c ", " —c ", " －c "] {
        if let Some((body, token)) = text.rsplit_once(marker) {
            if parse_color(token).is_ok() {
                return (body.trim_end().to_string(), Some(token.trim().to_string()));
            }
        }
    }
    (text.to_string(), None)
}

#[derive(Deserialize)]
struct DrawableFile {
    motions: HashMap<String, Vec<Vec<Drawable>>>,
}
#[derive(Deserialize)]
struct Drawable {
    texture: usize,
    opacity: f32,
    order: i32,
    vertices: [[f32; 2]; 4],
    uv: [[f32; 2]; 4],
}
#[derive(Deserialize)]
struct AlignFile {
    entries: HashMap<String, Align>,
}
#[derive(Deserialize)]
struct Align {
    position: [f32; 2],
    rotation: f32,
    scale: [f32; 2],
}

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

fn load_static_image(path: &std::path::Path) -> Result<Image, Error> {
    let data = Data::from_filename(path)
        .ok_or_else(|| Error::ImageDecodeError(format!("Failed to read {}", path.display())))?;
    let mut codec = skia_safe::Codec::from_data(data)
        .ok_or_else(|| Error::ImageDecodeError("Failed to decode image".to_string()))?;
    Ok(codec.get_frame(0)?.to_surface().image_snapshot())
}

fn asset_path(relative: &str) -> PathBuf {
    let deployed = IMAGES_DIR.join(relative);
    if deployed.is_file() || deployed.is_dir() {
        deployed
    } else {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("resources/images")
            .join(relative)
    }
}

fn tint_filter(color: Color) -> ColorFilter {
    color_filters::lighting(color, Color::BLACK).expect("lighting color filter")
}

fn make_dynamic_atlas(
    style: u32,
    text: &str,
    board_color: Color,
    frame_color: Color,
    inner_color: Color,
    text_color: Color,
) -> Result<Image, Error> {
    let base = asset_path("holdsign/paizipng");
    let mut surface = new_surface((1024, 1024));
    surface.canvas().clear(Color::TRANSPARENT);
    let accent = frame_color;
    // The line masks are mutually exclusive and reconstruct the original
    // line art, so antialiasing from the outer line cannot show through.
    for (layer, paint_color) in [
        ("底色", board_color),
        ("上", board_color),
        ("下", accent),
        ("线稿外", Color::BLACK),
        ("线稿内", inner_color),
    ] {
        let image = load_static_image(&base.join(format!("牌子{style}{layer}.png")))?;
        let mut paint = Paint::default();
        paint.set_color_filter(tint_filter(paint_color));
        surface
            .canvas()
            .draw_image(&image, (0.0, 256.0), Some(&paint));
    }
    let (pw, ph, px, py) = if style == 1 {
        (338, 213, 214, 181)
    } else {
        (380, 240, 190, 170)
    };
    let text_image = make_text_layer(text, pw, ph, text_color)?;
    surface
        .canvas()
        .draw_image(&text_image, (px as f32, (py + 256) as f32), None);
    Ok(surface.image_snapshot())
}

fn parse_args(
    raw: &str,
    default_text: &str,
    default_frame: Color,
    default_inner: Color,
) -> Result<(String, u32, Color, Color, Color, Color), Error> {
    let mut parts = raw.split('|').map(str::trim);
    let first = parts.next().unwrap_or_default();
    let mut body = first.to_string();
    let (body0, legacy_color) = split_color_tail(&body);
    body = body0;
    let mut style = 1u32;
    // Defaults sampled from the official xixi reference image:
    // white face, #f8b860 sign shell, and #f18625 inner line.
    let mut board_color = parse_color("ffffff")?;
    let mut frame_color = default_frame;
    let mut inner_color = default_inner;
    let mut text_color = parse_color(legacy_color.as_deref().unwrap_or("橙"))?;
    for part in parts {
        let Some((key, value)) = part.split_once('=') else {
            continue;
        };
        let key = key.trim();
        let value = value.trim();
        match key {
            "牌子" | "样式" | "style" => {
                style = value
                    .parse::<u32>()
                    .map_err(|_| Error::MemeFeedback("牌子样式必须是 1、2 或 3".into()))?;
                if !(1..=3).contains(&style) {
                    return Err(Error::MemeFeedback("牌子样式必须是 1、2 或 3".into()));
                }
            }
            "牌面色" | "底色" | "board_color" => board_color = parse_color(value)?,
            "牌子色" | "牌色" | "框色" | "边框色" | "frame_color" => {
                frame_color = parse_color(value)?
            }
            "内框色" | "内框颜色" | "inner_color" => inner_color = parse_color(value)?,
            "字色" | "文字色" | "text_color" => text_color = parse_color(value)?,
            _ => {}
        }
    }
    if body.is_empty() {
        body = default_text.to_string();
    }
    Ok((
        body,
        style,
        board_color,
        frame_color,
        inner_color,
        text_color,
    ))
}

pub(crate) fn render_base(
    template_id: u32,
    person_path: &str,
    text: &str,
    default_text: &str,
) -> Result<Vec<u8>, Error> {
    let name = format!("P{template_id}");
    let gif_path = person_path.to_string();
    let image_path = asset_path(&gif_path);
    if !(image_path.exists() && image_path.is_file()) {
        return Err(Error::ImageAssetMissing(gif_path));
    }
    let data = Data::from_filename(&image_path)
        .ok_or_else(|| Error::ImageDecodeError("Failed to read gif".to_string()))?;
    let mut codec = skia_safe::Codec::from_data(data)
        .ok_or_else(|| Error::ImageDecodeError("Failed to decode gif".to_string()))?;

    // 文字：剥色（与娅娅/小爱同一语义）-> 空则用默认文字
    let default_frame = if person_path.contains("/xixi/") {
        parse_color("f8b860")?
    } else {
        // Official yaya/ams sign assets use pink shell #f5c3c3.
        parse_color("f5c3c3")?
    };
    let default_inner = if person_path.contains("/xixi/") {
        parse_color("f18625")?
    } else {
        parse_color("000000")?
    };
    let (body, style, board_color, frame_color, inner_color, text_color) =
        parse_args(text, default_text, default_frame, default_inner)?;
    let atlas = make_dynamic_atlas(
        style,
        &body,
        board_color,
        frame_color,
        inner_color,
        text_color,
    )?;
    let hand = load_static_image(&asset_path("holdsign/model/texture_01.png"))?;
    let drawables: DrawableFile =
        serde_json::from_str(DRAWABLES_JSON).expect("invalid cubism_drawables.json");
    let aligns: AlignFile =
        serde_json::from_str(ALIGN_JSON).expect("invalid live2d_brand_align.json");
    let motion = drawables
        .motions
        .get(&name)
        .ok_or_else(|| Error::ImageAssetMissing(name.clone()))?;
    let align = aligns
        .entries
        .get(&name)
        .ok_or_else(|| Error::ImageAssetMissing(name.clone()))?;

    let frame_count = codec.get_frame_count().min(motion.len());
    let mut encoder = GifEncoder::new();
    for i in 0..frame_count {
        let frame = codec.get_frame(i)?;
        let duration = codec
            .get_frame_info(i)
            .map(|info| info.duration as f32 / 1000.0)
            .filter(|d| *d > 0.0)
            .unwrap_or(FALLBACK_FRAME_DURATION);
        let mut surface = frame.to_surface();
        let canvas = surface.canvas();
        let mut frame_drawables: Vec<_> = motion[i].iter().filter(|d| d.opacity > 0.001).collect();
        frame_drawables.sort_by_key(|d| d.order);
        for d in frame_drawables {
            let (c, s) = (align.rotation.cos(), align.rotation.sin());
            let mut positions = Vec::with_capacity(4);
            let mut tex = Vec::with_capacity(4);
            for (v, uv) in d.vertices.iter().zip(d.uv.iter()) {
                let x = v[0] * align.scale[0] * 768.0;
                let y = -v[1] * align.scale[1] * 768.0;
                positions.push(Point::new(
                    align.position[0] + c * x - s * y - 171.0,
                    align.position[1] + s * x + c * y - 355.0,
                ));
                let ty = (1.0 - uv[1]) * 1024.0;
                tex.push(Point::new(
                    uv[0] * 1024.0,
                    if d.texture == 0 { ty + 256.0 } else { ty },
                ));
            }
            let vertices = Vertices::new_copy(
                VertexMode::Triangles,
                &positions,
                &tex,
                &[Color::WHITE; 4],
                Some(&[0, 1, 2, 1, 3, 2]),
            );
            let image = if d.texture == 0 { &atlas } else { &hand };
            let shader = image
                .to_shader(None, SamplingOptions::default(), None)
                .ok_or_else(|| {
                    Error::ImageDecodeError("Failed to create texture shader".to_string())
                })?;
            let mut paint = Paint::default();
            paint.set_shader(shader);
            paint.set_alpha((d.opacity.clamp(0.0, 1.0) * 255.0) as u8);
            // Vertex colors are white; Modulate preserves the texture RGBA.
            // SrcOver would treat the vertex color as an opaque source and
            // erase the person layer behind transparent atlas pixels.
            canvas.draw_vertices(&vertices, BlendMode::Modulate, &paint);
        }
        encoder.add_frame(surface.image_snapshot(), duration)?;
    }
    Ok(encoder.finish()?)
}

macro_rules! holdsign_meme {
    ($key:literal, $id:literal, $person:literal, $fname:ident, $default_text:literal, [$($kw:literal),+]) => {
        fn $fname(
            _: Vec<InputImage>,
            texts: Vec<String>,
            _: NoOptions,
        ) -> Result<Vec<u8>, Error> {
            render_base($id, $person, texts.first().map(String::as_str).unwrap_or(""), $default_text)
        }

        register_meme!(
            $key,
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

holdsign_meme!(
    "xixi_holdsign_1",
    1,
    "holdsign/person/xixi/P1.gif",
    xixi_holdsign_1,
    "咕噜噜––",
    ["西西举牌", "西西举牌1"]
);

holdsign_meme!(
    "xixi_holdsign_2",
    2,
    "holdsign/person/xixi/P2.gif",
    xixi_holdsign_2,
    "点亮语义！",
    ["西西举牌2"]
);
holdsign_meme!(
    "xixi_holdsign_3",
    3,
    "holdsign/person/xixi/P3.gif",
    xixi_holdsign_3,
    "开心",
    ["西西举牌3"]
);
holdsign_meme!(
    "xixi_holdsign_4",
    4,
    "holdsign/person/xixi/P4.gif",
    xixi_holdsign_4,
    "悲伤",
    ["西西举牌4"]
);
holdsign_meme!(
    "xixi_holdsign_5",
    5,
    "holdsign/person/xixi/P5.gif",
    xixi_holdsign_5,
    "得意",
    ["西西举牌5"]
);
holdsign_meme!(
    "xixi_holdsign_6",
    6,
    "holdsign/person/xixi/P6.gif",
    xixi_holdsign_6,
    "哭哭",
    ["西西举牌6"]
);

holdsign_meme!(
    "yaya_holdsign_1",
    1,
    "holdsign/person/yaya/blink.gif",
    yaya_holdsign_1,
    "生日快乐！",
    ["娅娅举牌", "娅娅举牌1"]
);
holdsign_meme!(
    "yaya_holdsign_2",
    2,
    "holdsign/person/yaya/hongwen.gif",
    yaya_holdsign_2,
    "生日快乐！",
    ["娅娅举牌2"]
);
holdsign_meme!(
    "yaya_holdsign_3",
    3,
    "holdsign/person/yaya/kaixin.gif",
    yaya_holdsign_3,
    "生日快乐！",
    ["娅娅举牌3"]
);
holdsign_meme!(
    "yaya_holdsign_4",
    4,
    "holdsign/person/yaya/beishang.gif",
    yaya_holdsign_4,
    "生日快乐！",
    ["娅娅举牌4"]
);
holdsign_meme!(
    "yaya_holdsign_5",
    5,
    "holdsign/person/yaya/qidai.gif",
    yaya_holdsign_5,
    "生日快乐！",
    ["娅娅举牌5"]
);
holdsign_meme!(
    "yaya_holdsign_6",
    6,
    "holdsign/person/yaya/kuku.gif",
    yaya_holdsign_6,
    "生日快乐！",
    ["娅娅举牌6"]
);

holdsign_meme!(
    "ams_holdsign_1",
    1,
    "holdsign/person/ams/blink.gif",
    ams_holdsign_1,
    "生日快乐！",
    ["小爱举牌", "小爱举牌1"]
);
holdsign_meme!(
    "ams_holdsign_2",
    2,
    "holdsign/person/ams/hongwen.gif",
    ams_holdsign_2,
    "生日快乐！",
    ["小爱举牌2"]
);
holdsign_meme!(
    "ams_holdsign_3",
    3,
    "holdsign/person/ams/kaixin.gif",
    ams_holdsign_3,
    "生日快乐！",
    ["小爱举牌3"]
);
holdsign_meme!(
    "ams_holdsign_4",
    4,
    "holdsign/person/ams/beishang.gif",
    ams_holdsign_4,
    "生日快乐！",
    ["小爱举牌4"]
);
holdsign_meme!(
    "ams_holdsign_5",
    5,
    "holdsign/person/ams/qidai.gif",
    ams_holdsign_5,
    "生日快乐！",
    ["小爱举牌5"]
);
holdsign_meme!(
    "ams_holdsign_6",
    6,
    "holdsign/person/ams/kuku.gif",
    ams_holdsign_6,
    "生日快乐！",
    ["小爱举牌6"]
);
