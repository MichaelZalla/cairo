use std::collections::{HashMap, hash_map::Entry};

use crate::{
    font::{FontInfo, cache::FontCache},
    graphics::Graphics,
    texture::map::TextureBuffer,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TextCacheKey {
    pub font_info: FontInfo,
    pub text: String,
}

pub type TextMask = TextureBuffer<f32>;

type TextCacheValue = TextMask;

pub type TextCache = HashMap<TextCacheKey, TextCacheValue>;

pub fn cache_text(
    font_cache: &mut FontCache,
    text_cache: &mut TextCache,
    font_info: &FontInfo,
    text: &str,
) -> (u32, u32) {
    let key = TextCacheKey {
        font_info: font_info.clone(),
        text: text.to_string(),
    };

    if let Entry::Vacant(entry) = text_cache.entry(key.clone()) {
        let font = font_cache.load(font_info).unwrap();

        let (label_width, label_height, mask) =
            Graphics::make_text_mask(font.as_ref(), text).unwrap();

        entry.insert(mask.to_owned());

        println!("Cached rendered text ('{}', {}).", text, font_info);

        (label_width, label_height)
    } else {
        let buffer = text_cache.get(&key).unwrap();

        (buffer.0.width, buffer.0.height)
    }
}
