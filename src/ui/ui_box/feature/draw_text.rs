use crate::{
    buffer::Buffer2D,
    graphics::{Graphics, text::TextOperation},
    ui::{
        context::UIContext,
        ui_box::{UIBox, UIBoxFeatureFlags},
    },
};

impl UIBox {
    pub(in crate::ui::ui_box) fn draw_text(
        &self,
        ctx: &UIContext,
        target: &mut Buffer2D,
    ) -> Result<(), String> {
        let text_content = self.text_content.as_ref().expect(
            "Called UIBox::render() with `UIBoxFeatureFlags::DRAW_TEXT` when `text_content` is `None`!",
        );

        if text_content.is_empty() {
            return Ok(());
        }

        let text_color = self.styles.text_color.unwrap_or_default();

        let should_cache = !self.features.contains(UIBoxFeatureFlags::SKIP_TEXT_CACHING);

        let mut text_cache = ctx.text_cache.borrow_mut();
        let font_info = ctx.font_info.borrow();
        let mut font_cache_rc = ctx.font_cache.borrow_mut();
        let font_cache = font_cache_rc.as_mut().expect("Found a UIBox with `DrawText` feature enabled when `GLOBAL_UI_CONTEXT.font_cache` is `None`!");

        let (x, y) = self.get_pixel_coordinates();

        Graphics::text(
            target,
            font_cache,
            if should_cache {
                Some(&mut text_cache)
            } else {
                None
            },
            &font_info,
            &TextOperation {
                text: text_content,
                x,
                y,
                color: text_color,
            },
        )
    }
}
