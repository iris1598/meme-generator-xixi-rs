use meme_generator_core::error::Error;
use meme_generator_utils::{text::Text2Image, text_params};
use skia_safe::{Paint, textlayout::TextAlign};

/// 举牌统一字号逻辑（源自老版 xixi_holdsign_1）：
/// 从 MAX_FONT_SIZE 逐档 -1 试到 MIN_FONT_SIZE，每档按像素宽度贪心折行
/// （ASCII 字母数字连成词不拆行），最多 MAX_LINES 行，全部塞不下则报错。

pub const MIN_FONT_SIZE: f32 = 18.0;
pub const MAX_FONT_SIZE: f32 = 40.0;
pub const MAX_LINES: usize = 3;

pub struct FittedText {
    pub wrapped: String,
    pub size: f32,
}

pub fn measure_text(text: &str, size: f32, font_families: &[&str], paint: &Paint) -> Text2Image {
    Text2Image::from_text(
        text,
        size,
        text_params!(
            text_align = TextAlign::Center,
            font_families = font_families,
            paint = paint.clone(),
        ),
    )
}

fn measure_width(text: &str, size: f32, font_families: &[&str], paint: &Paint) -> f32 {
    measure_text(text, size, font_families, paint).longest_line()
}

/// 折行单元：ASCII 字母数字连成一个词，其余字符各自成单元。
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

/// 把 units 贪心折成每行不超 max_w 的行；某单元单独就超宽时返回 None。
fn wrap_units_in(
    units: &[String],
    size: f32,
    max_w: f32,
    font_families: &[&str],
    paint: &Paint,
) -> Option<Vec<String>> {
    if units.is_empty() {
        return Some(Vec::new());
    }
    let range = |start: usize, end: usize| units[start..end].concat();
    if measure_width(&range(0, units.len()), size, font_families, paint) <= max_w {
        return Some(vec![range(0, units.len())]);
    }
    let mut lines = Vec::new();
    let mut start = 0;
    while start < units.len() {
        if measure_width(&range(start, units.len()), size, font_families, paint) <= max_w {
            lines.push(range(start, units.len()).trim().to_string());
            break;
        }
        if measure_width(&range(start, start + 1), size, font_families, paint) > max_w {
            return None;
        }
        let mut lo = start + 1;
        let mut hi = units.len();
        while hi - lo > 1 {
            let mid = (lo + hi) / 2;
            if measure_width(&range(start, mid), size, font_families, paint) <= max_w {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        lines.push(range(start, lo).trim().to_string());
        start = lo;
    }
    Some(lines.into_iter().filter(|l| !l.is_empty()).collect())
}

pub fn fit_sign_text(
    text: &str,
    area_w: f32,
    area_h: f32,
    font_families: &[&str],
    paint: &Paint,
) -> Result<FittedText, Error> {
    let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
    if normalized.trim().is_empty() {
        return Ok(FittedText { wrapped: String::new(), size: MAX_FONT_SIZE });
    }
    let mut size = MAX_FONT_SIZE;
    loop {
        let mut lines: Vec<String> = Vec::new();
        let mut fits = true;
        for para in normalized.split('\n') {
            match wrap_units_in(&wrap_units(para), size, area_w, font_families, paint) {
                Some(ls) => lines.extend(ls),
                None => {
                    fits = false;
                    break;
                }
            }
        }
        if fits && !lines.is_empty() && lines.len() <= MAX_LINES {
            let wrapped = lines.join("\n");
            if measure_text(&wrapped, size, font_families, paint).height() <= area_h {
                return Ok(FittedText { wrapped, size });
            }
        }
        if size <= MIN_FONT_SIZE + f32::EPSILON {
            break;
        }
        size -= 1.0;
    }
    Err(Error::TextOverLength(
        "文字太长啦，牌子上写不下".to_string(),
    ))
}
