use std::ops::Rem;

use crate::{
    texture::map::{TextureMapStorageFormat, TextureMapWrapping},
    vec::{vec2::Vec2, vec3::Vec3},
};

use super::map::TextureMap;

fn apply_wrapping_options(uv: Vec2, map: &TextureMap) -> Vec2 {
    match map.options.wrapping {
        TextureMapWrapping::Repeat => Vec2 {
            x: if uv.x < 0.0 || uv.x >= 1.0 {
                uv.x.rem_euclid(1.0)
            } else {
                uv.x
            },
            y: if uv.y < 0.0 || uv.y >= 1.0 {
                uv.y.rem_euclid(1.0)
            } else {
                uv.y
            },
            z: 1.0,
        },
        TextureMapWrapping::ClampToEdge => Vec2 {
            x: uv.x.max(0.0).min(1.0),
            y: uv.y.max(0.0).min(1.0),
            z: 1.0,
        },
        TextureMapWrapping::ClampToBorder(_border_color) => {
            // Out-of-bounds UVs will remain out-of-bounds.

            uv
        }
    }
}

pub fn sample_nearest(uv: Vec2, map: &TextureMap, level_index: Option<usize>) -> (u8, u8, u8) {
    debug_assert!(map.is_loaded);

    let safe_uv = apply_wrapping_options(uv, map);

    debug_assert!(
        map.levels[0].data.len()
            == (map.width * map.height * map.get_bytes_per_pixel() as u32) as usize,
        "filepath={}, levels[0].data.len() = {}, map.width={}, map.height={}, map.bytes_per_pixel={}",
        map.info.filepath,
        map.levels[0].data.len(),
        map.width,
        map.height,
        map.get_bytes_per_pixel(),
    );

    // Determine our map dimensions, based on the level index.
    let level_width = match level_index {
        Some(index) => map.width / (2 as u32).pow(index as u32),
        None => map.width,
    };

    let level_height = match level_index {
        Some(index) => map.height / (2 as u32).pow(index as u32),
        None => map.height,
    };

    // Perform any out-of-bounds handling.

    match map.options.wrapping {
        TextureMapWrapping::ClampToBorder(border_color) => {
            if safe_uv.x < 0.0 || safe_uv.x > 1.0 || safe_uv.y < 0.0 || safe_uv.y > 1.0 {
                return border_color;
            }
        }
        _ => (),
    }

    // Maps the wrapped UV coordinate to the nearest whole texel coordinate.

    let texel_x = safe_uv.x * (level_width - 1) as f32;
    let texel_y = (1.0 - safe_uv.y) * (level_height - 1) as f32;

    return sample_from_texel((texel_x, texel_y), map, level_index);
}

pub fn sample_bilinear(uv: Vec2, map: &TextureMap, level_index: Option<usize>) -> (u8, u8, u8) {
    debug_assert!(map.is_loaded);

    let safe_uv = apply_wrapping_options(uv, map);

    // Determine our map dimensions, based on the level index.
    let level_width = match level_index {
        Some(index) => map.width / (2 as u32).pow(index as u32),
        None => map.width,
    };

    let level_height = match level_index {
        Some(index) => map.height / (2 as u32).pow(index as u32),
        None => map.height,
    };

    debug_assert!(
        level_index == None || level_index.unwrap() < map.levels.len(),
        "map={}, level_index={}, map.levels.len={}",
        map.info.filepath,
        level_index.unwrap(),
        map.levels.len(),
    );

    debug_assert!(
        level_width > 0 && level_height > 0,
        "map={}, level_width={}, level_height={}",
        map.info.filepath,
        level_width,
        level_height
    );

    // Maps the wrapped UV coordinate to a fractional texel coordinate.
    let wrapped_uv_as_fractional_texel = Vec2 {
        x: (safe_uv.x * (level_width - 1) as f32),
        y: ((1.0 - safe_uv.y) * (level_height - 1) as f32),
        z: 1.0,
    };

    // Performs bilinear filtering (interpolation)

    // See: https://en.wikipedia.org/wiki/Bilinear_interpolation#Computation
    // See: https://www.youtube.com/watch?v=AqscP7rc8_M

    let r: f32;
    let g: f32;
    let b: f32;

    let nearest_neighbors = get_neighbors(wrapped_uv_as_fractional_texel, map, level_index);

    match nearest_neighbors {
        // Case: One neighbor (top-left)
        (Some(texel), None, None, None) |

        // Case: One neighbor (top-right)
        (None, Some(texel), None, None) |

        // Case: One neighbor (bottom-left)
        (None, None, Some(texel), None) |

        // Case: One neighbor (bottom-right)
        (None, None, None, Some(texel)) => return sample_from_texel(texel, map, level_index),

        // Case: Two neighbors (left column)
        (Some(top_left), None, Some(bottom_left), None) => {
            // Interpolate between top_left and bottom_left (based on uv.y).

            let sample_a = sample_from_texel(top_left, map, level_index);
            let sample_b = sample_from_texel(bottom_left, map, level_index);

            let alpha = wrapped_uv_as_fractional_texel.y - top_left.1;

            r = sample_a.0 as f32 + (sample_b.0 as f32 - sample_a.0 as f32) * alpha;
            g = sample_a.1 as f32 + (sample_b.1 as f32 - sample_a.1 as f32) * alpha;
            b = sample_a.2 as f32 + (sample_b.2 as f32 - sample_a.2 as f32) * alpha;
        }

        // Case: Two neighbors (right column)
        (None, Some(top_right), None, Some(bottom_right)) => {
            // Interpolate between top_right and bottom_right (based on uv.y).

            let sample_a = sample_from_texel(top_right, map, level_index);
            let sample_b = sample_from_texel(bottom_right, map, level_index);

            let alpha = wrapped_uv_as_fractional_texel.y - top_right.1;

            r = sample_a.0 as f32 + (sample_b.0 as f32 - sample_a.0 as f32) * alpha;
            g = sample_a.1 as f32 + (sample_b.1 as f32 - sample_a.1 as f32) * alpha;
            b = sample_a.2 as f32 + (sample_b.2 as f32 - sample_a.2 as f32) * alpha;
        }

        // Case: Two neighbors (top row)
        (Some(top_left), Some(top_right), None, None) => {
            // Interpolate between top_left and top_right (based on uv.x).

            let sample_a = sample_from_texel(top_left, map, level_index);
            let sample_b = sample_from_texel(top_right, map, level_index);

            let alpha = wrapped_uv_as_fractional_texel.x - top_left.0;

            r = sample_a.0 as f32 + (sample_b.0 as f32 - sample_a.0 as f32) * alpha;
            g = sample_a.1 as f32 + (sample_b.1 as f32 - sample_a.1 as f32) * alpha;
            b = sample_a.2 as f32 + (sample_b.2 as f32 - sample_a.2 as f32) * alpha;
        }

        // Case: Two neighbors (bottom row)
        (None, None, Some(bottom_left), Some(bottom_right)) => {
            // Interpolate between bottom_left and bottom_right (based on uv.x).

            let sample_a = sample_from_texel(bottom_left, map, level_index);
            let sample_b = sample_from_texel(bottom_right, map, level_index);

            let alpha = wrapped_uv_as_fractional_texel.x - bottom_left.0;

            r = sample_a.0 as f32 + (sample_b.0 as f32 - sample_a.0 as f32) * alpha;
            g = sample_a.1 as f32 + (sample_b.1 as f32 - sample_a.1 as f32) * alpha;
            b = sample_a.2 as f32 + (sample_b.2 as f32 - sample_a.2 as f32) * alpha;
        }

        // Case: 4 neighbors
        (Some(top_left), Some(top_right), Some(bottom_left), Some(bottom_right)) => {
            let alpha_x = wrapped_uv_as_fractional_texel.x - top_left.0;
            let alpha_y = wrapped_uv_as_fractional_texel.y - top_left.1;

            // 1. Interpolate between top_left and top_right (based on uv.x).
            let sample_a_1 = sample_from_texel(top_left, map, level_index);
            let sample_b_1 = sample_from_texel(top_right, map, level_index);

            let r_1 = sample_a_1.0 as f32 + (sample_b_1.0 as f32 - sample_a_1.0 as f32) * alpha_x;
            let g_1 = sample_a_1.1 as f32 + (sample_b_1.1 as f32 - sample_a_1.1 as f32) * alpha_x;
            let b_1 = sample_a_1.2 as f32 + (sample_b_1.2 as f32 - sample_a_1.2 as f32) * alpha_x;

            // 2. Interpolate between bottom_left and bottom_right (based on uv.x).

            let sample_a_2 = sample_from_texel(bottom_left, map, level_index);
            let sample_b_2 = sample_from_texel(bottom_right, map, level_index);

            let r_2 = sample_a_2.0 as f32 + (sample_b_2.0 as f32 - sample_a_2.0 as f32) * alpha_x;
            let g_2 = sample_a_2.1 as f32 + (sample_b_2.1 as f32 - sample_a_2.1 as f32) * alpha_x;
            let b_2 = sample_a_2.2 as f32 + (sample_b_2.2 as f32 - sample_a_2.2 as f32) * alpha_x;

            // 3. Interpolate between 2 interpolated samples (based on uv.y).

            r = r_1 + (r_2 - r_1) * alpha_y;
            g = g_1 + (g_2 - g_1) * alpha_y;
            b = b_1 + (b_2 - b_1) * alpha_y;
        }

        // Invalid case: Zero neighbors
        // Invalid case: Two diagonal neighbors
        // Invalid case: Three neighbors
        (_top_left_option, _top_right_option, _bottom_left_option, _bottom_right_option) => {
            r = 0.0;
            g = 255.0;
            b = 0.0;
        }
    }

    (r as u8, g as u8, b as u8)
}

pub fn sample_trilinear(
    uv: Vec2,
    map: &TextureMap,
    near_level_index: usize,
    far_level_index: usize,
    alpha: f32,
) -> (u8, u8, u8) {
    // Sample a color from both mipmaps, using bilinear sampling.

    if near_level_index == far_level_index {
        return sample_bilinear(uv, map, Some(near_level_index));
    } else if alpha == 0.0 {
        return sample_bilinear(uv, map, Some(near_level_index));
    } else if alpha >= 1.0 {
        return sample_bilinear(uv, map, Some(far_level_index));
    }

    let near_color = sample_bilinear(uv, map, Some(near_level_index));
    let far_color = sample_bilinear(uv, map, Some(far_level_index));

    let near_color_vec3 = Vec3 {
        x: near_color.0 as f32,
        y: near_color.1 as f32,
        z: near_color.2 as f32,
    };

    let far_color_vec3 = Vec3 {
        x: far_color.0 as f32,
        y: far_color.1 as f32,
        z: far_color.2 as f32,
    };

    let color = Vec3::interpolate(near_color_vec3, far_color_vec3, alpha);

    (color.x as u8, color.y as u8, color.z as u8)
}

fn sample_from_texel(
    texel: (f32, f32),
    map: &TextureMap,
    level_index: Option<usize>,
) -> (u8, u8, u8) {
    // Determine our map width based on the level index.

    let level_width = match level_index {
        Some(index) => map.width / (2 as u32).pow(index as u32),
        None => map.width,
    };

    let texel_color_index =
        map.get_bytes_per_pixel() * (texel.1 as u32 * level_width + texel.0 as u32) as usize;

    let buffer = match level_index {
        Some(index) => {
            if index >= map.levels.len() {
                panic!();
            }
            &map.levels[index]
        }
        None => &map.levels[0],
    };

    debug_assert!(texel_color_index < buffer.data.len());

    let r = buffer.data[texel_color_index];
    let g;
    let b;

    match map.info.storage_format {
        TextureMapStorageFormat::RGB24 | TextureMapStorageFormat::RGBA32 => {
            g = buffer.data[texel_color_index + 1];
            b = buffer.data[texel_color_index + 2];
        }
        TextureMapStorageFormat::Index8(_target_channel) => {
            g = r;
            b = r;
        }
    }

    (r, g, b)
}

pub fn get_neighbors(
    fractional_texel: Vec2,
    map: &TextureMap,
    level_index: Option<usize>,
) -> (
    Option<(f32, f32)>,
    Option<(f32, f32)>,
    Option<(f32, f32)>,
    Option<(f32, f32)>,
) {
    let fractional_x = fractional_texel.x - (fractional_texel.x as u32) as f32;
    let fractional_y = fractional_texel.y - (fractional_texel.y as u32) as f32;

    let nearest_x = (fractional_texel.x as u32) as f32;
    let nearest_y = (fractional_texel.y as u32) as f32;

    debug_assert!(
        fractional_x >= 0.0 && fractional_x < 1.0,
        "fractional_x is negative, or greater than 1! (fractional_x = {}).",
        fractional_x
    );
    debug_assert!(
        fractional_y >= 0.0 && fractional_y < 1.0,
        "fractional_y is negative, or greater than 1! (fractional_y = {}).",
        fractional_y
    );

    // Determine the UV's 4 closest (neighboring) texels.

    let top_left: (f32, f32);
    let top_right: (f32, f32);
    let bottom_left: (f32, f32);
    let bottom_right: (f32, f32);

    if fractional_x < 0.5 && fractional_y < 0.5 {
        // Bottom-right
        top_left = (nearest_x - 1.0, nearest_y - 1.0);
        top_right = (nearest_x, nearest_y - 1.0);
        bottom_left = (nearest_x - 1.0, nearest_y);
        bottom_right = (nearest_x, nearest_y);
    } else if fractional_x >= 0.5 && fractional_y < 0.5 {
        // Bottom-left
        top_left = (nearest_x, nearest_y - 1.0);
        top_right = (nearest_x + 1.0, nearest_y - 1.0);
        bottom_left = (nearest_x, nearest_y);
        bottom_right = (nearest_x + 1.0, nearest_y);
    } else if fractional_x < 0.5 {
        // Top-right
        top_left = (nearest_x - 1.0, nearest_y);
        top_right = (nearest_x, nearest_y);
        bottom_left = (nearest_x - 1.0, nearest_y + 1.0);
        bottom_right = (nearest_x, nearest_y + 1.0);
    } else {
        // Top-left
        top_left = (nearest_x, nearest_y);
        top_right = (nearest_x + 1.0, nearest_y);
        bottom_left = (nearest_x, nearest_y + 1.0);
        bottom_right = (nearest_x + 1.0, nearest_y + 1.0);
    }

    // Determine our map dimensions, based on the level index.
    let level_width = match level_index {
        Some(index) => (map.width / (2 as u32).pow(index as u32)) as f32,
        None => map.width as f32,
    };

    let level_height = match level_index {
        Some(index) => (map.height / (2 as u32).pow(index as u32)) as f32,
        None => map.height as f32,
    };

    match (map.options.wrapping, level_width == 1.0) {
        (TextureMapWrapping::Repeat, _) | (_, true) => {
            return (
                Some((top_left.0.rem(level_width), top_left.1.rem(level_height))),
                Some((top_right.0.rem(level_width), top_right.1.rem(level_height))),
                Some((
                    bottom_left.0.rem(level_width),
                    bottom_left.1.rem(level_height),
                )),
                Some((
                    bottom_right.0.rem(level_width),
                    bottom_right.1.rem(level_height),
                )),
            );
        }
        (_, false) => (
            if top_left.0 >= 0.0 && top_left.1 >= 0.0 {
                Some((top_left.0, top_left.1))
            } else {
                None
            },
            if top_right.0 < (level_width - 1.0) && top_right.1 >= 0.0 {
                Some((top_right.0, top_right.1))
            } else {
                None
            },
            if bottom_left.0 >= 0.0 && bottom_left.1 < (level_height - 1.0) {
                Some((bottom_left.0, bottom_left.1))
            } else {
                None
            },
            if bottom_right.0 < (level_width - 1.0) && bottom_right.1 < (level_height - 1.0) {
                Some((bottom_right.0, bottom_right.1))
            } else {
                None
            },
        ),
    }
}
