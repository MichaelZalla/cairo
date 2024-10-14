use crate::{
    buffer::Buffer2D,
    color,
    ui::ui_box::{UIBox, UIBoxDragHandle},
};

impl UIBox {
    pub(in crate::ui::ui_box) fn draw_debug_drag_handles(&self, target: &mut Buffer2D) {
        let (x, y) = self.get_pixel_coordinates();
        let (width, height) = self.get_computed_pixel_size();

        let (x1, y1, x2, y2) = (
            x as i32,
            y as i32,
            (x + width - 1) as i32,
            (y + height - 1) as i32,
        );

        let handle = match &self.active_drag_handle {
            Some(active_handle) => Some(active_handle),
            None => match &self.hot_drag_handle {
                Some(hot_handle) => Some(hot_handle),
                None => None,
            },
        };

        let color = match &self.active_drag_handle {
            Some(_) => color::BLUE.to_u32(),
            None => match &self.hot_drag_handle {
                Some(_) => color::RED.to_u32(),
                None => 0,
            },
        };

        match &handle {
            Some(handle) => match handle {
                UIBoxDragHandle::Top => {
                    target.horizontal_line_unsafe(x1 as u32, x2 as u32, y1 as u32, color)
                }
                UIBoxDragHandle::Bottom => {
                    target.horizontal_line_unsafe(x1 as u32, x2 as u32, y2 as u32, color)
                }
                UIBoxDragHandle::Left => {
                    target.vertical_line_unsafe(x1 as u32, y1 as u32, y2 as u32, color)
                }
                UIBoxDragHandle::Right => {
                    target.vertical_line_unsafe(x2 as u32, y1 as u32, y2 as u32, color)
                }
            },
            None => (),
        }
    }
}
