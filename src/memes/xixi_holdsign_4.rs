use meme_generator_core::error::Error;
use meme_generator_utils::{
    builder::InputImage,
    encoder::GifEncoder,
    image::ImageExt,
    text::Text2Image,
    text_params,
    tools::{color_from_hex_code, load_image, local_date, new_paint},
};
use skia_safe::{textlayout::TextAlign, Canvas};

use crate::{options::NoOptions, register_meme};

const FRAME_NUM: u32 = 30;
const FPS: f32 = 30.0;

const MIN_FONT_SIZE: f32 = 18.0;
const MAX_FONT_SIZE: f32 = 40.0;

/// The sign's white interior, measured from the blank template frames,
/// expressed as the extent (in px) from the per-frame sign center in the
/// sign's rotated frame (u along the sign, v across it, +v down):
/// (left, right, top, bottom).
const SIGN_LEFT: f32 = 88.0;
const SIGN_RIGHT: f32 = 88.1;
const SIGN_TOP: f32 = 54.4;
const SIGN_BOTTOM: f32 = 54.3;

/// Margin (留白) kept between the text and the sign border.
const SIGN_PADDING: f32 = 12.0;

/// Usable text area inside the sign.
const TEXT_AREA_W: f32 = SIGN_LEFT + SIGN_RIGHT - 2.0 * SIGN_PADDING;
const TEXT_AREA_H: f32 = SIGN_TOP + SIGN_BOTTOM - 2.0 * SIGN_PADDING;

/// Sign center relative to the text center in the sign's rotated frame.
/// The per-frame centers are the sign centroids, so the text block is
/// centered on the sign.
const SIGN_CENTER_OFFSET: (f32, f32) = (
    (SIGN_RIGHT - SIGN_LEFT) / 2.0,
    (SIGN_BOTTOM - SIGN_TOP) / 2.0,
);

/// Per-frame sign center (cx, cy) and the sign/text rotation angle in degrees.
/// Derived from the reference images by detecting the white sign interior via
/// scipy (largest connected whitish blob) and taking the per-frame centroid +
/// PCA angle. The sign stays in roughly the same spot, wobbling gently.
const TEXT_CENTERS: [(f32, f32); FRAME_NUM as usize] = [
    (189.2, 115.7), (188.4, 115.7), (187.8, 115.2), (187.8, 115.1), (187.8, 115.1),
    (187.3, 114.9), (187.0, 114.7), (186.7, 114.4), (184.5, 112.9), (183.9, 112.3),
    (183.2, 111.7), (180.9, 109.0), (180.7, 108.3), (180.5, 107.6), (181.5, 105.9),
    (182.2, 105.6), (182.9, 105.4), (185.1, 105.3), (185.9, 105.5), (186.5, 105.8),
    (188.4, 107.1), (188.6, 107.6), (189.2, 108.1), (190.2, 110.2), (190.5, 111.3),
    (190.7, 112.0), (190.7, 114.1), (190.5, 114.7), (190.4, 115.1), (189.1, 115.7),
];

const TEXT_ANGLES: [f32; FRAME_NUM as usize] = [
        2.53,    2.21,    1.91,    1.91,    1.88,
        1.59,    1.47,    1.20,   -0.12,   -0.41,
       -1.05,   -2.84,   -3.07,   -3.38,   -3.63,
       -3.48,   -3.24,   -2.28,   -1.88,   -1.45,
       -0.45,    0.13,    0.60,    1.65,    2.09,
        2.29,    2.83,    2.90,    2.87,    2.48,
];

const DEFAULT_TEXT: &str = "QAQ";

fn measure_width(text: &str, size: f32, paint: &skia_safe::Paint) -> f32 {
    Text2Image::from_text(
        text,
        size,
        text_params!(
            text_align = TextAlign::Center,
            font_families = &["Kingnammm Maiyuan 2"],
            paint = paint.clone(),
        ),
    )
    .longest_line()
}

/// Wrap units: ASCII alphanumeric runs stay together (words), every other
/// character (CJK, punctuation, spaces) is its own unit.
fn wrap_units(text: &str) -> Vec<String> {
    let mut units = Vec::new();
    let mut word = String::new();
    for c in text.chars() {
        if c.is_ascii_alphanumeric() {
            word.push(c);
        } else {
            if !word.is_empty() {
                units.push(std::mem::take(&mut word));
            }
            units.push(c.to_string());
        }
    }
    if !word.is_empty() {
        units.push(word);
    }
    units
}

/// Greedily wrap `units` into lines each at most `max_w` wide at `size`.
/// Returns `None` if a single unit alone is wider than `max_w`.
fn wrap_units_in(units: &[String], size: f32, max_w: f32, paint: &skia_safe::Paint) -> Option<Vec<String>> {
    if units.is_empty() {
        return Some(Vec::new());
    }
    let unit_range = |start: usize, end: usize| -> String { units[start..end].concat() };
    if measure_width(&unit_range(0, units.len()), size, paint) <= max_w {
        return Some(vec![unit_range(0, units.len())]);
    }
    let mut lines = Vec::new();
    let mut start = 0;
    while start < units.len() {
        if measure_width(&unit_range(start, units.len()), size, paint) <= max_w {
            lines.push(unit_range(start, units.len()).trim().to_string());
            break;
        }
        if measure_width(&unit_range(start, start + 1), size, paint) > max_w {
            return None;
        }
        let mut lo = start + 1;
        let mut hi = units.len();
        while hi - lo > 1 {
            let mid = (lo + hi) / 2;
            if measure_width(&unit_range(start, mid), size, paint) <= max_w {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        lines.push(unit_range(start, lo).trim().to_string());
        start = lo;
    }
    Some(lines.into_iter().filter(|l| !l.is_empty()).collect())
}

fn wrap_lines(text: &str, size: f32, max_w: f32, paint: &skia_safe::Paint) -> Option<Vec<String>> {
    wrap_units_in(&wrap_units(text), size, max_w, paint)
}

struct FittedText {
    wrapped: String,
    font_size: f32,
}

/// Fit `text` into the sign's text area: first wrap lines at the largest
/// font size, only shrinking the font when the wrapped block is still too
/// tall for the sign. The chosen size and wrapping are reused for every
/// frame in one GIF.
fn fit_text(text: &str, paint: &skia_safe::Paint) -> FittedText {
    let mut size = MAX_FONT_SIZE;
    loop {
        if let Some(lines) = wrap_lines(text, size, TEXT_AREA_W, paint) {
            let wrapped = lines.join("\n");
            let img = Text2Image::from_text(
                &wrapped,
                size,
                text_params!(
                    text_align = TextAlign::Center,
                    font_families = &["Kingnammm Maiyuan 2"],
                    paint = paint.clone(),
                ),
            );
            if img.height() <= TEXT_AREA_H {
                return FittedText {
                    wrapped,
                    font_size: size,
                };
            }
        }
        if size <= MIN_FONT_SIZE + f32::EPSILON {
            break;
        }
        size -= 1.0;
    }
    let lines = wrap_units_in(
        &text.chars().map(|c| c.to_string()).collect::<Vec<_>>(),
        MIN_FONT_SIZE,
        TEXT_AREA_W,
        paint,
    )
    .unwrap_or_default();
    FittedText {
        wrapped: lines.join("\n"),
        font_size: MIN_FONT_SIZE,
    }
}

fn draw_rotated_text(
    canvas: &Canvas,
    text: &str,
    center: (f32, f32),
    angle: f32,
    font_size: f32,
    paint: skia_safe::Paint,
) {
    let img = Text2Image::from_text(
        text,
        font_size,
        text_params!(
            text_align = TextAlign::Center,
            font_families = &["Kingnammm Maiyuan 2"],
            paint = paint,
        ),
    );
    let w = img.longest_line();
    let h = img.height();
    canvas.save();
    canvas.translate((center.0, center.1));
    canvas.rotate(angle, None);
    img.draw_on_canvas(
        canvas,
        (SIGN_CENTER_OFFSET.0 - w / 2.0, SIGN_CENTER_OFFSET.1 - h / 2.0),
    );
    canvas.restore();
}

fn xixi_holdsign_4(
    _: Vec<InputImage>,
    texts: Vec<String>,
    _: NoOptions,
) -> Result<Vec<u8>, Error> {
    let text = if texts.is_empty() {
        DEFAULT_TEXT
    } else {
        texts[0].as_str()
    };

    let paint = new_paint(color_from_hex_code("#f8b860"));
    let fitted = fit_text(text, &paint);

    let mut encoder = GifEncoder::new();
    let duration = 1.0 / FPS;
    for i in 0..FRAME_NUM {
        let frame = load_image(format!("xixi_holdsign_4/{i}.png"))?;
        let mut surface = frame.to_surface();
        draw_rotated_text(
            surface.canvas(),
            &fitted.wrapped,
            TEXT_CENTERS[i as usize],
            TEXT_ANGLES[i as usize],
            fitted.font_size,
            paint.clone(),
        );
        encoder.add_frame(surface.image_snapshot(), duration)?;
    }
    Ok(encoder.finish()?)
}

register_meme!(
    "xixi_holdsign_4",
    xixi_holdsign_4,
    min_texts = 0,
    max_texts = 1,
    default_texts = &[DEFAULT_TEXT],
    keywords = &["西西举牌4"],
    date_created = local_date(2026, 9, 3),
    date_modified = local_date(2026, 9, 3),
);
