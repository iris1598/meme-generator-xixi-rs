use skia_safe::{ClipOp, Color, Path};

use meme_generator_core::error::Error;
use meme_generator_utils::{
    builder::InputImage,
    encoder::GifEncoder,
    image::{Fit, ImageExt},
    tools::{load_image, local_date, new_surface},
};

use crate::{options::NoOptions, register_meme};

const FRAME_NUM: u32 = 18;
const FPS: f32 = 16.7;

/// Radius (in px) of the circular window in the template's top-right.
const RADIUS: f32 = 65.0;

/// Per-frame center (cx, cy) of the circular window, from 素材2/centers.json.
const CENTERS: [(f32, f32); FRAME_NUM as usize] = [
    (241.94, 187.03), (242.00, 186.37), (242.07, 180.57), (242.17, 176.86), (242.21, 174.39),
    (242.69, 170.40), (242.00, 175.02), (242.00, 178.91), (241.79, 183.48), (241.94, 187.03),
    (242.00, 186.37), (242.07, 180.57), (242.19, 177.52), (242.22, 174.08), (242.00, 171.26),
    (242.00, 175.02), (242.00, 178.91), (241.79, 183.48),
];

fn xixi_goldpig_2(images: Vec<InputImage>, _: Vec<String>, _: NoOptions) -> Result<Vec<u8>, Error> {
    // Fit the face to cover the circular window (diameter x diameter), cropping
    // the source so the disc is always filled.
    let face = images[0].image.resize_fit((2 * RADIUS as i32, 2 * RADIUS as i32), Fit::Cover);

    let mut encoder = GifEncoder::new();
    let duration = 1.0 / FPS;
    for i in 0..FRAME_NUM as usize {
        let frame = load_image(format!("xixi_goldpig_2/{i}.png"))?;
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
    "xixi_goldpig_2",
    xixi_goldpig_2,
    min_images = 1,
    max_images = 1,
    keywords = &["西西展示"],
    date_created = local_date(2026, 8, 31),
    date_modified = local_date(2026, 8, 31),
);
