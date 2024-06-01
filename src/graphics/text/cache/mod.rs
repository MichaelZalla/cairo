use std::collections::{hash_map::Entry, HashMap};

use crate::{
    buffer::Buffer2D,
    font::{cache::FontCache, FontInfo},
    graphics::Graphics,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TextCacheKey {
    pub font_info: FontInfo,
    pub text: String,
}

type TextCacheValue = Buffer2D<u8>;

pub type TextCache = HashMap<TextCacheKey, TextCacheValue>;

pub fn cache_text(
    font_cache: &mut FontCache,
    text_cache: &mut TextCache,
    font_info: &FontInfo,
    text: &String,
) -> (u32, u32) {
    let key = TextCacheKey {
        font_info: font_info.clone(),
        text: text.clone(),
    };

    if let Entry::Vacant(entry) = text_cache.entry(key.clone()) {
        let font = font_cache.load(font_info).unwrap();

        let (label_width, label_height, text_texture) =
            Graphics::make_text_mask(font.as_ref(), text).unwrap();

        entry.insert(text_texture.0.to_owned());

        println!("Cached rendered text ('{}', {}).", text, font_info);

        (label_width, label_height)
    } else {
        let buffer = text_cache.get(&key).unwrap();

        (buffer.width, buffer.height)
    }
}
