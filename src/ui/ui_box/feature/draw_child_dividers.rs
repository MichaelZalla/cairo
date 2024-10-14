use std::{cell::RefCell, rc::Rc};

use crate::{
    buffer::Buffer2D,
    collections::tree::node::Node,
    graphics::Graphics,
    ui::ui_box::{UIBox, UILayoutDirection},
};

impl UIBox {
    pub(in crate::ui::ui_box) fn draw_child_dividers(
        &self,
        children: &[Rc<RefCell<Node<UIBox>>>],
        target: &mut Buffer2D,
    ) {
        let divider_color = self.styles.border_color.unwrap_or_default();

        for i in 0..(children.len() - 1) {
            let (child_a_rc, child_b_rc) = (&children[i], &children[i + 1]);

            let (x1, y1, x2, y2) = {
                let child_a_node = &*child_a_rc.borrow();
                let child_a_ui_box = &child_a_node.data;

                let child_b_node = &*child_b_rc.borrow();
                let child_b_ui_box = &child_b_node.data;

                let min_top = child_a_ui_box
                    .global_bounds
                    .top
                    .min(child_b_ui_box.global_bounds.top) as i32;

                let max_bottom = child_a_ui_box
                    .global_bounds
                    .bottom
                    .max(child_b_ui_box.global_bounds.bottom)
                    as i32;

                let min_left = child_a_ui_box
                    .global_bounds
                    .left
                    .min(child_b_ui_box.global_bounds.left) as i32;

                let max_right = child_a_ui_box
                    .global_bounds
                    .right
                    .max(child_b_ui_box.global_bounds.right) as i32;

                match self.layout_direction {
                    UILayoutDirection::TopToBottom => {
                        // Draw a horizontal line across the top of this child.

                        (
                            min_left,
                            child_b_ui_box.global_bounds.top as i32,
                            max_right,
                            child_b_ui_box.global_bounds.top as i32,
                        )
                    }
                    UILayoutDirection::LeftToRight => {
                        // Draw a vertical line along the left of this child.

                        (
                            child_b_ui_box.global_bounds.left as i32,
                            min_top,
                            child_b_ui_box.global_bounds.left as i32,
                            max_bottom,
                        )
                    }
                }
            };

            Graphics::line(target, x1, y1, x2, y2, &divider_color);
        }
    }
}
