use std::{collections::HashMap, sync::LazyLock};

use meme_generator_core::error::Error;
use meme_generator_utils::{builder::InputImage, tools::local_date};

use crate::{
    holdsign::{self, FrameRect},
    options::NoOptions,
    register_meme,
};

const CALIBRATION_JSON: &str = include_str!("../data/yaya_holdsign.json");
static CALIBRATION: LazyLock<HashMap<String, Vec<FrameRect>>> =
    LazyLock::new(|| holdsign::load_calibration(CALIBRATION_JSON));

const ASSET_DIR: &str = "yaya_holdsign";
const DEFAULT_TEXT: &str = "生日快乐！";

macro_rules! yaya_holdsign_meme {
    ($key:literal, $template:literal, $fname:ident, [$($kw:literal),+]) => {
        fn $fname(
            _: Vec<InputImage>,
            texts: Vec<String>,
            _: NoOptions,
        ) -> Result<Vec<u8>, Error> {
            holdsign::render(&CALIBRATION, ASSET_DIR, $template, texts, DEFAULT_TEXT)
        }

        register_meme!(
            $key,
            $fname,
            min_images = 0,
            max_images = 0,
            min_texts = 0,
            max_texts = 1,
            default_texts = &[DEFAULT_TEXT],
            keywords = &[$($kw),+],
            date_created = local_date(2026, 8, 30),
            date_modified = local_date(2026, 9, 6),
        );
    };
}

// 编号含义固定：1眨眼 2红温 3开心 4悲伤 5期待 6哭哭
yaya_holdsign_meme!("yaya_holdsign_1", "blink", yaya_holdsign_1, ["娅娅举牌", "娅娅举牌1"]);
yaya_holdsign_meme!("yaya_holdsign_2", "hongwen", yaya_holdsign_2, ["娅娅举牌2"]);
yaya_holdsign_meme!("yaya_holdsign_3", "kaixin", yaya_holdsign_3, ["娅娅举牌3"]);
yaya_holdsign_meme!("yaya_holdsign_4", "beishang", yaya_holdsign_4, ["娅娅举牌4"]);
yaya_holdsign_meme!("yaya_holdsign_5", "qidai", yaya_holdsign_5, ["娅娅举牌5"]);
yaya_holdsign_meme!("yaya_holdsign_6", "kuku", yaya_holdsign_6, ["娅娅举牌6"]);
