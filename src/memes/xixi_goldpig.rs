use skia_safe::{ClipOp, Color, Path};

use meme_generator_core::error::Error;
use meme_generator_utils::{
    builder::InputImage,
    encoder::GifEncoder,
    image::{Fit, ImageExt},
    tools::{load_image, local_date, new_surface},
};

use crate::{options::NoOptions, register_meme};

const FRAME_NUM: u32 = 27;
const FPS: f32 = 16.7;

/// Radius (in px) of the circular window in the template's bottom-left.
const RADIUS: f32 = 67.0;

/// Per-frame center (cx, cy) of the circular window, from 素材/centers.json.
const CENTERS: [(f32, f32); FRAME_NUM as usize] = [
    (118.57, 249.71), (118.65, 249.41), (118.61, 249.56), (118.84, 249.08), (118.86, 249.53),
    (118.80, 248.47), (118.97, 249.11), (119.09, 248.50), (119.25, 249.25), (119.33, 249.50),
    (119.50, 250.60), (119.50, 251.20), (119.65, 251.55), (119.50, 251.50), (119.68, 251.50),
    (119.66, 251.85), (119.65, 252.46), (119.34, 252.19), (119.34, 252.19), (119.34, 252.19),
    (119.29, 252.23), (118.99, 251.38), (118.78, 249.50), (119.00, 251.00), (118.80, 250.80),
    (118.65, 250.41), (118.65, 250.41),
];

fn xixi_goldpig(images: Vec<InputImage>, _: Vec<String>, _: NoOptions) -> Result<Vec<u8>, Error> {
    // Fit the face to cover the circular window (diameter x diameter), cropping
    // the source so the disc is always filled.
    let face = images[0].image.resize_fit((2 * RADIUS as i32, 2 * RADIUS as i32), Fit::Cover);

    let mut encoder = GifEncoder::new();
    let duration = 1.0 / FPS;
    for i in 0..FRAME_NUM as usize {
        let frame = load_image(format!("xixi_goldpig/{i}.png"))?;
        let mut surface = new_surface(frame.dimensions());
        let canvas = surface.canvas();
        canvas.clear(Color::TRANSPARENT);

        let (cx, cy) = CENTERS[i];
        canvas.save();
        let clip = Path::circle((cx, cy), RADIUS, None);
        canvas.clip_path(&clip, ClipOp::Intersect, true);
        canvas.translate((cx, cy));
        canvas.draw_image(&face, (-RADIUS, -RADIUS), None);
        canvas.restore();

        canvas.draw_image(&frame, (0, 0), None);
        encoder.add_frame(surface.image_snapshot(), duration)?;
    }
    Ok(encoder.finish()?)
}

register_meme!(
    "xixi_goldpig",
    xixi_goldpig,
    min_images = 1,
    max_images = 1,
    keywords = &["西西摸"],
    date_created = local_date(2026, 8, 31),
    date_modified = local_date(2026, 8, 31),
);
