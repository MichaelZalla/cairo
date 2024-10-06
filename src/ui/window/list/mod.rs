use sdl2::mouse::MouseButton;

use crate::{
    app::resolution::Resolution, device::mouse::MouseEventKind, mem::linked_list::LinkedList,
    ui::context::GLOBAL_UI_CONTEXT,
};

use super::Window;

#[derive(Default, Debug, Clone)]
pub struct WindowList<'a>(pub LinkedList<Window<'a>>);

impl<'a> WindowList<'a> {
    pub fn rebuild_ui_trees(&mut self, resolution: Resolution) {
        let mut focused_window = None;

        {
            let mut cursor = self.0.cursor_mut();

            while let Some(window) = cursor.peek_prev() {
                let mut did_focus = false;

                // Check if we should capture the current mouse event for this
                // window, exclusively.

                GLOBAL_UI_CONTEXT.with(|ctx| {
                    let mouse = &ctx.input_events.borrow().mouse;

                    if focused_window.is_none()
                        && window.active
                        && window
                            .extent
                            .contains(mouse.position.0 as u32, mouse.position.1 as u32)
                    {
                        if let Some(event) = mouse.button_event {
                            if matches!(
                                (event.button, event.kind),
                                (MouseButton::Left, MouseEventKind::Down)
                            ) {
                                did_focus = true;
                            }
                        }
                    }
                });

                GLOBAL_UI_CONTEXT.with(|ctx| {
                    // Rebuild the UI tree based on the latest user inputs.
                    window.rebuild_ui_trees(ctx, &resolution).unwrap();
                });

                if did_focus && cursor.index() != Some(1) {
                    // Take the focused window out of the window list (temporarily).
                    focused_window.replace(cursor.remove_prev().unwrap());

                    GLOBAL_UI_CONTEXT.with(|ctx| {
                        // Steal the mouse event used to focus the window.
                        let mut input_events = ctx.input_events.borrow_mut();

                        // @TODO Should take entire `MouseState`, not just `button_event`.
                        input_events.mouse.button_event.take();
                    });
                }

                // Advance the window cursor.
                cursor.move_prev();
            }
        }

        if let Some(window) = focused_window {
            // Re-insert the focused window at the end of the window list.
            self.0.push_back(window);
        }

        self.0.retain(|window| window.active);
    }
}
