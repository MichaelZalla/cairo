use std::{borrow::BorrowMut, sync::RwLock};

use sdl2::mouse::MouseButton;

use crate::{
    app::App,
    buffer::Buffer2D,
    color,
    device::{GameControllerState, KeyboardState, MouseEventKind, MouseState},
    graphics::Graphics,
    vec::vec2::Vec2,
};

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
    R: FnMut(&mut Buffer2D, &PanelInfo) -> Result<(), String>,
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
    R: FnMut(&mut Buffer2D, &PanelInfo) -> Result<(), String>,
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

                                match event.button {
                                    MouseButton::Left => {
                                        self.split(0.5)?;
                                    }
                                    _ => {}
                                }
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

    pub fn render(&mut self) -> Result<(), String> {
        // Renders a border around the panel's boundaries
        self.render_border();

        match self.left.borrow_mut() {
            Some(left) => {
                // Split panel scenario

                // 1. Render left panel to left panel pixel buffer
                left.render()?;

                // 2. Render right panel to right panel pixel buffer
                let right = self.right.as_mut().unwrap();
                right.render()?;

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

                match self.render_rwl {
                    Some(lock) => {
                        let mut callback = lock.write().unwrap();

                        (*callback)(&mut self.buffer, &self.info)?;
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

        self.left = Some(Box::new(Panel::new(
            PanelInfo {
                title: format!("{} - Left", self.info.title).to_string(),
                id: self.info.id * 2 + 1,
                x: self.info.x,                                 /* + padding as u32*/
                y: self.info.y,                                 /* + padding as u32*/
                width: (self.info.width as f32 * alpha) as u32, /* - (1.5 * padding) as u32*/
                height: self.info.height,                       /* - 2 * padding as u32*/
            },
            render_left,
        )));

        self.right = Some(Box::new(Panel::new(
            PanelInfo {
                title: format!("{} - Right", self.info.title).to_string(),
                id: self.info.id * 2 + 2,
                x: self.info.x + (self.info.width as f32 * alpha) as u32, /* + (0.5 * padding) as u32*/
                y: self.info.y,                                           /* + padding as u32 */
                width: (self.info.width as f32 * (1.0 - alpha)) as u32, /* - (1.5 * padding) as u32*/
                height: self.info.height,                               /* - 2 * padding as u32 */
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

    pub fn render_border(&mut self) {
        let left = 0.0;
        let top = 0.0;
        let right = self.info.width as f32 - 1.0;
        let bottom = self.info.height as f32 - 1.0;

        let panel_bounds = vec![
            // Top-left
            Vec2 {
                y: top,
                x: left,
                z: 0.0,
            },
            // Top-right
            Vec2 {
                y: top,
                x: right,
                z: 0.0,
            },
            // Bottom-right
            Vec2 {
                y: bottom,
                x: right,
                z: 0.0,
            },
            // Bottom-left
            Vec2 {
                y: bottom,
                x: left,
                z: 0.0,
            },
        ];

        Graphics::poly_line(&mut self.buffer, &panel_bounds, color::YELLOW);
    }
}
