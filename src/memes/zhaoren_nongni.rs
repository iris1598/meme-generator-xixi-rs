use skia_safe::{Color, IRect, ISize, textlayout::TextAlign};

use meme_generator_core::error::Error;
use meme_generator_utils::{
    builder::InputImage,
    canvas::CanvasExt,
    encoder::encode_png,
    image::ImageExt,
    text_params,
    tools::{load_image, local_date, new_paint, new_stroke_paint, new_surface},
};

use crate::{options::NoOptions, register_meme};

const DEFAULT_TEXT: &str = "再发找人弄你";

/// Text block center in the template (measured from 找人弄你.jpg vs 找人弄你_空白.jpg diff).
const CENTER: (f32, f32) = (568.5, 181.5);
/// Tilt of the template text: centroids of the six glyphs fit a line at ~-1.14° (right side up).
const ANGLE: f32 = -1.2;
/// Unrotated text area: width fits the template's 6 glyphs at MAX_FONT_SIZE,
/// height keeps multi-line blocks inside the bubble (auto-shrinks long texts).
const AREA: ISize = ISize::new(1140, 300);

fn zhaoren_nongni(
    _: Vec<InputImage>,
    texts: Vec<String>,
    _: NoOptions,
) -> Result<Vec<u8>, Error> {
    let text = if !texts.is_empty() { &texts[0] } else { DEFAULT_TEXT };
    let frame = load_image("zhaoren_nongni/0.jpg")?;
    let mut surface = frame.to_surface();

    let mut text_surface = new_surface(AREA);
    let canvas = text_surface.canvas();
    canvas.clear(Color::TRANSPARENT);
    canvas.draw_text_area_auto_font_size(
        IRect::from_ltrb(0, 0, AREA.width, AREA.height),
        text,
        20.0,
        208.0,
        text_params!(
            font_families = &["Kingnammm Maiyuan 2"],
            text_align = TextAlign::Center,
            paint = new_paint(Color::from_rgb(0, 0, 0)),
            stroke_paint = new_stroke_paint(Color::from_rgb(0, 0, 0), 6.0),
        ),
    )?;
    let text_image = text_surface.image_snapshot().rotate(ANGLE);

    let text_size = text_image.dimensions();
    surface.canvas().draw_image(
        &text_image,
        (
            (CENTER.0 - text_size.width as f32 / 2.0).round() as i32,
            (CENTER.1 - text_size.height as f32 / 2.0).round() as i32,
        ),
        None,
    );

    encode_png(surface.image_snapshot())
}

register_meme!(
    "zhaoren_nongni",
    zhaoren_nongni,
    min_texts = 1,
    max_texts = 1,
    default_texts = &[DEFAULT_TEXT],
    keywords = &["西西说"],
    date_created = local_date(2026, 9, 9),
    date_modified = local_date(2026, 9, 9),
);
