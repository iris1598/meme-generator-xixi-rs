use skia_safe::{Color, Image};

use meme_generator_core::error::Error;
use meme_generator_utils::{
    builder::InputImage,
    encoder::{make_gif_or_combined_gif, FrameAlign, GifInfo},
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
    let func = |i: usize, face_frames: Vec<Image>| {
        // Fit the face to cover the circular window, then mask it to a circle so it
        // stays inside the disc. Works for both static images and animated GIFs.
        let face = face_frames[0]
            .resize_fit((2 * RADIUS as i32, 2 * RADIUS as i32), Fit::Cover)
            .circle();
        let frame = load_image(format!("xixi_goldpig_2/{i}.png"))?;
        let mut surface = new_surface(frame.dimensions());
        let canvas = surface.canvas();
        canvas.clear(Color::TRANSPARENT);

        let (cx, cy) = CENTERS[i];
        canvas.draw_image(&face, (cx - RADIUS, cy - RADIUS), None);
        canvas.draw_image(&frame, (0, 0), None);
        Ok(surface.image_snapshot())
    };

    make_gif_or_combined_gif(
        images,
        func,
        GifInfo {
            frame_num: FRAME_NUM,
            duration: 1.0 / FPS,
        },
        FrameAlign::ExtendLoop,
    )
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
