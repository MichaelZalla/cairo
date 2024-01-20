use std::{borrow::BorrowMut, f32::consts::PI, sync::RwLock};

use sdl2::keyboard::Keycode;

use cairo::{
    device::{GameControllerState, KeyboardState, MouseState},
    entity::Entity,
    graphics::{pixelbuffer::PixelBuffer, Graphics},
    material::cache::MaterialCache,
    matrix::Mat4,
    pipeline::{options::PipelineOptions, Pipeline},
    scene::{
        camera::Camera,
        light::{AmbientLight, DirectionalLight, PointLight, SpotLight},
        Scene,
    },
    shader::{fragment::FragmentShader, ShaderContext},
    shaders::{
        default_fragment_shader::DefaultFragmentShader, default_vertex_shader::DefaultVertexShader,
    },
    vec::{vec3::Vec3, vec4::Vec4},
};

pub struct GeneratePrimitivesScene<'a> {
    pipeline: Pipeline<'a>,
    cameras: Vec<Camera>,
    active_camera_index: usize,
    directional_light: DirectionalLight,
    point_lights: Vec<PointLight>,
    spot_lights: Vec<SpotLight>,
    entities: &'a RwLock<Vec<&'a mut Entity<'a>>>,
    material_cache: &'a mut MaterialCache,
    shader_context: &'a RwLock<ShaderContext>,
    prev_mouse_state: MouseState,
    looking_at_point_light: bool,
    seconds_ellapsed: f32,
}

impl<'a> GeneratePrimitivesScene<'a> {
    pub fn new(
        canvas_width: u32,
        canvas_height: u32,
        entities: &'a RwLock<Vec<&'a mut Entity<'a>>>,
        material_cache: &'a mut MaterialCache,
        shader_context: &'a RwLock<ShaderContext>,
    ) -> Self {
        let graphics = Graphics {
            buffer: PixelBuffer::new(canvas_width, canvas_height),
        };

        let aspect_ratio = graphics.buffer.width_over_height;

        // Set up a camera for rendering our scene
        let mut camera: Camera = Camera::new(
            aspect_ratio,
            Vec3 {
                x: 15.0,
                y: 8.0,
                z: -15.0,
            },
            Vec3 {
                x: 0.0,
                y: 0.0,
                z: -7.5,
            }
            .as_normal(),
        );

        camera.movement_speed = 10.0;

        let camera2: Camera = Camera::new(
            aspect_ratio,
            Vec3 {
                x: 4.0,
                y: 8.0,
                z: -4.0,
            },
            Vec3 {
                x: 0.0,
                y: 0.0,
                z: -7.5,
            }
            .as_normal(),
        );

        // Define lights for our scene
        let ambient_light = AmbientLight {
            intensities: Vec3 {
                x: 0.1,
                y: 0.1,
                z: 0.1,
            },
        };

        let directional_light = DirectionalLight {
            intensities: Vec3 {
                x: 0.15,
                y: 0.15,
                z: 0.15,
            },
            direction: Vec4 {
                x: -1.0,
                y: -1.0,
                z: 1.0,
                w: 1.0,
            }
            .as_normal(),
        };

        let mut point_lights: Vec<PointLight> = vec![];

        let light_grid_subdivisions: usize = 4;
        let light_grid_size = 20.0;

        for x in 0..(light_grid_subdivisions + 1) {
            for z in 0..(light_grid_subdivisions + 1) {
                let mut light = PointLight::new();

                light.position = Vec3 {
                    x: -(light_grid_size / 2.0)
                        + (x as f32 / light_grid_subdivisions as f32) * light_grid_size,
                    y: 1.0,
                    z: -(light_grid_size / 2.0)
                        + (z as f32 / light_grid_subdivisions as f32) * light_grid_size,
                };

                point_lights.push(light);
            }
        }

        let mut spot_lights: Vec<SpotLight> = vec![SpotLight::new()];

        spot_lights[0].position = Vec3 {
            x: -6.0,
            y: 15.0,
            z: -6.0,
        };

        let pipeline_options: PipelineOptions = Default::default();

        let view_position = Vec4::new(camera.get_position(), 1.0);

        let view_inverse_transform = camera.get_view_inverse_transform();

        let projection_transform = camera.get_projection();

        let mut context = shader_context.write().unwrap();

        context.set_camera_position(view_position);
        context.set_view_inverse_transform(view_inverse_transform);
        context.set_projection(projection_transform);

        context.set_ambient_light(ambient_light);
        context.set_directional_light(directional_light);

        for (index, light) in point_lights.iter().enumerate() {
            context.set_point_light(index, *light);
        }

        context.set_spot_light(0, spot_lights[0]);

        let vertex_shader = DefaultVertexShader {
            context: shader_context,
        };

        let fragment_shader = DefaultFragmentShader::new(shader_context, None);

        let pipeline = Pipeline::new(
            graphics,
            camera.get_projection_z_near(),
            camera.get_projection_z_far(),
            shader_context,
            vertex_shader,
            fragment_shader,
            pipeline_options,
        );

        return GeneratePrimitivesScene {
            pipeline,
            entities,
            material_cache,
            shader_context,
            cameras: vec![camera, camera2],
            active_camera_index: 0,
            directional_light,
            point_lights,
            spot_lights,
            prev_mouse_state: MouseState::new(),
            looking_at_point_light: false,
            seconds_ellapsed: 0.0,
        };
    }
}

impl<'a> Scene for GeneratePrimitivesScene<'a> {
    fn update(
        &mut self,
        keyboard_state: &KeyboardState,
        mouse_state: &MouseState,
        game_controller_state: &GameControllerState,
        seconds_since_last_update: f32,
    ) {
        let mut context = self.shader_context.write().unwrap();

        self.seconds_ellapsed += seconds_since_last_update;

        let camera = (self.cameras[self.active_camera_index]).borrow_mut();

        for keycode in &keyboard_state.keys_pressed {
            match keycode {
                Keycode::L { .. } => {
                    self.looking_at_point_light = !self.looking_at_point_light;
                }
                _ => {}
            }
        }

        if self.looking_at_point_light {
            camera.set_target_position(self.point_lights[0].position);
        } else {
            camera.update(
                keyboard_state,
                mouse_state,
                game_controller_state,
                seconds_since_last_update,
            );

            context.set_projection(camera.get_projection());
        }

        let camera_view_inverse_transform = camera.get_view_inverse_transform();

        context.set_view_inverse_transform(camera_view_inverse_transform);

        context.set_directional_light(DirectionalLight {
            intensities: self.directional_light.intensities,
            direction: (self.directional_light.direction * camera_view_inverse_transform)
                .as_normal(),
        });

        self.pipeline
            .options
            .update(keyboard_state, mouse_state, game_controller_state);

        self.pipeline
            .fragment_shader
            .update(keyboard_state, mouse_state, game_controller_state);

        context.set_camera_position(Vec4::new(camera.get_position(), 1.0));

        let color_channel_phase_shift = 2.0 * PI / 3.0;

        let light_speed_factor = 0.66;
        let light_count = self.point_lights.len();

        for (index, light) in self.point_lights.iter_mut().enumerate() {
            let orbit_radius: f32 = 12.0;
            let max_point_light_intensity: f32 = 1.0;

            let light_phase_shift = (2.0 * PI / (light_count as f32)) * index as f32;

            light.intensities = Vec3 {
                x: (self.seconds_ellapsed + color_channel_phase_shift * 0.0 + light_phase_shift)
                    .sin()
                    / 2.0
                    + 0.5,
                y: (self.seconds_ellapsed + color_channel_phase_shift * 1.0 + light_phase_shift)
                    .sin()
                    / 2.0
                    + 0.5,
                z: (self.seconds_ellapsed + color_channel_phase_shift * 2.0 + light_phase_shift)
                    .sin()
                    / 2.0
                    + 0.5,
            } * max_point_light_intensity;

            let offset = index % 2 == 0;

            light.position = Vec3 {
                x: orbit_radius
                    * ((self.seconds_ellapsed * light_speed_factor) + light_phase_shift).sin()
                    * if offset { 1.5 } else { 1.0 },
                y: 1.0,
                z: orbit_radius
                    * ((self.seconds_ellapsed * light_speed_factor) + light_phase_shift).cos()
                    * if offset { 1.5 } else { 1.0 },
            };

            context.set_point_light(index, *light);
        }

        let camera2 = (self.cameras[1]).borrow_mut();

        camera2.set_target_position(self.point_lights[0].position);

        let max_spot_light_intensity: f32 = 0.6;

        self.spot_lights[0].intensities = Vec3 {
            x: (self.seconds_ellapsed + color_channel_phase_shift * 0.0).cos() / 2.0 + 0.5,
            y: (self.seconds_ellapsed + color_channel_phase_shift * 1.0).cos() / 2.0 + 0.5,
            z: (self.seconds_ellapsed + color_channel_phase_shift * 2.0).cos() / 2.0 + 0.5,
        } * max_spot_light_intensity;

        context.set_spot_light(0, self.spot_lights[0]);

        let mut entities = self.entities.write().unwrap();

        let rotation_speed = 0.3;

        for entity in entities.as_mut_slice() {
            if entity.mesh.object_name == "plane" {
                continue;
            }

            entity.rotation.z += 1.0 * rotation_speed * PI * seconds_since_last_update;
            entity.rotation.z %= 2.0 * PI;

            entity.rotation.x += 1.0 * rotation_speed * PI * seconds_since_last_update;
            entity.rotation.x %= 2.0 * PI;

            entity.rotation.y += 1.0 * rotation_speed * PI * seconds_since_last_update;
            entity.rotation.y %= 2.0 * PI;
        }

        self.prev_mouse_state = mouse_state.clone();
    }

    fn render(&mut self) {
        self.pipeline.begin_frame();

        for entity in self.entities.read().unwrap().as_slice() {
            self.pipeline
                .render_entity(&entity, Some(self.material_cache));
        }

        for (index, camera) in self.cameras.iter().enumerate() {
            if index == self.active_camera_index {
                for light in &self.point_lights {
                    self.pipeline.render_point_light(
                        &light,
                        Some(&camera),
                        Some(&mut self.material_cache),
                    );
                }

                for light in &self.spot_lights {
                    self.pipeline.render_spot_light(
                        &light,
                        Some(&camera),
                        Some(&mut self.material_cache),
                    );
                }

                continue;
            }

            self.pipeline.render_camera(camera);
        }
    }

    fn get_pixel_data(&self) -> &Vec<u32> {
        return self.pipeline.get_pixel_data();
    }
}
