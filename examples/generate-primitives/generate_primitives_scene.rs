use std::{borrow::BorrowMut, f32::consts::PI, sync::RwLock};

use sdl2::keyboard::Keycode;

use cairo::{
    device::{GameControllerState, KeyboardState, MouseState},
    effects::default_effect::DefaultEffect,
    entity::Entity,
    graphics::{Graphics, PixelBuffer},
    material::Material,
    matrix::Mat4,
    pipeline::{Pipeline, PipelineOptions},
    scene::{
        camera::Camera,
        light::{AmbientLight, DirectionalLight, PointLight, SpotLight},
        Scene,
    },
    vec::{
        vec3::{self, Vec3},
        vec4::Vec4,
    },
};

static FIELD_OF_VIEW: f32 = 75.0;

static PROJECTION_Z_NEAR: f32 = 0.3;
static PROJECTION_Z_FAR: f32 = 1000.0;

static CAMERA_MOVEMENT_SPEED: f32 = 50.0;

pub struct GeneratePrimitivesScene<'a> {
    pipeline: Pipeline<DefaultEffect>,
    pipeline_options: PipelineOptions,
    canvas_width: u32,
    canvas_height: u32,
    cameras: Vec<Camera>,
    active_camera_index: usize,
    // ambient_light: AmbientLight,
    directional_light: DirectionalLight,
    point_light: PointLight,
    entities: &'a RwLock<Vec<&'a mut Entity<'a>>>,
    materials: &'a Vec<Material>,
    prev_mouse_state: MouseState,
    seconds_ellapsed: f32,
}

impl<'a> GeneratePrimitivesScene<'a> {
    pub fn new(
        canvas_width: u32,
        canvas_height: u32,
        entities: &'a RwLock<Vec<&'a mut Entity<'a>>>,
        materials: &'a Vec<Material>,
    ) -> Self {
        let aspect_ratio = canvas_width as f32 / canvas_height as f32;

        let graphics = Graphics {
            buffer: PixelBuffer::new(canvas_width, canvas_height),
        };

        // Set up a camera for rendering our cube scene
        let camera: Camera = Camera::new(
            Vec3 {
                x: 0.0,
                y: 0.0,
                z: -5.0,
            },
            Default::default(),
        );

        // Define lights for our scene
        let ambient_light = AmbientLight {
            intensities: Vec3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
        };

        let directional_light = DirectionalLight {
            intensities: Vec3 {
                x: 0.1,
                y: 0.1,
                z: 0.1,
            },
            direction: Vec4 {
                x: -1.0,
                y: -1.0,
                z: 1.0,
                w: 1.0,
            }
            .as_normal(),
        };

        let point_light = PointLight::new();

        let spot_light = SpotLight {
            intensities: Vec3 {
                x: 0.7,
                y: 0.7,
                z: 0.7,
            },
            position: Vec3 {
                x: 0.0,
                y: 12.0,
                z: 0.0,
            },
            direction: Vec3 {
                x: 0.0,
                y: -1.0,
                z: 0.0,
            },
            inner_cutoff_angle: (2.5 as f32).to_radians().cos(),
            outer_cutoff_angle: (17.5 as f32).to_radians().cos(),
            constant_attenuation: 0.6,
            linear_attenuation: 0.35,
            quadratic_attenuation: 0.44,
        };

        // @TODO Pipeline to store a reference to PipelineOptions
        let pipeline_options = PipelineOptions {
            should_render_wireframe: false,
            should_render_shader: true,
            should_render_normals: false,
            should_cull_backfaces: true,
        };

        let world_transform = Mat4::scaling(1.0);

        let view_position = Vec4::new(camera.get_position(), 1.0);

        let view_inverse_transform = camera.get_view_inverse_transform();

        let projection_transform = Mat4::projection_for_fov(
            FIELD_OF_VIEW,
            aspect_ratio,
            PROJECTION_Z_NEAR,
            PROJECTION_Z_FAR,
        );

        let pipeline = Pipeline::new(
            graphics,
            DefaultEffect::new(
                world_transform,
                view_position,
                view_inverse_transform,
                projection_transform,
                ambient_light,
                directional_light,
                point_light,
                spot_light,
            ),
            pipeline_options,
        );

        return GeneratePrimitivesScene {
            pipeline,
            pipeline_options,
            entities,
            materials,
            cameras: vec![camera],
            active_camera_index: 0,
            // ambient_light,
            directional_light,
            point_light,
            canvas_width,
            canvas_height,
            prev_mouse_state: MouseState::new(),
            seconds_ellapsed: 0.0,
        };
    }
}

impl<'a> Scene for GeneratePrimitivesScene<'a> {
    fn update(
        &mut self,
        keyboard_state: &KeyboardState,
        mouse_state: &MouseState,
        _game_controller_state: &GameControllerState,
        seconds_since_last_update: f32,
    ) {
        self.seconds_ellapsed += seconds_since_last_update;

        // Calculate mouse position delta

        let mouse_position = mouse_state.position;

        let ndc_mouse_x = mouse_position.0 as f32 / self.canvas_width as f32;
        let ndc_mouse_y = mouse_position.1 as f32 / self.canvas_height as f32;

        let prev_ndc_mouse_x = self.prev_mouse_state.position.0 as f32 / self.canvas_width as f32;
        let prev_ndc_mouse_y = self.prev_mouse_state.position.1 as f32 / self.canvas_height as f32;

        let mouse_x_delta = ndc_mouse_x - prev_ndc_mouse_x;
        let mouse_y_delta = ndc_mouse_y - prev_ndc_mouse_y;

        // Update camera pitch and yaw, based on mouse position deltas.

        let camera = (self.cameras[self.active_camera_index]).borrow_mut();

        let pitch = camera.get_pitch();
        let yaw = camera.get_yaw();

        camera.set_pitch(pitch - mouse_y_delta * 2.0 * PI);
        camera.set_yaw(yaw + mouse_x_delta * 2.0 * PI);

        // Apply camera movement based on keyboard or gamepad input

        let camera_movement_step = CAMERA_MOVEMENT_SPEED * seconds_since_last_update;

        for keycode in &keyboard_state.keys_pressed {
            let position = camera.get_position();

            match keycode {
                Keycode::Up | Keycode::W { .. } => {
                    camera.set_position(position + camera.get_forward() * camera_movement_step);
                }
                Keycode::Down | Keycode::S { .. } => {
                    camera.set_position(position - camera.get_forward() * camera_movement_step);
                }
                Keycode::Left | Keycode::A { .. } => {
                    camera.set_position(position - camera.get_right() * camera_movement_step);
                }
                Keycode::Right | Keycode::D { .. } => {
                    camera.set_position(position + camera.get_right() * camera_movement_step);
                }
                Keycode::Q { .. } => {
                    camera.set_position(position - vec3::UP * camera_movement_step);
                }
                Keycode::E { .. } => {
                    camera.set_position(position + vec3::UP * camera_movement_step);
                }
                _ => {}
            }
        }

        self.pipeline
            .effect
            .set_camera_position(Vec4::new(camera.get_position(), 1.0));

        for keycode in &keyboard_state.keys_pressed {
            match keycode {
                Keycode::Num1 { .. } => {
                    self.pipeline_options.should_render_wireframe =
                        !self.pipeline_options.should_render_wireframe;

                    self.pipeline.set_options(self.pipeline_options);
                }
                Keycode::Num2 { .. } => {
                    self.pipeline_options.should_render_shader =
                        !self.pipeline_options.should_render_shader;

                    self.pipeline.set_options(self.pipeline_options);
                }
                Keycode::Num3 { .. } => {
                    self.pipeline_options.should_render_normals =
                        !self.pipeline_options.should_render_normals;

                    self.pipeline.set_options(self.pipeline_options);
                }
                Keycode::Num4 { .. } => {
                    self.pipeline_options.should_cull_backfaces =
                        !self.pipeline_options.should_cull_backfaces;

                    self.pipeline.set_options(self.pipeline_options);
                }
                _ => {}
            }
        }

        let phase_shift = 2.0 * PI / 3.0;
        let max_intensity: f32 = 0.6;

        self.point_light.intensities = Vec3 {
            x: (self.seconds_ellapsed + phase_shift * 0.0).sin() / 2.0 + 0.5,
            y: (self.seconds_ellapsed + phase_shift * 1.0).sin() / 2.0 + 0.5,
            z: (self.seconds_ellapsed + phase_shift * 2.0).sin() / 2.0 + 0.5,
        } * max_intensity;

        self.point_light.position = Vec3 {
            x: 7.0 * self.seconds_ellapsed.sin(),
            y: 1.0,
            z: 7.0 * self.seconds_ellapsed.cos(),
        };

        let mut entities = self.entities.write().unwrap();

        let rotation_speed = 0.3;

        for entity in entities.as_mut_slice() {
            if entity.mesh.object_name == "point_light" {
                entity.position = self.point_light.position;
            }

            if entity.mesh.object_name == "plane" || entity.mesh.object_name == "point_light" {
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
        self.pipeline.clear_pixel_buffer();

        if self.pipeline_options.should_render_shader {
            self.pipeline.clear_z_buffer();
        }

        let r = self.entities.read().unwrap();

        let camera = (self.cameras[self.active_camera_index]).borrow_mut();

        let camera_view_inverse_transform = camera.get_view_inverse_transform();

        self.pipeline
            .effect
            .set_view_inverse_transform(camera_view_inverse_transform);

        self.pipeline.effect.set_directional_light_direction(
            (self.directional_light.direction * camera_view_inverse_transform).as_normal(),
        );

        self.pipeline
            .effect
            .set_point_light_intensities(self.point_light.intensities);

        self.pipeline
            .effect
            .set_point_light_position(self.point_light.position);

        for entity in r.as_slice() {
            let world_transform = Mat4::scaling(1.0)
                * Mat4::rotation_x(entity.rotation.x)
                * Mat4::rotation_y(entity.rotation.y)
                * Mat4::rotation_z(entity.rotation.z)
                * Mat4::translation(entity.position);

            self.pipeline.effect.set_world_transform(world_transform);

            self.pipeline
                .render_mesh(&entity.mesh, Some(self.materials));
        }
    }

    fn get_pixel_data(&self) -> &Vec<u32> {
        return self.pipeline.get_pixel_data();
    }
}
