use std::{
    collections::{hash_map::Entry, HashMap},
    sync::RwLock,
};

use crate::{
    buffer::Buffer2D,
    font::{cache::FontCache, FontInfo},
    graphics::Graphics,
};

use super::TextOperation;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TextCacheKey<'a> {
    pub font_info: &'a FontInfo,
    pub text: String,
}

type TextCacheValue = Buffer2D<u8>;

pub type TextCache<'a> = HashMap<TextCacheKey<'a>, TextCacheValue>;

pub fn cache_text<'a>(
    font_cache_rwl: &'a RwLock<FontCache>,
    text_cache_rwl: &'a RwLock<TextCache<'a>>,
    font_info: &'a FontInfo,
    op: &TextOperation,
) {
    let text_cache_key = TextCacheKey {
        font_info,
        text: op.text.clone(),
    };

    let mut text_cache = text_cache_rwl.write().unwrap();

    match text_cache.entry(text_cache_key.clone()) {
        Entry::Occupied(_) => {
            // Occupied
        }
        Entry::Vacant(v) => {
            // Vacant
            let mut font_cache = font_cache_rwl.write().unwrap();

            let font = font_cache.load(font_info).unwrap();

            let (_label_width, _label_height, text_texture) =
                Graphics::make_text_texture(font.as_ref(), &op).unwrap();

            println!("Inserting texture for '{}' text into TextCache!", op.text);

            v.insert(text_texture);
        }
    };
}
