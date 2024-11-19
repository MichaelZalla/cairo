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

        let fill_color = if draw_box_boundaries && self.is_spacer() {
            Some(color::RED)
        } else {
            None
        };

        let fill_color_u32 = fill_color.map(|c| c.to_u32());

        let border_color = if draw_box_boundaries {
            Some(color::BLUE)
        } else if self.features.contains(UIBoxFeatureFlag::DrawBorder)
            && self.styles.border_color.is_some()
        {
            self.styles.border_color
        } else {
            None
        };

        let border_color_u32 = border_color.map(|c| c.to_u32());

        if self.features.contains(UIBoxFeatureFlag::MaskCircle) {
            let center = (x + width / 2, y + height / 2);

            let radius = width.min(height) as f32 / 2.0;

            Graphics::circle(
                target,
                center.0 as i32,
                center.1 as i32,
                radius as u32,
                fill_color_u32,
                border_color_u32,
            );
        } else {
            Graphics::rectangle(
                target,
                x,
                y,
                width,
                height,
                fill_color_u32,
                border_color_u32,
            );
        }
    }
}
