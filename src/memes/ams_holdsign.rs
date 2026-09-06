use std::{collections::HashMap, sync::LazyLock};

use meme_generator_core::error::Error;
use meme_generator_utils::{builder::InputImage, tools::local_date};

use crate::{
    holdsign::{self, FrameRect},
    options::NoOptions,
    register_meme,
};

const CALIBRATION_JSON: &str = include_str!("../data/ams_holdsign.json");
static CALIBRATION: LazyLock<HashMap<String, Vec<FrameRect>>> =
    LazyLock::new(|| holdsign::load_calibration(CALIBRATION_JSON));

const ASSET_DIR: &str = "ams_holdsign";
const DEFAULT_TEXT: &str = "生日快乐！";

macro_rules! ams_holdsign_meme {
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
ams_holdsign_meme!("ams_holdsign_1", "blink", ams_holdsign_1, ["小爱举牌", "小爱举牌1"]);
ams_holdsign_meme!("ams_holdsign_2", "hongwen", ams_holdsign_2, ["小爱举牌2"]);
ams_holdsign_meme!("ams_holdsign_3", "kaixin", ams_holdsign_3, ["小爱举牌3"]);
ams_holdsign_meme!("ams_holdsign_4", "beishang", ams_holdsign_4, ["小爱举牌4"]);
ams_holdsign_meme!("ams_holdsign_5", "qidai", ams_holdsign_5, ["小爱举牌5"]);
ams_holdsign_meme!("ams_holdsign_6", "kuku", ams_holdsign_6, ["小爱举牌6"]);
