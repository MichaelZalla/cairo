use crate::vec::vec2::Vec2;

use super::TextureMap;

pub fn sample_nearest(uv: Vec2, map: &TextureMap) -> (u8, u8, u8) {
    debug_assert!(map.is_loaded);

    // Wraps UV coordinates into the range [0.0, 1.0).
    let wrapped_uv = Vec2 {
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
    };

    debug_assert!(
        map.pixel_data.len()
            == (map.width * map.height * TextureMap::BYTES_PER_PIXEL as u32) as usize
    );

    // Maps the wrapped UV coordinate to the nearest whole texel coordinate.

    let texel_x = (1.0 - wrapped_uv.x) * (map.width - 1) as f32;
    let texel_y = (1.0 - wrapped_uv.y) * (map.height - 1) as f32;

    return sample_from_texel((texel_x, texel_y), map);
}

pub fn sample_bilinear(uv: Vec2, map: &TextureMap) -> (u8, u8, u8) {
    debug_assert!(map.is_loaded);

    // Wraps UV coordinates into the range [0.0, 1.0).
    let uv_safe = Vec2 {
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
    };

    debug_assert!(
        map.pixel_data.len()
            == (map.width * map.height * TextureMap::BYTES_PER_PIXEL as u32) as usize
    );

    // Maps the wrapped UV coordinate to a fractional texel coordinate.
    let wrapped_uv_as_fractional_texel = Vec2 {
        x: ((1.0 - uv_safe.x) * (map.width - 1) as f32),
        y: ((1.0 - uv_safe.y) * (map.height - 1) as f32),
        z: 1.0,
    };

    // Performs bilinear filtering (interpolation)

    // See: https://en.wikipedia.org/wiki/Bilinear_interpolation#Computation
    // See: https://www.youtube.com/watch?v=AqscP7rc8_M

    let r: f32;
    let g: f32;
    let b: f32;

    let nearest_neighbors = get_neighbors(wrapped_uv_as_fractional_texel, map);

    match nearest_neighbors {
        // Case: One neighbor (top-left)
        (Some(texel), None, None, None) |

        // Case: One neighbor (top-right)
        (None, Some(texel), None, None) |

        // Case: One neighbor (bottom-left)
        (None, None, Some(texel), None) |

        // Case: One neighbor (bottom-right)
        (None, None, None, Some(texel)) => return sample_from_texel(texel, map),

        // Case: Two neighbors (left column)
        (Some(top_left), None, Some(bottom_left), None) => {
            // Interpolate between top_left and bottom_left (based on uv.y).

            let sample_a = sample_from_texel(top_left, map);
            let sample_b = sample_from_texel(bottom_left, map);

            let alpha = wrapped_uv_as_fractional_texel.y - top_left.1;

            r = sample_a.0 as f32 + (sample_b.0 as f32 - sample_a.0 as f32) * alpha;
            g = sample_a.1 as f32 + (sample_b.1 as f32 - sample_a.1 as f32) * alpha;
            b = sample_a.2 as f32 + (sample_b.2 as f32 - sample_a.2 as f32) * alpha;
        }

        // Case: Two neighbors (right column)
        (None, Some(top_right), None, Some(bottom_right)) => {
            // Interpolate between top_right and bottom_right (based on uv.y).

            let sample_a = sample_from_texel(top_right, map);
            let sample_b = sample_from_texel(bottom_right, map);

            let alpha = wrapped_uv_as_fractional_texel.y - top_right.1;

            r = sample_a.0 as f32 + (sample_b.0 as f32 - sample_a.0 as f32) * alpha;
            g = sample_a.1 as f32 + (sample_b.1 as f32 - sample_a.1 as f32) * alpha;
            b = sample_a.2 as f32 + (sample_b.2 as f32 - sample_a.2 as f32) * alpha;
        }

        // Case: Two neighbors (top row)
        (Some(top_left), Some(top_right), None, None) => {
            // Interpolate between top_left and top_right (based on uv.x).

            let sample_a = sample_from_texel(top_left, map);
            let sample_b = sample_from_texel(top_right, map);

            let alpha = wrapped_uv_as_fractional_texel.x - top_left.0;

            r = sample_a.0 as f32 + (sample_b.0 as f32 - sample_a.0 as f32) * alpha;
            g = sample_a.1 as f32 + (sample_b.1 as f32 - sample_a.1 as f32) * alpha;
            b = sample_a.2 as f32 + (sample_b.2 as f32 - sample_a.2 as f32) * alpha;
        }

        // Case: Two neighbors (bottom row)
        (None, None, Some(bottom_left), Some(bottom_right)) => {
            // Interpolate between bottom_left and bottom_right (based on uv.x).

            let sample_a = sample_from_texel(bottom_left, map);
            let sample_b = sample_from_texel(bottom_right, map);

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
            let sample_a_1 = sample_from_texel(top_left, map);
            let sample_b_1 = sample_from_texel(top_right, map);

            let r_1 = sample_a_1.0 as f32 + (sample_b_1.0 as f32 - sample_a_1.0 as f32) * alpha_x;
            let g_1 = sample_a_1.1 as f32 + (sample_b_1.1 as f32 - sample_a_1.1 as f32) * alpha_x;
            let b_1 = sample_a_1.2 as f32 + (sample_b_1.2 as f32 - sample_a_1.2 as f32) * alpha_x;

            // 2. Interpolate between bottom_left and bottom_right (based on uv.x).

            let sample_a_2 = sample_from_texel(bottom_left, map);
            let sample_b_2 = sample_from_texel(bottom_right, map);

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

fn sample_from_texel(texel: (f32, f32), map: &TextureMap) -> (u8, u8, u8) {
    let texel_color_index =
        TextureMap::BYTES_PER_PIXEL * (texel.1 as u32 * map.width + texel.0 as u32) as usize;

    let pixels = &map.pixel_data;

    debug_assert!(texel_color_index < pixels.len());

    return (
        pixels[texel_color_index],
        pixels[texel_color_index + 1],
        pixels[texel_color_index + 2],
    );
}

pub fn get_neighbors(
    fractional_texel: Vec2,
    map: &TextureMap,
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

    // Determine each neighboring texel's contribution.

    (
        if top_left.0 >= 0.0 && top_left.1 >= 0.0 {
            Some((top_left.0, top_left.1))
        } else {
            None
        },
        if top_right.0 < (map.width - 1) as f32 && top_right.1 >= 0.0 {
            Some((top_right.0, top_right.1))
        } else {
            None
        },
        if bottom_left.0 >= 0.0 && bottom_left.1 < (map.height - 1) as f32 {
            Some((bottom_left.0, bottom_left.1))
        } else {
            None
        },
        if bottom_right.0 < (map.width - 1) as f32 && bottom_right.1 < (map.height - 1) as f32 {
            Some((bottom_right.0, bottom_right.1))
        } else {
            None
        },
    )
}
