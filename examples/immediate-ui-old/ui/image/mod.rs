use std::cell::RefMut;

use cairo::{
    buffer::Buffer2D,
    color::Color,
    graphics::Graphics,
    texture::{map::TextureMap, sample::TextureSamplingMethod},
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

pub fn do_image(
    ctx: &mut RefMut<'_, UIContext>,
    layout: &mut UILayoutContext,
    map: &mut TextureMap,
    options: &ImageOptions,
    parent_buffer: &mut Buffer2D,
) -> DoImageResult {
    let id = UIID {
        item: ctx.next_id(),
    };

    if !map.has_mipmaps_generated && map.validate_for_mipmapping().is_ok() {
        map.generate_mipmaps().unwrap();
    }

    let result = DoImageResult {};

    layout.prepare_cursor(options.width, options.height);

    draw_image(ctx, &id, layout, map, options, parent_buffer, &result);

    layout.advance_cursor(options.width, options.height);

    result
}

fn draw_image(
    _ctx: &mut RefMut<'_, UIContext>,
    _id: &UIID,
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

    map.blit_resized(
        cursor.y,
        cursor.x,
        options.width,
        options.height,
        TextureSamplingMethod::Trilinear,
        parent_buffer,
    );

    // Draw the optional inner border.

    if let Some(color) = options.border {
        Graphics::poly_line(
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
            true,
            &color,
        )
    }
}
