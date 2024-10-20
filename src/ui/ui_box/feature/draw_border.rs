use crate::{
    buffer::Buffer2D,
    color,
    graphics::Graphics,
    ui::ui_box::{UIBox, UIBoxFeatureFlag},
};

impl UIBox {
    pub(in crate::ui::ui_box) fn draw_border(
        &self,
        draw_box_boundaries: bool,
        target: &mut Buffer2D,
    ) {
        let (x, y) = self.get_pixel_coordinates();
        let (width, height) = self.get_computed_pixel_size();

        let border_color = if draw_box_boundaries {
            Some(&color::BLUE)
        } else if self.features.contains(UIBoxFeatureFlag::DrawBorder)
            && self.styles.border_color.is_some()
        {
            self.styles.border_color.as_ref()
        } else {
            None
        };

        let fill_color = if draw_box_boundaries && self.is_spacer() {
            Some(&color::RED)
        } else {
            None
        };

        if self.features.contains(UIBoxFeatureFlag::MaskCircle) {
            let radius = width.min(height) as f32 / 2.0;
            let center = (x + width / 2, y + height / 2);

            Graphics::circle(
                target,
                center.0,
                center.1,
                radius as u32,
                fill_color,
                border_color,
            );
        } else {
            Graphics::rectangle(target, x, y, width, height, fill_color, border_color);
        }
    }
}
