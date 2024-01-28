use std::{borrow::BorrowMut, sync::RwLock};

use sdl2::mouse::MouseButton;

use crate::{
    app::App,
    buffer::Buffer2D,
    color,
    device::{GameControllerState, KeyboardState, MouseEventKind, MouseState},
    font::{cache::FontCache, FontInfo},
    graphics::text::TextOperation,
    graphics::Graphics,
    ui::button::{do_button, ButtonOptions},
};

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

        if x < self.info.x as i32
            || x >= (self.info.x + self.info.width) as i32
            || y < self.info.y as i32
            || y >= (self.info.y + self.info.height) as i32
        {
            // Click occurred outside of this panel.

            return Ok(());
        }

        let local_x = x - self.info.x as i32;
        let local_y = y - self.info.y as i32;

        match mouse_state.button_event {
            Some(event) => {
                match event.kind {
                    MouseEventKind::Down => {
                        // Click occurred inside of this panel

                        let child_target: &mut Box<Panel<R>>;

                        match self.left.borrow_mut() {
                            Some(left) => {
                                // Split panel scenario

                                let right = self.right.as_mut().unwrap();

                                if x < right.info.x as i32 {
                                    // Occurred in left child

                                    child_target = left;
                                } else {
                                    // Occurred in right child

                                    child_target = right;
                                }

                                let mut local_mouse_state = mouse_state.clone();

                                local_mouse_state.position.0 = local_x - child_target.info.x as i32;
                                local_mouse_state.position.1 = local_y - child_target.info.y as i32;

                                println!(
                                    "[{}] Panel {} observed a click event ({:?}) at ({},{}) (local coords).",
                                    self.info.id,
                                    child_target.info.id,
                                    event.button,
                                    local_mouse_state.position.0,
                                    local_mouse_state.position.1
                                );

                                if child_target.left.borrow_mut().is_none()
                                    && event.button == MouseButton::Right
                                {
                                    self.merge()?;
                                } else {
                                    child_target.update(
                                        app,
                                        &keyboard_state,
                                        &mouse_state,
                                        &game_controller_state,
                                    )?;
                                }
                            }
                            _ => {
                                // Merged panel scenario

                                println!(
                                    "[{}] Panel {} observed a click event ({:?}) at ({},{}) (local coords).",
                                    self.info.id,
                                    self.info.id,
                                    event.button,
                                    local_x,
                                    local_y
                                );
                            }
                        }
                    }
                    _ => (),
                }
            }
            _ => (),
        }

        Ok(())
    }

    pub fn render(
        &mut self,
        app: &mut App,
        keyboard_state: &KeyboardState,
        mouse_state: &MouseState,
        font_cache: &'static RwLock<FontCache<'static>>,
        font_info: &FontInfo,
    ) -> Result<(), String> {
        match self.left.borrow_mut() {
            Some(left) => {
                // Split panel scenario

                // 1. Render left panel to left panel pixel buffer
                left.render(app, keyboard_state, mouse_state, font_cache, font_info)?;

                // 2. Render right panel to right panel pixel buffer
                let right = self.right.as_mut().unwrap();
                right.render(app, keyboard_state, mouse_state, font_cache, font_info)?;

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

                // Renders a border around the panel's boundaries.
                self.draw_panel_border();

                // Renders a default title-bar for this panel.
                self.draw_panel_title_bar(mouse_state, font_cache, font_info)?;

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

    pub fn split(&mut self, alpha: f32) -> Result<(), String> {
        match self.left {
            Some(_) => {
                return Err("Called Panel::split() on an already-split panel!".to_string());
            }
            _ => {}
        }

        self.alpha = alpha;

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

    fn draw_panel_border(&mut self) {
        let x: u32 = 0;
        let y = 0;
        let width = self.info.width - 1;
        let height = self.info.height - 1;

        Graphics::rectangle(&mut self.buffer, x, y, width, height, color::YELLOW)
    }

    fn draw_panel_title_bar(
        &mut self,
        mouse_state: &MouseState,
        font_cache: &'static RwLock<FontCache<'static>>,
        font_info: &FontInfo,
    ) -> Result<(), String> {
        static PANEL_TITLE_BAR_HEIGHT: u32 = 26;

        let (x1, y1, x2, y2) = (
            0 as i32,
            PANEL_TITLE_BAR_HEIGHT as i32,
            self.info.width as i32,
            PANEL_TITLE_BAR_HEIGHT as i32,
        );

        Graphics::line(&mut self.buffer, x1, y1, x2, y2, color::YELLOW);

        {
            let mut cache = font_cache.write().unwrap();

            let font = cache.load(&font_info).unwrap();

            let spacing = PANEL_TITLE_BAR_HEIGHT / 2 - font_info.point_size as u32 / 2;

            Graphics::text(
                &mut self.buffer,
                &font,
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
                x: CLOSE_BUTTON_OFFSET,
                y: CLOSE_BUTTON_OFFSET,
                align_right: true,
                ..Default::default()
            };

            if do_button(&self.info, &mut self.buffer, mouse_state, &button_options) {
                println!("Closing panel {}...", self.info.id);
            }
        }

        Ok(())
    }
}
