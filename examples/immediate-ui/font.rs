use cairo::{app::App, font::cache::FontCache, ui::context::GLOBAL_UI_CONTEXT};

pub(crate) fn load_system_font(app: &App, font_path: String) {
    GLOBAL_UI_CONTEXT.with(|ctx| {
        ctx.font_cache
            .borrow_mut()
            .replace(FontCache::new(app.context.ttf_context));

        {
            let mut font_info = ctx.font_info.borrow_mut();

            font_info.filepath = font_path;
            font_info.point_size = 14;
        }
    });
}
