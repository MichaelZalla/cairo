use std::{cell::RefCell, rc::Rc};

use cairo::{
    buffer::{framebuffer::Framebuffer, Buffer2D},
    render::Renderer,
    scene::{
        context::SceneContext,
        light::directional_light::{DirectionalLight, SHADOW_MAP_CAMERA_COUNT},
    },
    shader::context::ShaderContext,
    texture::map::{TextureMap, TextureMapWrapping},
};

pub fn render_shadow_maps(
    light: &DirectionalLight,
    scene_context: &SceneContext,
    shader_context_rc: &Rc<RefCell<ShaderContext>>,
    framebuffer_rc: &Rc<RefCell<Framebuffer>>,
    renderer: &RefCell<dyn Renderer>,
) -> Result<[TextureMap<f32>; SHADOW_MAP_CAMERA_COUNT], String> {
    let (width, height) = {
        let framebuffer = framebuffer_rc.borrow();

        debug_assert_eq!(framebuffer.width, framebuffer.height);

        (framebuffer.width, framebuffer.width)
    };

    let blank_texture =
        TextureMap::<f32>::from_buffer(width, height, Buffer2D::<f32>::new(width, height, None));

    let mut maps: [TextureMap<f32>; SHADOW_MAP_CAMERA_COUNT] =
        [blank_texture.clone(), blank_texture.clone(), blank_texture];

        match light.shadow_map_cameras.as_ref() {
            Some(cameras) => {
                for (depth_index, (_far_z, camera)) in cameras.iter().enumerate() {
                    let map = &mut maps[depth_index];

                    {
                        let framebuffer = framebuffer_rc.borrow_mut();

                        match framebuffer.attachments.depth.as_ref() {
                            Some(attachment) => {
                                let mut zbuffer = attachment.borrow_mut();
                
                                zbuffer.set_projection_z_near(camera.get_projection_z_near());
                                zbuffer.set_projection_z_far(camera.get_projection_z_far());
                            }
                            None => panic!(),
                        }
                    }

                    {
                        let mut shader_context = shader_context_rc.borrow_mut();
    
                        shader_context.set_directional_light_view_projection_index(Some(depth_index));
                
                        camera.update_shader_context(&mut shader_context);
                    }

                    let resources = &scene_context.resources;
                    let scenes = scene_context.scenes.borrow();

                    let scene = &scenes[0];
                
                    match scene.render(resources, renderer, None) {
                        Ok(()) => {
                            // Blit our framebuffer's color attachment buffer to
                            // our cubemap face texture.
                
                            let framebuffer = framebuffer_rc.borrow();
                
                            match &framebuffer.attachments.forward_or_deferred_hdr {
                                Some(hdr_attachment_rc) => {
                                    let hdr_attachment = hdr_attachment_rc.borrow();
                
                                    map.sampling_options.wrapping = TextureMapWrapping::ClampToEdge;
                
                                    let buffer = &mut map.levels[0].0;
                
                                    for y in 0..buffer.height {
                                        for x in 0..buffer.width {
                                            buffer.set(x, y, hdr_attachment.get(x, y).x);
                                        }
                                    }
                                }
                                None => return Err(
                                    "Called CubeMap::<f32>::render_scene() with a Framebuffer with no HDR attachment!".to_string()
                                ),
                            }
                        }
                        Err(e) => panic!("{}", e),
                    }
                }
            },
            None => panic!(),
        }
    
        Ok(maps)
}
