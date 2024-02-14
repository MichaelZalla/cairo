use self::map::TextureMap;

pub mod cubemap;
pub mod map;
pub mod sample;
pub mod uv;

fn get_half_scaled(half_scaled_dimension: u32, pixel_data: &Vec<u8>) -> Vec<u8> {
    let mut result: Vec<u8> = vec![];

    let full_scale_stride = half_scaled_dimension as usize * 2 * TextureMap::BYTES_PER_PIXEL;

    let half_scale_stride = half_scaled_dimension as usize * TextureMap::BYTES_PER_PIXEL;
    let half_scaled_pixel_count = half_scaled_dimension as usize * half_scaled_dimension as usize;

    result.resize(half_scaled_pixel_count * TextureMap::BYTES_PER_PIXEL, 255);

    for small_y in 0..half_scaled_dimension as usize {
        for small_x in 0..half_scaled_dimension as usize {
            let big_y = small_y * 2;
            let big_x = small_x * 2;

            let mut r: u32 = 0;
            let mut g: u32 = 0;
            let mut b: u32 = 0;

            let top_left = big_y * full_scale_stride + big_x * TextureMap::BYTES_PER_PIXEL;
            let top_right = top_left + TextureMap::BYTES_PER_PIXEL;
            let bottom_left = top_left + full_scale_stride;
            let bottom_right = bottom_left + TextureMap::BYTES_PER_PIXEL;

            for index in [top_left, top_right, bottom_left, bottom_right].iter() {
                r += pixel_data[*index] as u32;
                g += pixel_data[*index + 1] as u32;
                b += pixel_data[*index + 2] as u32;
            }

            let half_scaled_index =
                small_y * half_scale_stride + small_x * TextureMap::BYTES_PER_PIXEL;

            let r_u8 = (r as f32 / 4.0) as u8;
            let g_u8 = (g as f32 / 4.0) as u8;
            let b_u8 = (b as f32 / 4.0) as u8;

            result[half_scaled_index] = r_u8;
            result[half_scaled_index + 1] = g_u8;
            result[half_scaled_index + 2] = b_u8;
        }
    }

    return result;
}
