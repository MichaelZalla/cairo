use std::{rc::Rc, sync::RwLock};

use crate::{
    app::App,
    buffer::Buffer2D,
    color,
    device::{GameControllerState, KeyboardState, MouseState},
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

pub struct Panel<'a, U, R>
where
    U: FnMut(&mut App, &KeyboardState, &MouseState, &GameControllerState) -> (),
    R: FnMut(&mut Buffer2D, &PanelInfo) -> Result<(), String>,
{
    pub info: PanelInfo,
    pub buffer: Buffer2D,
    pub update: U,
    render_rwl: Option<&'a RwLock<R>>,
    left: Option<Rc<Panel<'a, U, R>>>,
    right: Option<Rc<Panel<'a, U, R>>>,
    alpha: f32,
}

impl<'a, U, R> Panel<'a, U, R>
where
    U: FnMut(&mut App, &KeyboardState, &MouseState, &GameControllerState) -> (),
    R: FnMut(&mut Buffer2D, &PanelInfo) -> Result<(), String>,
{
    pub fn new(info: PanelInfo, update: U, render_rwl: Option<&'a RwLock<R>>) -> Self
    where
        U: FnMut(&mut App, &KeyboardState, &MouseState, &GameControllerState) -> (),
        R: FnMut(&mut Buffer2D, &PanelInfo) -> Result<(), String>,
    {
        let buffer = Buffer2D::new(info.width, info.height, None);

        return Panel {
            info,
            buffer,
            update,
            render_rwl,
            left: None,
            right: None,
            alpha: 1.0,
        };
    }

    pub fn render(&mut self) -> Result<(), String> {
        // Renders a border around the panel's boundaries
        self.render_border();

        match self.render_rwl {
            Some(lock) => {
                let mut callback = lock.write().unwrap();

                (*callback)(&mut self.buffer, &self.info)
            }
            None => Ok(()),
        }
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

        Graphics::poly_line(&mut self.buffer, &panel_bounds, color::YELLOW);
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
