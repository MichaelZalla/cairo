use crate::{
    buffer::Buffer2D,
    color::{self, Color},
    graphics::Graphics,
    ui::ui_box::{UIBox, UIBoxFeatureFlag},
};

static UI_BOX_HOT_COLOR: Color = color::RED;
static UI_BOX_ACTIVE_COLOR: Color = color::YELLOW;

impl UIBox {
    pub(in crate::ui::ui_box) fn draw_fill(
        &self,
        is_hot_transitioning: bool,
        is_active_transitioning: bool,
        draw_active_hover_indicators: bool,
        target: &mut Buffer2D,
    ) {
        let fill_color = if draw_active_hover_indicators {
            let end = self.styles.fill_color.unwrap_or_default();

            if is_active_transitioning {
                let with_hot = UI_BOX_HOT_COLOR.lerp_linear(end, self.hot_transition);

                Some(UI_BOX_ACTIVE_COLOR.lerp_linear(with_hot, self.active_transition))
            } else if is_hot_transitioning {
                Some(UI_BOX_HOT_COLOR.lerp_linear(end, 0.5 + self.hot_transition / 2.0))
            } else {
                self.styles.fill_color
            }
        } else {
            self.styles.fill_color
        };

        let (x, y) = self.get_pixel_coordinates();

        let (width, height) = self.get_computed_pixel_size();

        let fill_color_u32 = fill_color.map(|c| c.to_u32());

        if self.features.contains(UIBoxFeatureFlag::MaskCircle) {
            let radius = (width.min(height) as f32 / 2.0).floor();

            let center = (x + width / 2, y + height / 2);

            Graphics::circle(
                target,
                center.0 as i32,
                center.1 as i32,
                radius as u32,
                fill_color_u32,
                None,
            );
        } else {
            Graphics::rectangle(target, x, y, width, height, fill_color_u32, None);
        }
    }
}
