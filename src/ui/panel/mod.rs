use std::rc::Rc;

use crate::{
    color,
    device::{GameControllerState, KeyboardState, MouseState},
    graphics::{Graphics, PixelBuffer},
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

pub struct Panel<U, R>
where
    U: FnMut(&KeyboardState, &MouseState, &GameControllerState, f32) -> (),
    R: FnMut(&mut Graphics, &PanelInfo) -> Result<Vec<u32>, String>,
{
    pub info: PanelInfo,
    graphics: Graphics,
    pub update: U,
    _render: R,
    left: Option<Rc<Panel<U, R>>>,
    right: Option<Rc<Panel<U, R>>>,
    alpha: f32,
}

impl<U, R> Panel<U, R>
where
    U: FnMut(&KeyboardState, &MouseState, &GameControllerState, f32) -> (),
    R: FnMut(&mut Graphics, &PanelInfo) -> Result<Vec<u32>, String>,
{
    pub fn new(info: PanelInfo, update: U, render: R) -> Self
    where
        U: FnMut(&KeyboardState, &MouseState, &GameControllerState, f32) -> (),
        R: FnMut(&mut Graphics, &PanelInfo) -> Result<Vec<u32>, String>,
    {
        let graphics = Graphics {
            buffer: PixelBuffer::new(info.width, info.height),
        };

        return Panel {
            info,
            graphics,
            update,
            _render: render,
            left: None,
            right: None,
            alpha: 1.0,
        };
    }

    pub fn render(&mut self) -> Result<Vec<u32>, String> {
        // Renders a border around the panel's boundaries
        self.render_border();

        return (self._render)(&mut self.graphics, &self.info);
    }

    fn render_border(&mut self) {
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

        self.graphics.poly_line(&panel_bounds, color::YELLOW);
    }

    pub fn split(&mut self) -> Result<(), String> {
        match self.left {
            Some(_) => {
                return Err("Called Panel::split() on an already-split panel!".to_string());
            }
            _ => {}
        }

        // @TODO Implementation

        Ok(())
    }

    pub fn merge(&mut self) -> Result<(), String> {
        match self.left {
            None => {
                return Err("Called Panel::merge() on an unsplit panel!".to_string());
            }
            _ => {}
        }

        // @TODO Implementation

        Ok(())
    }
}