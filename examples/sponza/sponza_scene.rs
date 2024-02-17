use std::{borrow::BorrowMut, sync::RwLock};

use sdl2::keyboard::Keycode;

use cairo::{
    app::App,
    buffer::framebuffer::Framebuffer,
    color,
    context::ApplicationRenderingContext,
    debug::message::DebugMessageBuffer,
    device::{GameControllerState, KeyboardState, MouseState},
    entity::Entity,
    font::{cache::FontCache, FontInfo},
    graphics::Graphics,
    material::cache::MaterialCache,
    pipeline::{options::PipelineOptions, zbuffer::DepthTestMethod, Pipeline},
    scene::{
        camera::Camera,
        light::{AmbientLight, DirectionalLight, PointLight, SpotLight},
        Scene,
    },
    shader::{fragment::FragmentShaderFn, ShaderContext},
    shaders::{
        debug_shaders::{
            albedo_fragment_shader::AlbedoFragmentShader,
            depth_fragment_shader::DepthFragmentShader,
            normal_fragment_shader::NormalFragmentShader,
            specular_intensity_fragment_shader::SpecularIntensityFragmentShader,
            uv_test_fragment_shader::UvTestFragmentShader,
        },
        default_fragment_shader::DEFAULT_FRAGMENT_SHADER,
        // default_geometry_shader::DEFAULT_GEOMETRY_SHADER,
        default_vertex_shader::DEFAULT_VERTEX_SHADER,
    },
    texture::{cubemap::CubeMap, map::TextureMapStorageFormat},
    vec::{
        vec3::{self, Vec3},
        vec4::Vec4,
    },
};

static SPONZA_CENTER: Vec3 = Vec3 {
    x: -572.3847 + 500.0,
    y: 233.06613,
    z: -43.05618,
};

pub struct SponzaScene<'a> {
    framebuffer_rwl: &'a RwLock<Framebuffer>,
    font_cache_rwl: &'static RwLock<FontCache<'static>>,
    font_info: &'static FontInfo,
    debug_message_buffer: DebugMessageBuffer,
    pipeline: Pipeline<'a>,
    fragment_shaders: Vec<FragmentShaderFn>,
    active_fragment_shader_index: usize,
    cameras: Vec<Camera>,
    active_camera_index: usize,
    directional_light: DirectionalLight,
    point_lights: Vec<PointLight>,
    spot_lights: Vec<SpotLight>,
    entities: &'a RwLock<Vec<Entity<'a>>>,
    skybox: CubeMap,
    materials: &'a mut MaterialCache,
    shader_context: &'a RwLock<ShaderContext>,
}

impl<'a> SponzaScene<'a> {
    pub fn new(
        framebuffer_rwl: &'a RwLock<Framebuffer>,
        font_cache_rwl: &'static RwLock<FontCache<'static>>,
        font_info: &'static FontInfo,
        rendering_context: &ApplicationRenderingContext,
        entities: &'a RwLock<Vec<Entity<'a>>>,
        materials: &'a mut MaterialCache,
        shader_context: &'a RwLock<ShaderContext>,
    ) -> Self {
        let framebuffer = framebuffer_rwl.read().unwrap();

        let vertex_shader = DEFAULT_VERTEX_SHADER;

        let fragment_shaders = vec![
            DEFAULT_FRAGMENT_SHADER,
            AlbedoFragmentShader,
            DepthFragmentShader,
            NormalFragmentShader,
            SpecularIntensityFragmentShader,
            UvTestFragmentShader,
        ];

        let active_fragment_shader_index: usize = 0;

        let debug_message_buffer: DebugMessageBuffer = Default::default();

        // Set up a camera for rendering our scene

        let aspect_ratio = framebuffer.width_over_height;

        let camera_position = Vec3 {
            x: 1000.0,
            y: 300.0,
            z: 0.0,
        };

        // Set up a camera for rendering our scene
        let mut camera: Camera = Camera::from_perspective(
            camera_position,
            camera_position + vec3::LEFT,
            75.0,
            aspect_ratio,
        );

        camera.movement_speed = 300.0;

        camera.set_projection_z_far(10000.0);

        // Define lights for our scene

        let ambient_light = AmbientLight {
            intensities: Vec3::ones() * 0.1,
        };

        let directional_light = DirectionalLight {
            intensities: Vec3::ones() * 0.1,
            direction: Vec4::new(vec3::UP * -1.0, 1.0).as_normal(),
        };

        let mut point_light = PointLight::new();

        point_light.intensities = color::BLUE.to_vec3() / 255.0 * 15.0;

        point_light.specular_intensity = 1.0;

        point_light.constant_attenuation = 1.0;
        point_light.linear_attenuation = 0.007;
        point_light.quadratic_attenuation = 0.0002;

        let mut spot_light = SpotLight::new();

        spot_light.intensities = color::RED.to_vec3() / 255.0 * 15.0;

        spot_light.constant_attenuation = 1.0;
        spot_light.linear_attenuation = 0.007;
        spot_light.quadratic_attenuation = 0.0002;

        let mut skybox = CubeMap::cross(
            "examples/skybox/assets/cross/horizontal_cross.png",
            TextureMapStorageFormat::RGB24,
        );

        skybox.load(rendering_context).unwrap();

        let pipeline_options: PipelineOptions = Default::default();

        let view_position = Vec4::new(camera.look_vector.get_position(), 1.0);

        let view_inverse_transform = camera.get_view_inverse_transform();

        let projection_transform = camera.get_projection();

        let mut context = shader_context.write().unwrap();

        context.set_view_position(view_position);
        context.set_view_inverse_transform(view_inverse_transform);
        context.set_projection(projection_transform);

        context.set_ambient_light(ambient_light);
        context.set_directional_light(directional_light);
        context.set_point_light(0, point_light);
        context.set_spot_light(0, spot_light);

        let mut pipeline = Pipeline::new(
            shader_context,
            vertex_shader,
            fragment_shaders[active_fragment_shader_index],
            pipeline_options,
        );

        pipeline.geometry_shader_options.diffuse_mapping_active = false;
        pipeline.geometry_shader_options.specular_mapping_active = true;
        pipeline.geometry_shader_options.normal_mapping_active = true;

        return SponzaScene {
            framebuffer_rwl,
            font_cache_rwl,
            font_info,
            debug_message_buffer,
            pipeline,
            fragment_shaders,
            active_fragment_shader_index,
            entities,
            skybox,
            materials,
            shader_context,
            cameras: vec![camera],
            active_camera_index: 0,
            directional_light,
            point_lights: vec![point_light],
            spot_lights: vec![spot_light],
        };
    }
}

impl<'a> Scene for SponzaScene<'a> {
    fn update(
        &mut self,
        app: &App,
        keyboard_state: &KeyboardState,
        mouse_state: &MouseState,
        game_controller_state: &GameControllerState,
    ) {
        let mut context = self.shader_context.write().unwrap();

        let camera = (self.cameras[self.active_camera_index]).borrow_mut();

        camera.update(
            &app.timing_info,
            keyboard_state,
            mouse_state,
            game_controller_state,
        );

        for keycode in &keyboard_state.keys_pressed {
            match keycode {
                Keycode::I { .. } => {
                    let framebuffer = self.framebuffer_rwl.write().unwrap();

                    let mut depth_buffer = framebuffer
                        .attachments
                        .depth
                        .as_ref()
                        .unwrap()
                        .write()
                        .unwrap();

                    let methods = vec![
                        DepthTestMethod::Always,
                        DepthTestMethod::Never,
                        DepthTestMethod::Less,
                        DepthTestMethod::Equal,
                        DepthTestMethod::LessThanOrEqual,
                        DepthTestMethod::Greater,
                        DepthTestMethod::NotEqual,
                        DepthTestMethod::GreaterThanOrEqual,
                    ];

                    let mut index = methods
                        .iter()
                        .position(|&method| method == *(depth_buffer.get_depth_test_method()))
                        .unwrap();

                    index = if index == (methods.len() - 1) {
                        0
                    } else {
                        index + 1
                    };

                    depth_buffer.set_depth_test_method(methods[index])
                }
                Keycode::H { .. } => {
                    self.active_fragment_shader_index += 1;

                    if self.active_fragment_shader_index == self.fragment_shaders.len() {
                        self.active_fragment_shader_index = 0;
                    }

                    self.pipeline.set_fragment_shader(
                        self.fragment_shaders[self.active_fragment_shader_index],
                    );
                }
                _ => {}
            }
        }

        self.pipeline
            .options
            .update(keyboard_state, mouse_state, game_controller_state);

        self.pipeline.geometry_shader_options.update(
            keyboard_state,
            mouse_state,
            game_controller_state,
        );

        context.set_view_position(Vec4::new(camera.look_vector.get_position(), 1.0));

        context.set_projection(camera.get_projection());

        let uptime = app.timing_info.uptime_seconds;

        self.directional_light.direction = Vec4::new(
            Vec3 {
                x: uptime.sin(),
                y: -1.0,
                z: uptime.cos(),
            },
            1.0,
        )
        .as_normal();

        context.set_directional_light(self.directional_light);

        self.point_lights[0].position = SPONZA_CENTER
            + Vec3 {
                x: 1000.0 * (uptime).sin(),
                y: 300.0,
                z: 0.0,
            };

        context.set_point_light(0, self.point_lights[0]);

        self.spot_lights[0].look_vector.set_position(
            SPONZA_CENTER
                + Vec3 {
                    x: -1000.0 * (uptime).sin(),
                    y: 500.0,
                    z: 0.0,
                },
        );

        context.set_spot_light(0, self.spot_lights[0]);

        let camera_view_inverse_transform = camera.get_view_inverse_transform();

        context.set_view_inverse_transform(camera_view_inverse_transform);

        // Write to debug log

        self.debug_message_buffer.write(format!(
            "Resolution: {}x{}",
            app.window_info.canvas_width, app.window_info.canvas_height
        ));

        self.debug_message_buffer
            .write(format!("FPS: {:.*}", 0, app.timing_info.frames_per_second));

        self.debug_message_buffer
            .write(format!("Seconds ellapsed: {:.*}", 2, uptime));

        self.debug_message_buffer.write(format!(
            "Camera position: {}",
            self.cameras[self.active_camera_index]
                .look_vector
                .get_position()
        ));

        self.debug_message_buffer.write(format!(
            "Wireframe: {}",
            if self.pipeline.options.do_wireframe {
                "On"
            } else {
                "Off"
            }
        ));

        self.debug_message_buffer.write(format!(
            "Rasterized geometry: {}",
            if self.pipeline.options.do_rasterized_geometry {
                "On"
            } else {
                "Off"
            }
        ));

        if self.pipeline.options.do_rasterized_geometry {
            self.debug_message_buffer.write(format!(
                "Culling reject mask: {:?}",
                self.pipeline.options.face_culling_strategy.reject
            ));

            self.debug_message_buffer.write(format!(
                "Culling window order: {:?}",
                self.pipeline.options.face_culling_strategy.winding_order
            ));

            {
                let framebuffer = self.framebuffer_rwl.read().unwrap();

                let depth_buffer = framebuffer
                    .attachments
                    .depth
                    .as_ref()
                    .unwrap()
                    .read()
                    .unwrap();

                self.debug_message_buffer.write(format!(
                    "Depth test method: {:?}",
                    depth_buffer.get_depth_test_method()
                ));
            }

            self.debug_message_buffer.write(format!(
                "Lighting: {}",
                if self.pipeline.options.do_lighting {
                    "On"
                } else {
                    "Off"
                }
            ));

            self.debug_message_buffer.write(format!(
                "Fragment shader: {}",
                [
                    "DEFAULT_FRAGMENT_SHADER",
                    "AlbedoFragmentShader",
                    "DepthFragmentShader",
                    "NormalFragmentShader",
                    "SpecularIntensityFragmentShader",
                    "UvTestFragmentShader",
                ][self.active_fragment_shader_index]
            ));
        }
    }

    fn render(&mut self) {
        self.pipeline.bind_framebuffer(Some(&self.framebuffer_rwl));

        self.pipeline.begin_frame();

        let camera = self.cameras[self.active_camera_index];

        for entity in self.entities.read().unwrap().as_slice() {
            self.pipeline.render_entity(&entity, Some(self.materials));
        }

        self.pipeline.render_point_light(
            &self.point_lights[0],
            Some(&camera),
            Some(&mut self.materials),
        );

        self.pipeline.render_spot_light(
            &self.spot_lights[0],
            Some(&camera),
            Some(&mut self.materials),
        );

        self.pipeline.render_skybox(&self.skybox, &camera);

        self.pipeline.end_frame();

        // Render debug messages

        {
            let framebuffer = self.framebuffer_rwl.write().unwrap();

            let mut color_buffer = framebuffer
                .attachments
                .color
                .as_ref()
                .unwrap()
                .write()
                .unwrap();

            let debug_messages = self.debug_message_buffer.borrow_mut();

            {
                Graphics::render_debug_messages(
                    &mut *color_buffer,
                    self.font_cache_rwl,
                    self.font_info,
                    (12, 12),
                    1.0,
                    debug_messages,
                );
            }
        }
    }
}
