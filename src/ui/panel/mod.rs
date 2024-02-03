use std::{
    borrow::BorrowMut,
    sync::{RwLock, RwLockWriteGuard},
};

use crate::{
    app::App,
    buffer::Buffer2D,
    color,
    device::{GameControllerState, KeyboardState, MouseEventKind, MouseState},
    font::{cache::FontCache, FontInfo},
    graphics::text::TextOperation,
    graphics::{text::cache::TextCache, Graphics},
    ui::{
        button::{do_button, ButtonOptions},
        context::UIID,
        layout::{ItemLayoutHorizontalAlignment, ItemLayoutOptions},
    },
};

use super::context::UIContext;

pub static PANEL_TITLE_BAR_HEIGHT: u32 = 26;

#[derive(Default, Debug)]
pub struct PanelInfo {
    pub id: u32,
    pub title: String,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

pub struct Panel<'a, R>
where
    R: FnMut(
        &PanelInfo,
        &mut Buffer2D,
        &mut App,
        &KeyboardState,
        &MouseState,
    ) -> Result<(), String>,
{
    pub info: PanelInfo,
    pub buffer: Buffer2D,
    render_rwl: Option<&'a RwLock<R>>,
    left: Option<Box<Panel<'a, R>>>,
    right: Option<Box<Panel<'a, R>>>,
    alpha: f32,
    is_resizing: bool,
}

impl<'a, R> Panel<'a, R>
where
    R: FnMut(
        &PanelInfo,
        &mut Buffer2D,
        &mut App,
        &KeyboardState,
        &MouseState,
    ) -> Result<(), String>,
{
    pub fn new(info: PanelInfo, render_rwl: Option<&'a RwLock<R>>) -> Self {
        let buffer = Buffer2D::new(info.width, info.height, None);

        return Panel {
            info,
            buffer,
            render_rwl,
            left: None,
            right: None,
            alpha: 1.0,
            is_resizing: false,
        };
    }

    pub fn is_root(&self) -> bool {
        self.info.id == 0
    }

    pub fn update(
        &mut self,
        app: &mut App,
        keyboard_state: &KeyboardState,
        mouse_state: &MouseState,
        game_controller_state: &GameControllerState,
    ) -> Result<(), String> {
        let (x, y) = mouse_state.position;

        match mouse_state.button_event {
            Some(event) => match event.kind {
                MouseEventKind::Up => {
                    // Any mouse-up event would end our resizing, even if
                    // the event occurs outside of this panel's boundaries.

                    self.is_resizing = false;
                }
                _ => (),
            },
            None => (),
        }

        if x < self.info.x as i32
            || x >= (self.info.x + self.info.width) as i32
            || y < self.info.y as i32
            || y >= (self.info.y + self.info.height) as i32
        {
            // Mouse is not inside this panel.

            return Ok(());
        }

        match self.left.as_mut() {
            Some(left) => {
                left.update(app, keyboard_state, mouse_state, game_controller_state)?;
            }
            None => (),
        }

        match self.right.as_mut() {
            Some(right) => {
                right.update(app, keyboard_state, mouse_state, game_controller_state)?;
            }
            None => (),
        }

        static PANEL_DIVIDER_MOUSE_PADDING: u32 = 8;

        match self.left.borrow_mut() {
            Some(left) => {
                // Check if the mouse is within a region that bounds the panels' divider.

                let panel_divider_x = self.info.x + left.info.width;

                if mouse_state.position.0 > (panel_divider_x - PANEL_DIVIDER_MOUSE_PADDING) as i32
                    && mouse_state.position.0
                        < (panel_divider_x + PANEL_DIVIDER_MOUSE_PADDING) as i32
                {
                    // Set the system cursor to a horizontal-sizing style.

                    let cursor =
                        sdl2::mouse::Cursor::from_system(sdl2::mouse::SystemCursor::SizeWE)?;

                    cursor.set();

                    // Check for a "start resize" or "end resize" event (i.e.,
                    // start mouse drag or end mouse drag).

                    match mouse_state.button_event {
                        Some(event) => match event.kind {
                            MouseEventKind::Down => {
                                self.is_resizing = true;
                            }
                            _ => (),
                        },
                        None => (),
                    }
                }

                if self.is_resizing {
                    self.handle_drag_resize(mouse_state);
                }
            }
            None => (),
        }

        Ok(())
    }

    pub fn render(
        &mut self,
        app: &mut App,
        keyboard_state: &KeyboardState,
        mouse_state: &MouseState,
        ui_context: &'static RwLock<UIContext>,
        font_cache: &'static RwLock<FontCache<'static>>,
        text_cache: &'static RwLock<TextCache<'static>>,
        font_info: &'static FontInfo,
    ) -> Result<(), String> {
        match self.left.borrow_mut() {
            Some(left) => {
                // Split panel scenario

                // 1. Render left panel to left panel pixel buffer
                left.render(
                    app,
                    keyboard_state,
                    mouse_state,
                    ui_context,
                    font_cache,
                    text_cache,
                    font_info,
                )?;

                // 2. Render right panel to right panel pixel buffer
                let right = self.right.as_mut().unwrap();
                right.render(
                    app,
                    keyboard_state,
                    mouse_state,
                    ui_context,
                    font_cache,
                    text_cache,
                    font_info,
                )?;

                // 3. Blit left and right panel pixel buffers onto parent pixel buffer
                self.buffer.blit_from(
                    left.info.x - self.info.x,
                    left.info.y - self.info.y,
                    &left.buffer,
                );

                self.buffer.blit_from(
                    right.info.x - self.info.x,
                    right.info.y - self.info.y,
                    &right.buffer,
                );
            }
            _ => {
                // Merged panel scenario

                self.buffer.clear(None);

                // Renders a border around the panel's boundaries.
                self.draw_panel_frame();

                {
                    let mut ctx = ui_context.write().unwrap();

                    // Renders a default title-bar for this panel.
                    self.draw_panel_title_bar(
                        mouse_state,
                        &mut ctx,
                        font_cache,
                        font_info,
                        text_cache,
                    )?;
                }

                // Runs the custom render callback, if any.
                match self.render_rwl {
                    Some(lock) => {
                        let mut callback = lock.write().unwrap();

                        (*callback)(
                            &self.info,
                            &mut self.buffer,
                            app,
                            keyboard_state,
                            mouse_state,
                        )?;
                    }
                    _ => {}
                }
            }
        }

        return Ok(());
    }

    pub fn split(&mut self) -> Result<(), String> {
        match self.left {
            Some(_) => {
                return Err("Called Panel::split() on an already-split panel!".to_string());
            }
            _ => {}
        }

        self.alpha = 0.5;

        // Generate 2 new sub-panels

        // let padding = 8.0;

        let render_left = self.render_rwl.take();

        let left_id = self.info.id * 2 + 1;
        let right_id = left_id + 1;

        self.left = Some(Box::new(Panel::new(
            PanelInfo {
                title: format!("Panel {}", left_id).to_string(),
                id: left_id,
                x: self.info.x, /* + padding as u32*/
                y: self.info.y, /* + padding as u32*/
                width: (self.info.width as f32 * self.alpha) as u32, /* - (1.5 * padding) as u32*/
                height: self.info.height, /* - 2 * padding as u32*/
            },
            render_left,
        )));

        self.right = Some(Box::new(Panel::new(
            PanelInfo {
                title: format!("Panel {}", right_id).to_string(),
                id: right_id,
                x: self.info.x + (self.info.width as f32 * self.alpha) as u32, /* + (0.5 * padding) as u32*/
                y: self.info.y, /* + padding as u32 */
                width: (self.info.width as f32 * (1.0 - self.alpha)) as u32, /* - (1.5 * padding) as u32*/
                height: self.info.height, /* - 2 * padding as u32 */
            },
            render_left.clone(),
        )));

        Ok(())
    }

    pub fn merge(&mut self) -> Result<(), String> {
        match self.left {
            None => {
                return Err("Called Panel::merge() on an unsplit panel!".to_string());
            }
            _ => {}
        }

        let render = self.left.as_mut().unwrap().render_rwl.take();

        self.left = None;
        self.right = None;

        self.render_rwl = render;

        self.buffer.clear(None);

        Ok(())
    }

    fn handle_drag_resize(&mut self, mouse_state: &MouseState) {
        let left = self.left.as_mut().unwrap();
        let right = self.right.as_mut().unwrap();

        static MINIMUM_PANEL_WIDTH: u32 = 128;

        let relative_motion_x = mouse_state.relative_motion.0;

        // Update left child's width (and resize its buffer).

        left.info.width = (left.info.width as i32 + relative_motion_x)
            .max(MINIMUM_PANEL_WIDTH as i32)
            .min(self.info.width as i32 - MINIMUM_PANEL_WIDTH as i32)
            as u32;

        left.buffer.resize(left.info.width, left.info.height);

        // Update right child's x and width (and resize its buffer).

        right.info.x = (right.info.x as i32 + relative_motion_x)
            .max(MINIMUM_PANEL_WIDTH as i32)
            .min(self.info.width as i32 - MINIMUM_PANEL_WIDTH as i32) as u32;

        right.info.width = (right.info.width as i32 - relative_motion_x)
            .max(MINIMUM_PANEL_WIDTH as i32)
            .min(self.info.width as i32 - MINIMUM_PANEL_WIDTH as i32)
            as u32;

        right.buffer.resize(right.info.width, right.info.height);

        // Update our parent's split alpha.

        self.alpha = left.info.width as f32 / self.info.width as f32;
    }

    fn draw_panel_frame(&mut self) {
        let x: u32 = 0;
        let y = 0;
        let width = self.info.width - 1;
        let height = self.info.height - 1;

        Graphics::rectangle(&mut self.buffer, x, y, width, height, color::YELLOW, None)
    }

    fn draw_panel_title_bar(
        &mut self,
        mouse_state: &MouseState,
        ctx: &mut RwLockWriteGuard<'_, UIContext>,
        font_cache: &'static RwLock<FontCache<'static>>,
        font_info: &'static FontInfo,
        text_cache: &'static RwLock<TextCache<'static>>,
    ) -> Result<(), String> {
        let (x1, y1, x2, y2) = (
            0 as i32,
            PANEL_TITLE_BAR_HEIGHT as i32,
            self.info.width as i32,
            PANEL_TITLE_BAR_HEIGHT as i32,
        );

        Graphics::line(&mut self.buffer, x1, y1, x2, y2, color::YELLOW);

        {
            let spacing = PANEL_TITLE_BAR_HEIGHT / 2 - font_info.point_size as u32 / 2;

            Graphics::text(
                &mut self.buffer,
                font_cache,
                Some(text_cache),
                font_info,
                &TextOperation {
                    text: &format!("Panel {}", self.info.id),
                    x: spacing,
                    y: spacing,
                    color: color::YELLOW,
                },
            )?;
        }

        if !self.is_root() {
            static CLOSE_BUTTON_SIZE: u32 = 14;
            static CLOSE_BUTTON_OFFSET: u32 = (PANEL_TITLE_BAR_HEIGHT - CLOSE_BUTTON_SIZE) / 2;

            let button_options = ButtonOptions {
                layout_options: ItemLayoutOptions {
                    x_offset: CLOSE_BUTTON_OFFSET,
                    y_offset: CLOSE_BUTTON_OFFSET,
                    horizontal_alignment: ItemLayoutHorizontalAlignment::Right,
                    ..Default::default()
                },
                label: "Close".to_string(),
                ..Default::default()
            };

            if do_button(
                ctx,
                UIID {
                    parent: self.info.id,
                    item: 0,
                    index: 0,
                },
                &self.info,
                &mut self.buffer,
                mouse_state,
                font_cache,
                text_cache,
                font_info,
                &button_options,
            )
            .was_released
            {
                println!("Closing panel {}...", self.info.id);
            }
        }

        Ok(())
    }
}
