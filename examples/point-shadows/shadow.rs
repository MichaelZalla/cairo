use std::{cell::RefCell, rc::Rc};

use cairo::{
    buffer::{framebuffer::Framebuffer, Buffer2D},
    color::Color,
    render::Renderer,
    resource::handle::Handle,
    scene::{
        camera::Camera,
        context::SceneContext,
        light::point_light::{
            PointLight, POINT_LIGHT_SHADOW_CAMERA_FAR, POINT_LIGHT_SHADOW_CAMERA_NEAR,
        },
    },
    shader::context::ShaderContext,
    texture::{
        cubemap::{CubeMap, Side, CUBEMAP_SIDE_COLORS, CUBE_MAP_SIDES},
        map::{TextureMap, TextureMapWrapping},
    },
    vec::vec3::Vec3,
};

pub fn blit_shadow_map_horizontal_cross(shadow_map: &CubeMap<f32>, target: &mut Buffer2D) {
    let shadow_map_size = shadow_map.sides[0].width;

    for side in &CUBE_MAP_SIDES {
        let (side_index, block_coordinate) = (side.get_index(), side.get_block_coordinate(true));

        blit_shadow_map_side(
            side_index,
            &shadow_map.sides[side_index],
            block_coordinate.0 * shadow_map_size,
            block_coordinate.1 * shadow_map_size,
            target,
        );
    }
}

fn blit_shadow_map_side(
    side_index: usize,
    side: &TextureMap<f32>,
    x_offset: u32,
    y_offset: u32,
    target: &mut Buffer2D,
) {
    for y in 0..side.height {
        for x in 0..side.width {
            let mut output = CUBEMAP_SIDE_COLORS[side_index];

            let sampled_depth = side.levels[0].0.get(x, y) * 255.0;

            if sampled_depth > 0.0001 {
                output = Color::from_vec3(Vec3::ones() * sampled_depth);
            }

            target.set(x + x_offset, y + y_offset, output.to_u32());
        }
    }
}

fn render_point_shadows_to_cubemap(
    light: &PointLight,
    scene_context: &SceneContext,
    renderer_rc: &RefCell<dyn Renderer>,
    shader_context_rc: &RefCell<ShaderContext>,
    framebuffer_rc: &Rc<RefCell<Framebuffer>>,
) -> Result<CubeMap<f32>, String> {
    let mut shadow_map = {
        let mut shadow_map = CubeMap::<f32>::from_framebuffer(&framebuffer_rc.borrow());

        for side in CUBE_MAP_SIDES {
            let side_map = &mut shadow_map.sides[side.get_index()];

            side_map.sampling_options.wrapping = TextureMapWrapping::ClampToEdge;
        }

        shadow_map
    };

    let mut cubemap_face_camera = {
        let mut camera = Camera::from_perspective(light.position, Default::default(), 90.0, 1.0);

        // @NOTE(mzalla) Assumes the same near and far is set for the
        // framebuffer's depth attachment.

        camera.set_projection_z_near(POINT_LIGHT_SHADOW_CAMERA_NEAR);
        camera.set_projection_z_far(POINT_LIGHT_SHADOW_CAMERA_FAR);

        camera
    };

    {
        let mut shader_context = (*shader_context_rc).borrow_mut();

        cubemap_face_camera.update_shader_context(&mut shader_context);
    }

    for side in CUBE_MAP_SIDES {
        if let Side::Up = &side {
            continue;
        }

        cubemap_face_camera
            .look_vector
            .set_target_position(light.position + side.get_direction());

        {
            let mut shader_context = (*shader_context_rc).borrow_mut();

            shader_context
                .set_view_inverse_transform(cubemap_face_camera.get_view_inverse_transform());
        }

        let resources = scene_context.resources.borrow();
        let scenes = scene_context.scenes.borrow();

        let scene = &scenes[0];

        match scene.render(&resources, renderer_rc, None) {
            Ok(()) => {
                // Blit our framebuffer's HDR attachment buffer to our cubemap's
                // corresponding side (texture map).

                let framebuffer = framebuffer_rc.borrow();

                match &framebuffer.attachments.forward_or_deferred_hdr {
                    Some(hdr_attachment_rc) => {
                        let hdr_attachment = hdr_attachment_rc.borrow();

                        blit_hdr_attachment_to_cubemap_side(&hdr_attachment, &mut shadow_map.sides[side.get_index()]);
                    }
                    None => return Err("Called CubeMap::<f32>::render_scene() with a Framebuffer with no HDR attachment!".to_string()),
                }
            }
            Err(e) => panic!("{}", e),
        }
    }

    Ok(shadow_map)
}

fn blit_hdr_attachment_to_cubemap_side(
    hdr_buffer: &Buffer2D<Vec3>,
    cubemap_side: &mut TextureMap<f32>,
) {
    let buffer = &mut cubemap_side.levels[0].0;

    for y in 0..buffer.height {
        for x in 0..buffer.width {
            buffer.set(x, y, hdr_buffer.get(x, y).x);
        }
    }
}

pub fn update_point_light_shadow_maps(
    scene_context: &SceneContext,
    shadow_map_renderer_rc: &RefCell<dyn Renderer>,
    shadow_map_shader_context_rc: &RefCell<ShaderContext>,
    shadow_map_framebuffer_rc: &Rc<RefCell<Framebuffer>>,
) {
    // Render point shadow map.

    let mut point_light_shadow_maps: Vec<(Handle, CubeMap<f32>)> = vec![];

    {
        let resources = scene_context.resources.borrow();

        let point_light_arena = resources.point_light.borrow();

        for entry in point_light_arena.entries.iter().flatten() {
            let light = &entry.item;

            if let Some(handle) = light.shadow_map.as_ref() {
                let shadow_map = render_point_shadows_to_cubemap(
                    light,
                    scene_context,
                    shadow_map_renderer_rc,
                    shadow_map_shader_context_rc,
                    shadow_map_framebuffer_rc,
                )
                .unwrap();

                point_light_shadow_maps.push((*handle, shadow_map));
            }
        }
    }

    {
        let resources = scene_context.resources.borrow();

        let mut cubemap_f32_arena = resources.cubemap_f32.borrow_mut();

        for (handle, cubemap) in &point_light_shadow_maps {
            if let Ok(entry) = cubemap_f32_arena.get_mut(handle) {
                cubemap.clone_into(&mut entry.item);
            }
        }
    }
}
