use crate::{
    buffer::Buffer2D,
    color,
    texture::{
        map::{TextureBuffer, TextureMap},
        sample::{sample_bilinear_u8, sample_bilinear_vec3},
    },
    vec::{vec2::Vec2, vec3::Vec3},
};

use super::SoftwareRenderer;

static BLOOM_STRENGTH: f32 = 0.03;

static DIRT_MASK_INTENSITY: f32 = 20.0;

impl SoftwareRenderer {
    pub(in crate::software_renderer) fn do_bloom_pass(&mut self) {
        match &self.framebuffer {
            Some(framebuffer_rc) => {
                let framebuffer = framebuffer_rc.borrow_mut();

                if let (Some(deferred_buffer_rc), Some(bloom_texture_map_rc)) = (
                    framebuffer.attachments.deferred_hdr.as_ref(),
                    framebuffer.attachments.bloom.as_ref(),
                ) {
                    let mut deferred_buffer = deferred_buffer_rc.borrow_mut();
                    let mut bloom_texture_map = bloom_texture_map_rc.borrow_mut();

                    // Blit the HDR color buffer to the bloom buffer's largest mipmap (level 0).

                    bloom_texture_map.levels[0]
                        .0
                        .copy(deferred_buffer.data.as_slice());

                    // Blur the bloom buffer.

                    do_physically_based_bloom(&mut bloom_texture_map);

                    // Blend our physically based bloom back into the color buffer.

                    let bloom_buffer = &mut bloom_texture_map.levels[0].0;

                    if let Some(handle) = self.options.bloom_dirt_mask_handle.as_ref() {
                        let texture_u8_arena = self.scene_resources.texture_u8.borrow();

                        if let Ok(entry) = texture_u8_arena.get(handle) {
                            let dirt_mask = &entry.item;

                            for x in 0..bloom_buffer.width {
                                for y in 0..bloom_buffer.height {
                                    let bloom = *bloom_buffer.get(x, y);

                                    if bloom.is_zero() {
                                        continue;
                                    }

                                    let dirt = {
                                        let uv = Vec2 {
                                            x: bloom_buffer.texel_size_over_2.x
                                                + x as f32 * bloom_buffer.texel_size.x,
                                            y: 1.0
                                                - (bloom_buffer.texel_size_over_2.y
                                                    + y as f32 * bloom_buffer.texel_size.y),
                                            z: 0.0,
                                        };

                                        let (r, _g, _b) = sample_bilinear_u8(uv, dirt_mask, None);

                                        r as f32 / 255.0 * DIRT_MASK_INTENSITY
                                    };

                                    bloom_buffer.set(x, y, bloom + bloom * dirt);
                                }
                            }
                        }
                    }

                    deferred_buffer.copy_lerp(bloom_buffer.data.as_slice(), BLOOM_STRENGTH);
                }
            }
            None => panic!(),
        }
    }
}

fn make_mipmaps(map: &mut TextureMap<Vec3>, levels: usize) {
    let (mut width, mut height) = (map.width, map.height);

    // Note: `levels` counts the number of mipmaps smaller than the source map.

    for level in 0..levels {
        width /= 2;
        height /= 2;

        assert!(
            width != 0,
            "Cannot produce {} mipmap levels for framebuffer of width {}!",
            levels,
            map.width
        );

        assert!(
            height != 0,
            "Cannot produce {} mipmap levels for framebuffer of height {}!",
            levels,
            map.height
        );

        let fill_value = match level {
            0 | 3 => color::RED.to_vec3() / 255.0,
            1 | 4 => color::GREEN.to_vec3() / 255.0,
            2 => color::BLUE.to_vec3() / 255.0,
            _ => panic!(),
        };

        let buffer = Buffer2D::new(width, height, Some(fill_value));

        let mipmap = TextureBuffer(buffer);

        map.levels.push(mipmap);
    }
}

fn do_physically_based_bloom(map: &mut TextureMap<Vec3>) {
    // 1. Ensure that mipmaps are present.

    static MIPMAP_LEVELS: usize = 4;

    if map.levels.len() == 1 {
        // Square dimensions are not required.

        make_mipmaps(map, MIPMAP_LEVELS);
    }

    // 2. Downsample.

    for level_index in 0..map.levels.len() - 1 {
        downsample(map, level_index);
    }

    // 3. Upsample.

    for level_index in (1..map.levels.len()).rev() {
        upsample(map, level_index);
    }

    // Final blur is stored in `map.levels[0]`.
}

fn downsample(map: &mut TextureMap<Vec3>, from_mipmap_index: usize) {
    let to_mipmap_index = from_mipmap_index + 1;

    let src_texel_size = map.levels[from_mipmap_index].0.texel_size;

    let (dest_width, dest_height, dst_texel_size, dst_texel_size_over_2) = {
        let dest = &map.levels[to_mipmap_index].0;

        (
            dest.width,
            dest.height,
            dest.texel_size,
            dest.texel_size_over_2,
        )
    };

    let level_index = Some(from_mipmap_index);

    for x in 0..dest_width {
        let pixel_center_uv_x = dst_texel_size_over_2.x + x as f32 * dst_texel_size.x;

        for y in 0..dest_height {
            let pixel_center_uv_y = 1.0 - (dst_texel_size_over_2.y + y as f32 * dst_texel_size.y);

            let center = Vec2 {
                x: pixel_center_uv_x,
                y: pixel_center_uv_y,
                z: 0.0,
            };

            let a = {
                let uv = center.offset(-2.0 * src_texel_size.x, 2.0 * src_texel_size.y);

                sample_bilinear_vec3(uv, map, level_index)
            };

            let b = {
                let uv = center.offset(0.0, 2.0 * src_texel_size.y);

                sample_bilinear_vec3(uv, map, level_index)
            };

            let c = {
                let uv = center.offset(2.0 * src_texel_size.x, 2.0 * src_texel_size.y);

                sample_bilinear_vec3(uv, map, level_index)
            };

            let d = {
                let uv = center.offset(-2.0 * src_texel_size.x, 0.0);

                sample_bilinear_vec3(uv, map, level_index)
            };

            let e = sample_bilinear_vec3(center, map, Some(from_mipmap_index));

            let f = {
                let uv = center.offset(2.0 * src_texel_size.x, 0.0);

                sample_bilinear_vec3(uv, map, level_index)
            };

            let g = {
                let uv = center.offset(-2.0 * src_texel_size.x, -2.0 * src_texel_size.y);

                sample_bilinear_vec3(uv, map, level_index)
            };

            let h = {
                let uv = center.offset(0.0, -2.0 * src_texel_size.y);

                sample_bilinear_vec3(uv, map, level_index)
            };

            let i = {
                let uv = center.offset(2.0 * src_texel_size.x, -2.0 * src_texel_size.y);

                sample_bilinear_vec3(uv, map, level_index)
            };

            let j = {
                let uv = center.offset(-src_texel_size.x, src_texel_size.y);

                sample_bilinear_vec3(uv, map, level_index)
            };

            let k = {
                let uv = center.offset(src_texel_size.x, src_texel_size.y);

                sample_bilinear_vec3(uv, map, level_index)
            };

            let l = {
                let uv = center.offset(-src_texel_size.x, -src_texel_size.y);

                sample_bilinear_vec3(uv, map, level_index)
            };

            let m = {
                let uv = center.offset(src_texel_size.x, -src_texel_size.y);

                sample_bilinear_vec3(uv, map, level_index)
            };

            let downsampled = {
                let mut sample = e * 0.125;

                sample += (a + c + g + i) * 0.03125;
                sample += (b + d + f + h) * 0.0625;
                sample += (j + k + l + m) * 0.125;

                sample.clamp_min(0.0001)
            };

            map.levels[to_mipmap_index].0.set(x, y, downsampled);
        }
    }
}

fn upsample(map: &mut TextureMap<Vec3>, from_mipmap_index: usize) {
    let to_mipmap_index = from_mipmap_index - 1;

    static FILTER_RADIUS: f32 = 0.005;

    let aspect_ratio = map.levels[0].0.width_over_height;

    let offset = Vec2 {
        x: FILTER_RADIUS,
        y: FILTER_RADIUS * aspect_ratio,
        z: 0.0,
    };

    let (dest_width, dest_height, dst_texel_size, dst_texel_size_over_2) = {
        let dest = &map.levels[to_mipmap_index].0;

        (
            dest.width,
            dest.height,
            dest.texel_size,
            dest.texel_size_over_2,
        )
    };

    let level_index = Some(from_mipmap_index);

    for x in 0..dest_width {
        let pixel_center_uv_x = dst_texel_size_over_2.x + x as f32 * dst_texel_size.x;

        for y in 0..dest_height {
            let pixel_center_uv_y = 1.0 - (dst_texel_size_over_2.y + y as f32 * dst_texel_size.y);

            let center = Vec2 {
                x: pixel_center_uv_x,
                y: pixel_center_uv_y,
                z: 0.0,
            };

            let a = {
                let uv = center.offset(-offset.x, offset.y);

                sample_bilinear_vec3(uv, map, level_index)
            };

            let b = {
                let uv = center.offset(0.0, offset.y);

                sample_bilinear_vec3(uv, map, level_index)
            };

            let c = {
                let uv = center + offset;

                sample_bilinear_vec3(uv, map, level_index)
            };

            let d = {
                let uv = center.offset(-offset.x, offset.y);

                sample_bilinear_vec3(uv, map, level_index)
            };

            let e = sample_bilinear_vec3(center, map, level_index);

            let f = {
                let uv = center.offset(offset.x, 0.0);

                sample_bilinear_vec3(uv, map, level_index)
            };

            let g = {
                let uv = center - offset;

                sample_bilinear_vec3(uv, map, level_index)
            };

            let h = {
                let uv = center.offset(0.0, -offset.y);

                sample_bilinear_vec3(uv, map, level_index)
            };

            let i = {
                let uv = center.offset(offset.x, -offset.y);

                sample_bilinear_vec3(uv, map, level_index)
            };

            let upsampled = {
                static ONE_OVER_SAMPLE_COUNT: f32 = 1.0 / 16.0;

                let mut sample = e * 4.0;

                sample += (b + d + f + h) * 2.0;
                sample += a + c + g + i;
                sample *= ONE_OVER_SAMPLE_COUNT;

                sample
            };

            map.levels[to_mipmap_index].0.set(x, y, upsampled);
        }
    }
}
