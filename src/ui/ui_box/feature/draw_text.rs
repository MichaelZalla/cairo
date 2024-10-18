use crate::{
    buffer::Buffer2D,
    graphics::{text::TextOperation, Graphics},
    ui::{
        context::UIContext,
        ui_box::{UIBox, UIBoxFeatureFlag},
    },
};

impl UIBox {
    pub(in crate::ui::ui_box) fn draw_text(
        &self,
        ctx: &UIContext,
        target: &mut Buffer2D,
    ) -> Result<(), String> {
        let text_content = self.text_content.as_ref().expect(
            "Called UIBox::render() with `UIBoxFeatureFlag::DrawText` when `text_content` is `None`!",
        );

        if text_content.is_empty() {
            return Ok(());
        }

        let text_color = self.styles.text_color.unwrap_or_default();

        let should_cache = !self.features.contains(UIBoxFeatureFlag::SkipTextCaching);

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
