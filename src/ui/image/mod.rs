use std::cell::RefMut;

use crate::{
    buffer::Buffer2D,
    color::Color,
    graphics::Graphics,
    texture::{map::TextureMap, sample::sample_trilinear},
    vec::vec2::Vec2,
};

use super::{
    context::{UIContext, UIID},
    layout::UILayoutContext,
};

#[derive(Debug)]
pub struct ImageOptions {
    pub width: u32,
    pub height: u32,
    pub border: Option<Color>,
}

pub struct DoImageResult {}

pub fn do_image<'a>(
    ctx: &mut RefMut<'_, UIContext>,
    id: UIID,
    layout: &mut UILayoutContext,
    map: &'a mut TextureMap,
    options: &ImageOptions,
    parent_buffer: &mut Buffer2D,
) -> DoImageResult {
    if !map.is_mipmapped {
        map.enable_mipmapping().unwrap();
    }

    let result = DoImageResult {};

    layout.prepare_cursor(options.width, options.height);

    draw_image(ctx, id, layout, map, options, parent_buffer, &result);

    layout.advance_cursor(options.width, options.height);

    result
}

fn draw_image(
    _ctx: &mut RefMut<'_, UIContext>,
    _id: UIID,
    layout: &UILayoutContext,
    map: &TextureMap,
    options: &ImageOptions,
    parent_buffer: &mut Buffer2D,
    _result: &DoImageResult,
) {
    let cursor = layout.get_cursor();

    let image_top_left = (cursor.x, cursor.y);
    let image_top_right = (image_top_left.0 + options.width - 1, image_top_left.1);
    let image_bottom_right = (
        image_top_left.0 + options.width - 1,
        image_top_left.1 + options.height - 1,
    );
    let image_bottom_left = (image_top_left.0, image_top_left.1 + options.height - 1);

    // Draw the image, with an optional (inner) border.

    let mut far_level_index = 0;
    let mut near_level_index = 0;

    if map.levels.len() > 0 {
        while map.levels[near_level_index].width >= options.width
            && near_level_index < map.levels.len() - 1
        {
            near_level_index += 1;
        }

        far_level_index = near_level_index - 1;
    }

    let alpha = (options.width - map.levels[near_level_index].width) as f32
        / (map.levels[far_level_index].width - map.levels[near_level_index].width) as f32;

    for sample_y in 0..options.height {
        for sample_x in 0..options.width {
            let uv = Vec2 {
                x: sample_x as f32 / options.width as f32,
                y: 1.0 - sample_y as f32 / options.height as f32,
                z: 0.0,
            };

            let sample = sample_trilinear(uv, map, near_level_index, far_level_index, alpha);

            let (screen_x, screen_y) = (cursor.x + sample_x, cursor.y + sample_y);

            if screen_x < parent_buffer.width && screen_y < parent_buffer.height {
                parent_buffer.set(
                    screen_x,
                    screen_y,
                    Color::rgb(sample.0, sample.1, sample.2).to_u32(),
                )
            }
        }
    }

    // Draw the optional inner border.

    match options.border {
        Some(color) => Graphics::poly_line(
            parent_buffer,
            &[
                Vec2 {
                    x: image_top_left.0 as f32,
                    y: image_top_left.1 as f32,
                    z: 0.0,
                },
                Vec2 {
                    x: image_top_right.0 as f32,
                    y: image_top_right.1 as f32,
                    z: 0.0,
                },
                Vec2 {
                    x: image_bottom_right.0 as f32,
                    y: image_bottom_right.1 as f32,
                    z: 0.0,
                },
                Vec2 {
                    x: image_bottom_left.0 as f32,
                    y: image_bottom_left.1 as f32,
                    z: 0.0,
                },
            ],
            color,
        ),
        None => (),
    }
}
