use std::{collections::HashMap, path::Path, rc::Rc};

use sdl2::ttf::{Font as SDLFont, Sdl2TtfContext};

use super::FontInfo;

type FontHashMapKey = FontInfo;
type FontHashMapValue<'l> = Rc<SDLFont<'l, 'static>>;
type FontHashMap<'l> = HashMap<FontHashMapKey, FontHashMapValue<'l>>;

pub struct FontCache<'l> {
    context: &'l Sdl2TtfContext,
    cache: FontHashMap<'l>,
}

impl<'l> FontCache<'l> {
    pub fn new(context: &'l Sdl2TtfContext) -> Self {
        Self {
            context,
            cache: Default::default(),
        }
    }

    pub fn load(&mut self, info: &FontInfo) -> Result<FontHashMapValue, String> {
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

                        return Ok(value);
                    }
                    Err(e) => return Err(e),
                }
            }
        }
    }
}
