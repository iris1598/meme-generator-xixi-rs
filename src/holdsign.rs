use std::collections::HashMap;

use serde::Deserialize;
use skia_safe::{Color, Data, Image};

use meme_generator_core::error::Error;
use meme_generator_utils::{
    config::IMAGES_DIR,
    decoder::CodecExt,
    encoder::GifEncoder,
    image::ImageExt,
    tools::{new_paint, new_surface},
};

use crate::textfit::{fit_sign_text, measure_text};

// 举牌（人物层 + 牌子层双 GIF 合成）通用引擎，供 yaya_holdsign / ams_holdsign 共用。
pub(crate) const FONT_FAMILIES: &[&str] = &["Kingnammm Maiyuan 2", "荆南麦圆 2"];
pub(crate) const PAD: f32 = 12.0; // 文字安全区 = 牌面四边形内缩
const DEFAULT_COLOR: &str = "粉";
const FALLBACK_FRAME_DURATION: f32 = 0.03;

#[derive(Deserialize)]
pub(crate) struct FrameRect {
    pub corners: [[f32; 2]; 4],
    pub center: [f32; 2],
    pub width: f32,
    pub height: f32,
}

pub(crate) fn load_calibration(json: &str) -> HashMap<String, Vec<FrameRect>> {
    serde_json::from_str(json).expect("invalid jupai calibration json")
}

pub(crate) const NAMED_HEX: [(&str, &str); 30] = [
    ("红", "#e74c3c"), ("深红", "#c0392b"), ("浅红", "#ff6b6b"),
    ("橙", "#ffae2e"), ("深橙", "#d35400"), ("浅橙", "#f39c12"),
    ("黄", "#f1c40f"), ("深黄", "#d4ac0d"), ("浅黄", "#fff176"),
    ("绿", "#2ecc71"), ("深绿", "#27ae60"), ("浅绿", "#82e0aa"),
    ("青", "#1abc9c"), ("深青", "#16a085"), ("浅青", "#7fffd4"),
    ("蓝", "#3498db"), ("深蓝", "#2874a6"), ("浅蓝", "#85c1e2"),
    ("紫", "#9b59b6"), ("深紫", "#7d3c98"), ("浅紫", "#d2b4de"),
    ("天蓝", "#87ceeb"), ("金", "#ffd700"), ("粉", "#f6c4c4"),
    ("黑", "#000000"), ("白", "#ffffff"),
    ("灰", "#95a5a6"), ("深灰", "#34495e"), ("浅灰", "#bdc3c7"), ("棕", "#8b5a2b"),
];

pub(crate) fn hex_to_color(hex: &str) -> Color {
    let b = hex.as_bytes();
    let parse = |i: usize| {
        u8::from_str_radix(std::str::from_utf8(&b[i..i + 2]).unwrap(), 16).unwrap()
    };
    Color::from_rgb(parse(1), parse(3), parse(5))
}

pub(crate) fn parse_color(color: &str) -> Result<Color, Error> {
    let s = color.trim().trim_start_matches('#');
    if let Some((_, hex)) = NAMED_HEX.iter().find(|(name, _)| *name == s) {
        return Ok(hex_to_color(hex));
    }
    let csv = s.replace('，', ",");
    let parts: Vec<&str> = csv.split(',').collect();
    if parts.len() == 3 {
        let mut v = [0i32; 3];
        for (dst, src) in v.iter_mut().zip(&parts) {
            *dst = src.trim().parse::<i32>()
                .map_err(|_| Error::MemeFeedback(format!("颜色参数不认识：{color}")))?;
        }
        return if v.iter().all(|x| (0..=255).contains(x)) {
            Ok(Color::from_rgb(v[0] as u8, v[1] as u8, v[2] as u8))
        } else {
            Err(Error::MemeFeedback(format!("颜色越界：{color}")))
        };
    }
    if hex6(s) {
        return Ok(hex_to_color(&format!("#{s}")));
    }
    Err(Error::MemeFeedback(format!(
        "颜色参数不认识：{color}（可用预设 {}、r,g,b 或 6 位色号）",
        NAMED_HEX.iter().map(|(n, _)| *n).collect::<Vec<_>>().join("、")
    )))
}

// ---------------- 文本尾部颜色标记（#颜色 / 旧写法 -c 颜色） ----------------

fn is_hash(c: char) -> bool {
    c == '#' || c == '＃'
}

fn hex6(s: &str) -> bool {
    s.len() == 6 && s.bytes().all(|b| b.is_ascii_hexdigit())
}

/// 合法颜色 token：预设名（30 色）、6 位 hex 或 r,g,b 三元组。
pub(crate) fn is_color_token(token: &str) -> bool {
    NAMED_HEX.iter().any(|(n, _)| *n == token) || hex6(token) || parse_color(token).is_ok()
}

/// 文本以旧写法 "-c 颜色" 结尾时剥出 (正文, 颜色 token)；全角横线（－/—/–/﹣）、大小写 c、横线后可带空格。
pub(crate) fn strip_c_tail(text: &str) -> Option<(String, String)> {
    let s = text.trim_end();
    let chars: Vec<char> = s.chars().collect();
    let norm: Vec<char> = chars
        .iter()
        .map(|c| match c {
            '－' | '—' | '–' | '﹣' => '-',
            c => *c,
        })
        .collect();
    let mut j = norm.len();
    while j > 0 && norm[j - 1].is_whitespace() {
        j -= 1;
    }
    let mut k = j;
    while k > 0 && !norm[k - 1].is_whitespace() {
        k -= 1;
    }
    if k < j {
        let mut m = k; // "-c" 段的结束（\s* 之后）
        while m > 0 && norm[m - 1].is_whitespace() {
            m -= 1;
        }
        if m >= 1 && matches!(norm[m - 1], 'c' | 'C' | 'ｃ') {
            let mut m2 = m - 1; // '-' 前的 \s*
            while m2 > 0 && norm[m2 - 1].is_whitespace() {
                m2 -= 1;
            }
            if m2 >= 1 && norm[m2 - 1] == '-' {
                let token: String = norm[k..j].iter().collect();
                let head: String = chars[..m2 - 1].iter().collect();
                return Some((head.trim_end().to_string(), token));
            }
        }
    }
    None
}

/// 从文本尾部解析颜色标记，返回 (剥色正文, 颜色 token 或 None)。三套举牌共用同一语义：
/// - 末尾「#颜色」/「＃颜色」（全角等效）：30 色预设名、6 位 hex 或 r,g,b，匹配成功才剥离；
///   最多从尾部剥离两层标记，「#」与「##」均生效，同时出现时以最右单个「#」为准；
/// - 「##」连续 3 个及以上或 token 非法时不剥离，原样保留（如 `C#编程`、`你好###红`）；
/// - 无有效「#颜色」时回退旧写法「-c 颜色」（见 `strip_c_tail`）。
pub(crate) fn split_color_tail(text: &str) -> (String, Option<String>) {
    let mut remaining: Vec<char> = text.trim().chars().collect();
    let mut tokens: Vec<(String, u32)> = Vec::new(); // 剥离顺序与正文顺序相反
    for _ in 0..2 {
        let idx = match remaining.iter().rposition(|c| is_hash(*c)) {
            Some(i) => i,
            None => break,
        };
        let mut start = idx;
        while start > 0 && is_hash(remaining[start - 1]) {
            start -= 1;
        }
        let run = (idx - start + 1) as u32; // 末尾连续 #/＃ 的个数
        if run > 2 {
            break;
        }
        let token: String = remaining[idx + 1..].iter().collect::<String>().trim().into();
        if token.is_empty() || !is_color_token(&token) {
            break;
        }
        tokens.push((token, run));
        remaining.truncate(start);
    }
    if let Some((token, _)) = tokens.iter().find(|(_, run)| *run == 1).or(tokens.first()) {
        let body: String = remaining.iter().collect();
        return (body.trim_end().to_string(), Some(token.clone()));
    }
    if let Some((body, token)) = strip_c_tail(text.trim()) {
        return (body, Some(token));
    }
    (text.trim().to_string(), None)
}

/// 文字层：统一字号排版，整块居中于安全区画布。
pub(crate) fn make_text_layer(
    text: &str,
    w: i32,
    h: i32,
    color: Color,
) -> Result<Image, Error> {
    let paint = new_paint(color);
    let fitted = fit_sign_text(text, w as f32, h as f32, FONT_FAMILIES, &paint)?;
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

/// 标定框底边倾角（度），与插件 _bottom_angle 一致。
fn bottom_angle(rect: &FrameRect) -> f32 {
    let mut bottom: Vec<[f32; 2]> = rect.corners.iter().copied().collect();
    bottom.sort_by(|a, b| b[1].total_cmp(&a[1]));
    let (mut left, mut right) = (bottom[0], bottom[1]);
    if left[0] > right[0] {
        std::mem::swap(&mut left, &mut right);
    }
    (right[1] - left[1]).atan2(right[0] - left[0]).to_degrees()
}

fn open_codec(rel: String) -> Result<skia_safe::Codec<'static>, Error> {
    let path = IMAGES_DIR.join(&rel);
    if !(path.exists() && path.is_file()) {
        return Err(Error::ImageAssetMissing(rel));
    }
    let data = Data::from_filename(&path)
        .ok_or_else(|| Error::ImageDecodeError(format!("Failed to read {rel}")))?;
    skia_safe::Codec::from_data(data)
        .ok_or_else(|| Error::ImageDecodeError(format!("Failed to decode {rel}")))
}

pub(crate) fn render(
    calibration: &HashMap<String, Vec<FrameRect>>,
    asset_dir: &str,
    template: &str,
    texts: Vec<String>,
    default_text: &str,
) -> Result<Vec<u8>, Error> {
    let rects = calibration
        .get(template)
        .ok_or_else(|| Error::ImageAssetMissing(template.to_string()))?;

    // 文字：剥色 -> 空则用默认文字
    let raw = texts.first().cloned().unwrap_or_default();
    let (body, color_token) = split_color_tail(raw.trim());
    let mut body = body.trim_end().to_string();
    let color_token = color_token.unwrap_or_else(|| DEFAULT_COLOR.to_string());
    if body.is_empty() {
        body = default_text.to_string();
    }
    let color = parse_color(&color_token)?;

    let mut base_codec = open_codec(format!("{asset_dir}/base/{template}.gif"))?;
    let mut sign_codec = open_codec(format!("{asset_dir}/sign/{template}.gif"))?;
    let frame_count = base_codec
        .get_frame_count()
        .min(sign_codec.get_frame_count()) as usize;
    let frame_count = frame_count.min(rects.len());
    if frame_count == 0 {
        return Err(Error::ImageAssetMissing(format!(
            "{asset_dir}: 模板 {template} 素材为空"
        )));
    }

    let quad = &rects[0];
    let tw = (quad.width.round() as i32 - 2 * PAD as i32).max(1);
    let th = (quad.height.round() as i32 - 2 * PAD as i32).max(1);
    let text_layer = make_text_layer(&body, tw, th, color)?;

    let mut encoder = GifEncoder::new();
    for i in 0..frame_count {
        let rect = &rects[i];
        let base = base_codec.get_frame(i)?;
        let sign = sign_codec.get_frame(i)?;
        let duration = sign_codec
            .get_frame_info(i)
            .map(|info| info.duration as f32 / 1000.0)
            .filter(|d| *d > 0.0)
            .unwrap_or(FALLBACK_FRAME_DURATION);

        let mut surface = base.to_surface();
        let canvas = surface.canvas();
        canvas.draw_image(&sign, (0, 0), None);

        let dims = text_layer.dimensions();
        canvas.save();
        canvas.translate((rect.center[0], rect.center[1]));
        canvas.rotate(bottom_angle(rect), None);
        canvas.draw_image(
            &text_layer,
            (-dims.width as f32 / 2.0, -dims.height as f32 / 2.0),
            None,
        );
        canvas.restore();
        encoder.add_frame(surface.image_snapshot(), duration)?;
    }
    Ok(encoder.finish()?)
}
