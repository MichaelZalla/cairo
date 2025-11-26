use cairo::{
    buffer::Buffer2D,
    color::Color,
    texture::{
        cubemap::{CUBE_MAP_SIDES, CUBEMAP_SIDE_COLORS, CubeMap},
        map::TextureMap,
        sample::sample_nearest_f32,
    },
    vec::{vec2::Vec2, vec3},
};

pub fn debug_blit_shadow_map_horizontal_cross(shadow_map: &CubeMap<f32>, target: &mut Buffer2D) {
    fn debug_blit_shadow_map_side(
        side_index: usize,
        side: &TextureMap<f32>,
        x_offset: u32,
        y_offset: u32,
        target: &mut Buffer2D,
    ) {
        let uv_step = 1.0 / side.width as f32;

        for y in 0..side.height {
            for x in 0..side.width {
                let uv = Vec2 {
                    x: x as f32 * uv_step,
                    y: 1.0 - y as f32 * uv_step,
                    z: 0.0,
                };

                let depth_sample = sample_nearest_f32(uv, side);

                let depth_sample_u8 = depth_sample * 255.0;

                let color = if depth_sample > 0.0001 {
                    Color::from_vec3(vec3::ONES * depth_sample_u8)
                } else {
                    CUBEMAP_SIDE_COLORS[side_index]
                };

                target.set(x + x_offset, y + y_offset, color.to_u32());
            }
        }
    }

    let shadow_map_size = shadow_map.sides[0].width;

    for side in &CUBE_MAP_SIDES {
        let (side_index, block_coordinate) = (side.get_index(), side.get_block_coordinate(true));

        debug_blit_shadow_map_side(
            side_index,
            &shadow_map.sides[side_index],
            block_coordinate.0 * shadow_map_size,
            block_coordinate.1 * shadow_map_size,
            target,
        );
    }
}
