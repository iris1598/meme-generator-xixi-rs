use std::{collections::HashMap, sync::LazyLock};

use serde::Deserialize;
use skia_safe::{BlendMode, Color, Data, Image, textlayout::TextAlign};

use meme_generator_core::error::Error;
use meme_generator_utils::{
    builder::InputImage,
    config::IMAGES_DIR,
    decoder::CodecExt,
    encoder::GifEncoder,
    image::{Fit, ImageExt},
    text::Text2Image,
    text_params,
    tools::{local_date, new_paint, new_surface},
};

use crate::{options::NoOptions, register_meme};

const CALIBRATION_JSON: &str = include_str!("../data/denia_calibration.json");

const FONT_FAMILIES: &str = "Alimama FangYuanTi VF";
const PAD: f32 = 12.0; // 文字安全区 = 牌面四边形内缩
const MIN_SIZE: i32 = 12; // 可读下限，仍放不下则报文字过长
const INK_TARGET: f32 = 26.0; // 长文本锚定墨高（px）
const INK_MAX: f32 = 40.0; // 1~2 字允许放大的墨高上限
const INK_RATIO: f32 = 0.925; // 方圆体 CJK 墨高/em（移植时 PIL 实测）
const NO_HEAD: &str = "，。！？；：、）】》…!?,.:;";
const DEFAULT_TEXT: &str = "生日快乐！";
const FALLBACK_FRAME_DURATION: f32 = 0.03;
const ASSET_DIR: &str = "denia_jupai";

const NAMED_RGB: [(&str, (u8, u8, u8)); 12] = [
    ("黑", (50, 50, 50)), ("粉", (246, 196, 196)), ("红", (231, 76, 60)),
    ("橙", (230, 126, 34)), ("黄", (241, 196, 15)), ("绿", (46, 204, 113)),
    ("深绿", (22, 86, 54)), ("青", (26, 188, 156)), ("蓝", (52, 152, 219)),
    ("深蓝", (40, 116, 166)), ("紫", (155, 89, 182)), ("金", (255, 215, 0)),
];

#[derive(Deserialize)]
struct FrameRect {
    corners: [[f32; 2]; 4],
    center: [f32; 2],
    width: f32,
    height: f32,
}

static CALIBRATION: LazyLock<HashMap<String, Vec<FrameRect>>> = LazyLock::new(|| {
    serde_json::from_str(CALIBRATION_JSON).expect("invalid denia_calibration.json")
});

fn parse_color(color: &str) -> Result<Color, Error> {
    let s = color.trim().trim_start_matches('#');
    if let Some((_, rgb)) = NAMED_RGB.iter().find(|(name, _)| *name == s) {
        return Ok(Color::from_rgb(rgb.0, rgb.1, rgb.2));
    }
    let csv = s.replace('，', ",");
    let parts: Vec<&str> = csv.split(',').collect();
    let rgb = if parts.len() == 3 {
        let mut v = [0i32; 3];
        for (dst, src) in v.iter_mut().zip(&parts) {
            *dst = src.trim().parse::<i32>()
                .map_err(|_| Error::MemeFeedback(format!("颜色参数不认识：{color}")))?;
        }
        v
    } else if s.len() == 6 && s.bytes().all(|b| b.is_ascii_hexdigit()) {
        let b = s.as_bytes();
        let p = |i: usize| {
            u8::from_str_radix(std::str::from_utf8(&b[i..i + 2]).unwrap(), 16)
                .map_err(|_| Error::MemeFeedback(format!("颜色参数不认识：{color}")))
        };
        [p(0)? as i32, p(2)? as i32, p(4)? as i32]
    } else {
        return Err(Error::MemeFeedback(format!(
            "颜色参数不认识：{color}（可用预设 {}、r,g,b 或 6 位色号）",
            NAMED_RGB.iter().map(|(n, _)| *n).collect::<Vec<_>>().join("、")
        )));
    };
    if rgb.iter().all(|v| (0..=255).contains(v)) {
        Ok(Color::from_rgb(rgb[0] as u8, rgb[1] as u8, rgb[2] as u8))
    } else {
        Err(Error::MemeFeedback(format!("颜色越界：{color}")))
    }
}

// ---------------- 文本尾部颜色标记（#颜色 / 旧写法 -c 颜色） ----------------

fn is_hash(c: char) -> bool {
    c == '#' || c == '＃'
}

fn hex6(s: &str) -> bool {
    s.len() == 6 && s.bytes().all(|b| b.is_ascii_hexdigit())
}

fn named_or_hex(token: &str) -> bool {
    NAMED_RGB.iter().any(|(n, _)| *n == token) || hex6(token)
}

/// 从文本尾部解析颜色标记，返回 (剥色正文, 颜色或 None)。
/// 文字#颜色：预设名或 6 位 hex，匹配成功才剥掉，否则 # 原样保留（## 转义字面 #）；
/// 文字-c 颜色：旧写法，兼容全角横线（－/—/–/﹣）、大小写、横线后可带空格。
fn split_color_tail(text: &str) -> (String, Option<String>) {
    let s = text.trim_end();
    let chars: Vec<char> = s.chars().collect();
    // ---- 结尾 "#token" / "＃token" ----
    // token = 结尾空白前、连续的 [^\s#＃]，其前一字符须为 #/＃
    let mut end = chars.len();
    while end > 0 && chars[end - 1].is_whitespace() {
        end -= 1;
    }
    let mut start = end;
    while start > 0 && !chars[start - 1].is_whitespace() && !is_hash(chars[start - 1]) {
        start -= 1;
    }
    if start > 0 && is_hash(chars[start - 1]) && start < end {
        let token: String = chars[start..end].iter().collect();
        let head: String = chars[..start - 1].iter().collect();
        if head.ends_with(['#', '＃']) {
            // ## 转义：还原一个字面 #
            return (head + &token, None);
        }
        if named_or_hex(&token) {
            return (head.trim_end().to_string(), Some(token));
        }
    }
    // ---- 旧写法 "-c 颜色"（全角横线归一） ----
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
                return (head.trim_end().to_string(), Some(token));
            }
        }
    }
    (text.to_string(), None)
}

// ---------------- emoji 分词（与插件 _tokenize 一致） ----------------

#[derive(Clone)]
enum Tok {
    T(char),
    E(String),
}

fn in_ranges(c: char, ranges: &[(u32, u32)]) -> bool {
    let cp = c as u32;
    ranges.iter().any(|(lo, hi)| cp >= *lo && cp <= *hi)
}

fn is_emoji_base(c: char) -> bool {
    matches!(c as u32, 0x1F000..=0x1FAFF)
        || in_ranges(
            c,
            &[
                (0x2600, 0x27BF),
                (0x2B00, 0x2BFF),
                (0x23E9, 0x23FA),
                (0x2190, 0x21AF),
            ],
        )
        || matches!(
            c,
            '\u{00A9}' | '\u{00AE}' | '\u{203C}' | '\u{2049}' | '\u{2122}' | '\u{2139}'
                | '\u{231A}' | '\u{231B}' | '\u{2328}' | '\u{23CF}' | '\u{24C2}' | '\u{25AA}'
                | '\u{25AB}' | '\u{25B6}' | '\u{25C0}' | '\u{2934}' | '\u{2935}' | '\u{3030}'
                | '\u{303D}' | '\u{3297}' | '\u{3299}'
        )
}

fn is_emoji_presentation(c: char) -> bool {
    matches!(
        c,
        '\u{2B50}' | '\u{2B55}' | '\u{26A1}' | '\u{26BD}' | '\u{26BE}' | '\u{26C4}' | '\u{26C5}'
            | '\u{26D4}' | '\u{26F3}' | '\u{26F5}' | '\u{26FD}' | '\u{2705}' | '\u{2708}'
            | '\u{270A}' | '\u{270B}' | '\u{270C}' | '\u{270D}' | '\u{270F}' | '\u{2712}'
            | '\u{2714}' | '\u{2716}' | '\u{2728}' | '\u{2733}' | '\u{2734}' | '\u{2744}'
            | '\u{2747}' | '\u{274C}' | '\u{274E}' | '\u{2753}' | '\u{2754}' | '\u{2755}'
            | '\u{2757}' | '\u{2763}' | '\u{2764}' | '\u{2795}' | '\u{2796}' | '\u{2797}'
            | '\u{27A1}' | '\u{27B0}' | '\u{27BF}'
    )
}

fn is_skin(c: char) -> bool {
    matches!(c as u32, 0x1F3FB..=0x1F3FF)
}

/// 切成文本单字 / emoji 簇两种 token（ZWJ 序列、肤色、VS16、keycap 归入簇）。
fn tokenize(text: &str) -> Vec<Tok> {
    let chars: Vec<char> = text.chars().collect();
    let mut tokens = Vec::new();
    let mut buf: Vec<char> = Vec::new();
    let mut i = 0;
    let n = chars.len();
    let flush = |buf: &mut Vec<char>, tokens: &mut Vec<Tok>| {
        tokens.extend(buf.drain(..).map(Tok::T));
    };
    while i < n {
        let ch = chars[i];
        // keycap: #*0-9 + U+20E3 (+ VS16)
        if matches!(ch, '#' | '*' | '0'..='9')
            && i + 1 < n
            && chars[i + 1] == '\u{20E3}'
        {
            let mut j = i + 1;
            if j + 1 < n && chars[j + 1] == '\u{FE0F}' {
                j += 1;
            }
            flush(&mut buf, &mut tokens);
            tokens.push(Tok::E(chars[i..=j].iter().collect()));
            i = j + 1;
            continue;
        }
        if is_emoji_base(ch) {
            let mut cluster = String::from(ch);
            i += 1;
            if i < n && (chars[i] == '\u{FE0F}' || is_skin(chars[i])) {
                cluster.push(chars[i]);
                i += 1;
            }
            while i + 1 < n && chars[i] == '\u{200D}' && is_emoji_base(chars[i + 1]) {
                cluster.push(chars[i]);
                cluster.push(chars[i + 1]);
                i += 2;
                if i < n && (chars[i] == '\u{FE0F}' || is_skin(chars[i])) {
                    cluster.push(chars[i]);
                    i += 1;
                }
            }
            let emoji_display = matches!(ch as u32, 0x1F000..=0x1FAFF)
                || is_emoji_presentation(ch)
                || cluster.contains('\u{FE0F}')
                || cluster.contains('\u{200D}');
            if emoji_display {
                flush(&mut buf, &mut tokens);
                tokens.push(Tok::E(cluster));
            } else {
                // 普通文本符号（☆ © ← 等）
                buf.extend(cluster.chars());
            }
            continue;
        }
        buf.push(ch);
        i += 1;
    }
    flush(&mut buf, &mut tokens);
    tokens
}

// ---------------- 换行排版 ----------------

fn tok_width(tok: &Tok, size: f32, paint: &skia_safe::Paint) -> f32 {
    let s = match tok {
        Tok::T(c) => c.to_string(),
        Tok::E(cluster) => cluster.clone(),
    };
    Text2Image::from_text(
        s,
        size,
        text_params!(
            text_align = TextAlign::Center,
            font_families = &[FONT_FAMILIES],
            paint = paint.clone(),
        ),
    )
    .longest_line()
}

/// token 级像素宽度换行 + 行首标点禁则（标点回收上一行行尾）。
fn wrap_tokens(tokens: &[Tok], widths: &[f32], iw: f32) -> Vec<(Vec<(Tok, f32)>, f32)> {
    let mut lines: Vec<(Vec<(Tok, f32)>, f32)> = Vec::new();
    let mut cur: Vec<(Tok, f32)> = Vec::new();
    let mut sum = 0.0f32;
    for (tok, w) in tokens.iter().zip(widths) {
        if !cur.is_empty() && sum + w > iw {
            lines.push((std::mem::take(&mut cur), sum));
            sum = 0.0;
        }
        sum += w;
        cur.push((tok.clone(), *w));
    }
    if !cur.is_empty() {
        lines.push((cur, sum));
    }
    // 行首禁则：下一行行首若是禁则标点，把该 token 移到上一行行尾（可连续回收）
    let mut li = 1;
    while li < lines.len() {
        let movable = matches!(&lines[li].0.first(), Some((Tok::T(c), _)) if NO_HEAD.contains(*c));
        if !movable {
            li += 1;
            continue;
        }
        let (tok, w) = lines[li].0.remove(0);
        lines[li].1 -= w;
        lines[li - 1].0.push((tok, w));
        lines[li - 1].1 += w;
        if lines[li].0.is_empty() {
            lines.remove(li);
        }
    }
    lines.retain(|(l, _)| !l.is_empty());
    lines
}

fn line_to_string(line: &[(Tok, f32)]) -> String {
    line.iter()
        .map(|(t, _)| match t {
            Tok::T(c) => c.to_string(),
            Tok::E(s) => s.clone(),
        })
        .collect()
}

fn ink_target(text: &str) -> f32 {
    let n: f32 = text
        .chars()
        .map(|c| if (c as u32) > 0x2E7F { 1.0 } else { 0.5 })
        .sum();
    let t = ((n - 2.0) / 6.0).clamp(0.0, 1.0);
    INK_MAX + (INK_TARGET - INK_MAX) * t
}

fn text2image(text: &str, size: f32, paint: &skia_safe::Paint) -> Text2Image {
    Text2Image::from_text(
        text,
        size,
        text_params!(
            text_align = TextAlign::Center,
            font_families = &[FONT_FAMILIES],
            paint = paint.clone(),
        ),
    )
}

/// 从墨高锚定字号往下找能放进安全区的最大字号，返回 (字号, 换行后文本)。
fn fit_text(text: &str, iw: f32, ih: f32, paint: &skia_safe::Paint) -> Result<(f32, String), Error> {
    let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
    let default = ((ink_target(text) / INK_RATIO).round() as i32).max(10);
    for size in (MIN_SIZE..=default).rev() {
        let s = size as f32;
        let mut lines: Vec<(Vec<(Tok, f32)>, f32)> = Vec::new();
        let mut overflow = false;
        for para in normalized.split('\n') {
            let tokens = tokenize(para);
            let widths: Vec<f32> = tokens.iter().map(|t| tok_width(t, s, paint)).collect();
            let mut para_lines = wrap_tokens(&tokens, &widths, iw);
            if para_lines.iter().any(|(_, w)| *w > iw) {
                overflow = true;
            }
            lines.append(&mut para_lines);
        }
        if lines.is_empty() || overflow {
            continue;
        }
        let wrapped: String = lines
            .iter()
            .map(|(l, _)| line_to_string(l))
            .collect::<Vec<_>>()
            .join("\n");
        let t2i = text2image(&wrapped, s, paint);
        if t2i.longest_line() <= iw && t2i.height() <= ih {
            return Ok((s, wrapped));
        }
    }
    Err(Error::TextOverLength(
        "文字太长啦，牌子上写不下".to_string(),
    ))
}

/// 文字层：单色文字 + 彩色 emoji 一次成图，整块居中于安全区画布。
fn make_text_layer(
    text: &str,
    w: i32,
    h: i32,
    color: Color,
) -> Result<Image, Error> {
    let paint = new_paint(color);
    let (_size, wrapped) = fit_text(text, w as f32, h as f32, &paint)?;
    let t2i = text2image(&wrapped, _size, &paint);
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

fn render(template: &str, texts: Vec<String>, image: Option<&Image>) -> Result<Vec<u8>, Error> {
    let rects = CALIBRATION
        .get(template)
        .ok_or_else(|| Error::ImageAssetMissing(template.to_string()))?;

    // 文字：剥色 -> 空且无图用默认文字（与插件 main._make 行为一致）
    let raw = texts.first().cloned().unwrap_or_default();
    let (body, color_token) = split_color_tail(raw.trim());
    let mut body = body.trim_end().to_string();
    let color_token = color_token.unwrap_or_else(|| "粉".to_string());
    if body.is_empty() && image.is_none() {
        body = DEFAULT_TEXT.to_string();
    }
    let color = parse_color(&color_token)?;

    let mut base_codec = open_codec(format!("{ASSET_DIR}/base/{template}.gif"))?;
    let mut sign_codec = open_codec(format!("{ASSET_DIR}/sign/{template}.gif"))?;
    let frame_count = base_codec
        .get_frame_count()
        .min(sign_codec.get_frame_count()) as usize;
    let frame_count = frame_count.min(rects.len());
    if frame_count == 0 {
        return Err(Error::ImageAssetMissing(format!(
            "{ASSET_DIR}: 模板 {template} 素材为空"
        )));
    }

    let text_layer = if body.is_empty() {
        None
    } else {
        let quad = &rects[0];
        let tw = (quad.width.round() as i32 - 2 * PAD as i32).max(1);
        let th = (quad.height.round() as i32 - 2 * PAD as i32).max(1);
        Some(make_text_layer(&body, tw, th, color)?)
    };

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

        // 塞图：cover 铺满标定框 -> 旋转贴框心 -> 乘白面遮罩（只落白面）
        if let Some(img) = image {
            let w = rect.width.round().max(1.0) as i32;
            let h = rect.height.round().max(1.0) as i32;
            let face = img.resize_fit((w, h), Fit::Cover);
            let mut layer = new_surface(base.dimensions());
            let lc = layer.canvas();
            lc.clear(Color::TRANSPARENT);
            lc.save();
            lc.translate((rect.center[0], rect.center[1]));
            lc.rotate(bottom_angle(rect), None);
            lc.draw_image(&face, (-w as f32 / 2.0, -h as f32 / 2.0), None);
            lc.restore();
            let mask = meme_generator_utils::tools::load_image(format!(
                "{ASSET_DIR}/panel/{template}/{i}.png"
            ))?;
            let mut mask_paint = skia_safe::Paint::default();
            mask_paint.set_blend_mode(BlendMode::DstIn);
            lc.draw_image(&mask, (0, 0), Some(&mask_paint));
            canvas.draw_image(&layer.image_snapshot(), (0, 0), None);
        }

        if let Some(layer) = &text_layer {
            let dims = layer.dimensions();
            canvas.save();
            canvas.translate((rect.center[0], rect.center[1]));
            canvas.rotate(bottom_angle(rect), None);
            canvas.draw_image(
                layer,
                (-dims.width as f32 / 2.0, -dims.height as f32 / 2.0),
                None,
            );
            canvas.restore();
        }
        encoder.add_frame(surface.image_snapshot(), duration)?;
    }
    Ok(encoder.finish()?)
}

fn make_entry(
    template: &str,
    images: Vec<InputImage>,
    texts: Vec<String>,
) -> Result<Vec<u8>, Error> {
    let image = images.first().map(|i| &i.image);
    render(template, texts, image)
}

macro_rules! denia_jupai_meme {
    ($key:literal, $template:literal, $fname:ident, [$($kw:literal),+]) => {
        fn $fname(
            images: Vec<InputImage>,
            texts: Vec<String>,
            _: NoOptions,
        ) -> Result<Vec<u8>, Error> {
            make_entry($template, images, texts)
        }

        register_meme!(
            $key,
            $fname,
            min_images = 0,
            max_images = 1,
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
denia_jupai_meme!("denia_jupai_1", "blink", denia_jupai_1, ["娅娅举牌", "娅娅举牌1", "娅娅眨眼"]);
denia_jupai_meme!("denia_jupai_2", "hongwen", denia_jupai_2, ["娅娅举牌2", "娅娅红温"]);
denia_jupai_meme!("denia_jupai_3", "kaixin", denia_jupai_3, ["娅娅举牌3", "娅娅开心"]);
denia_jupai_meme!("denia_jupai_4", "beishang", denia_jupai_4, ["娅娅举牌4", "娅娅悲伤"]);
denia_jupai_meme!("denia_jupai_5", "qidai", denia_jupai_5, ["娅娅举牌5", "娅娅期待"]);
denia_jupai_meme!("denia_jupai_6", "kuku", denia_jupai_6, ["娅娅举牌6", "娅娅哭哭"]);
denia_jupai_meme!("am_jupai_1", "am_blink", am_jupai_1, ["小爱举牌", "小爱举牌1", "小爱眨眼"]);
denia_jupai_meme!("am_jupai_2", "am_hongwen", am_jupai_2, ["小爱举牌2", "小爱红温"]);
denia_jupai_meme!("am_jupai_3", "am_kaixin", am_jupai_3, ["小爱举牌3", "小爱开心"]);
denia_jupai_meme!("am_jupai_4", "am_beishang", am_jupai_4, ["小爱举牌4", "小爱悲伤"]);
denia_jupai_meme!("am_jupai_5", "am_qidai", am_jupai_5, ["小爱举牌5", "小爱期待"]);
denia_jupai_meme!("am_jupai_6", "am_kuku", am_jupai_6, ["小爱举牌6", "小爱哭哭"]);
