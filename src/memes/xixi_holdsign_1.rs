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

/// Max width the text may occupy in the sign (in image pixels).
/// Measured from the original "咕噜噜--" / "点亮语乂" reference frames: about 130 px.
const MAX_TEXT_WIDTH: f32 = 130.0;
const MIN_FONT_SIZE: f32 = 18.0;
const MAX_FONT_SIZE: f32 = 40.0;

/// Per-frame text center (cx, cy) and the sign/text rotation angle in degrees.
/// Derived from the reference images by detecting the white sign interior via
/// scipy (largest connected whitish blob) and taking the per-frame centroid +
/// PCA angle of the largest non-white region inside the sign.
const TEXT_CENTERS: [(f32, f32); FRAME_NUM as usize] = [
    (134.7,  97.9), (130.5,  98.3), (126.9,  98.8), (126.9,  98.8), (126.7,  98.8),
    (125.4,  99.5), (124.6,  99.9), (123.7, 100.6), (117.8, 104.9), (116.2, 106.5),
    (114.9, 108.4), (110.1, 115.0), (110.2, 116.3), (110.6, 117.4), (113.8, 119.0),
    (116.0, 119.0), (124.0, 117.9), (127.0, 117.1), (130.4, 116.2), (139.7, 112.3),
    (141.5, 111.3), (143.0, 110.1), (147.7, 105.6), (148.5, 103.9), (148.4, 102.8),
    (145.7, 100.4), (144.6,  99.4), (143.3,  98.9), (135.9,  97.9), (133.3,  98.0),
];

const TEXT_ANGLES: [f32; FRAME_NUM as usize] = [
       3.48,    2.69,    1.98,    1.98,    2.01,
       1.76,    1.53,    1.27,   -0.13,   -0.74,
      -1.22,   -2.86,   -3.08,   -3.21,   -2.98,
      -2.63,   -1.24,   -0.39,    0.03,    2.35,
       2.80,    3.24,    4.71,    5.07,    5.15,
       5.18,    5.02,    4.88,    3.71,    3.27,
];

const DEFAULT_TEXT: &str = "咕噜噜--";

fn pick_font_size(text: &str, paint: &skia_safe::Paint) -> f32 {
    let mut size = MAX_FONT_SIZE;
    while size >= MIN_FONT_SIZE {
        let img = Text2Image::from_text(
            text,
            size,
            text_params!(
                text_align = TextAlign::Center,
                font_families = &["Kingnammm Maiyuan 2"],
                paint = paint.clone(),
            ),
        );
        if img.longest_line() <= MAX_TEXT_WIDTH {
            return size;
        }
        size -= 1.0;
    }
    MIN_FONT_SIZE
}

fn draw_rotated_text(
    canvas: &Canvas,
    text: &str,
    center: (f32, f32),
    angle: f32,
    font_size: f32,
    paint: skia_safe::Paint,
) {
    let text2image = Text2Image::from_text(
        text,
        font_size,
        text_params!(
            text_align = TextAlign::Center,
            font_families = &["Kingnammm Maiyuan 2"],
            paint = paint,
        ),
    );
    let w = text2image.longest_line();
    let h = text2image.height();
    canvas.save();
    canvas.translate((center.0, center.1));
    canvas.rotate(angle, None);
    text2image.draw_on_canvas(canvas, (-w / 2.0, -h / 2.0));
    canvas.restore();
}

fn xixi_holdsign_1(
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
    let font_size = pick_font_size(text, &paint);

    let mut encoder = GifEncoder::new();
    let duration = 1.0 / FPS;
    for i in 0..FRAME_NUM {
        let frame = load_image(format!("xixi_holdsign_1/{i}.png"))?;
        let mut surface = frame.to_surface();
        draw_rotated_text(
            surface.canvas(),
            text,
            TEXT_CENTERS[i as usize],
            TEXT_ANGLES[i as usize],
            font_size,
            paint.clone(),
        );
        encoder.add_frame(surface.image_snapshot(), duration)?;
    }
    Ok(encoder.finish()?)
}

register_meme!(
    "xixi_holdsign_1",
    xixi_holdsign_1,
    min_texts = 0,
    max_texts = 1,
    default_texts = &[DEFAULT_TEXT],
    keywords = &["西西举牌", "西西举牌1"],
    date_created = local_date(2026, 8, 30),
    date_modified = local_date(2026, 8, 30),
);
