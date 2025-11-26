use std::{
    collections::{HashMap, hash_map::Iter},
    fmt::Debug,
    path::Path,
    rc::Rc,
};

use sdl2::ttf::{Font as SDLFont, Sdl2TtfContext};

use super::FontInfo;

type FontHashMapKey = FontInfo;
type FontHashMapValue<'l> = Rc<SDLFont<'l, 'static>>;
type FontHashMap<'l> = HashMap<FontHashMapKey, FontHashMapValue<'l>>;

#[derive(Clone)]
pub struct FontCache {
    context: &'static Sdl2TtfContext,
    cache: FontHashMap<'static>,
}

impl Debug for FontCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FontCache")
            .field("context", &"Sdl2TtfContext")
            .field(
                "cache",
                &format!("FontHashMap({} entries)", self.cache.len()),
            )
            .finish()
    }
}

impl FontCache {
    pub fn new(context: &'static Sdl2TtfContext) -> Self {
        Self {
            context,
            cache: Default::default(),
        }
    }

    pub fn load(&mut self, info: &FontInfo) -> Result<FontHashMapValue<'_>, String> {
        match self.cache.get(info) {
            Some(font) => Ok(font.clone()),
            None => {
                let path = Path::new(&info.filepath);

                match self.context.load_font(path, info.point_size) {
                    Ok(mut sdl_font) => {
                        sdl_font.set_style(sdl2::ttf::FontStyle::NORMAL);

                        let key = info.clone();
                        let value = Rc::new(sdl_font);

                        self.cache.insert(key, value.clone());

                        Ok(value)
                    }
                    Err(e) => Err(e),
                }
            }
        }
    }

    pub fn iter(&self) -> Iter<'_, FontHashMapKey, FontHashMapValue<'_>> {
        self.cache.iter()
    }
}
