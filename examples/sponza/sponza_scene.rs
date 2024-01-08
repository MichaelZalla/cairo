use std::{borrow::BorrowMut, f32::consts::PI, sync::RwLock};

use sdl2::keyboard::Keycode;

use cairo::{
    device::{GameControllerState, KeyboardState, MouseState},
    effect::Effect,
    effects::default_effect::DefaultEffect,
    entity::Entity,
    graphics::Graphics,
    material::cache::MaterialCache,
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

static PROJECTION_Z_NEAR: f32 = 0.3;
static PROJECTION_Z_FAR: f32 = 10000.0;

static CAMERA_MOVEMENT_SPEED: f32 = 300.0;

static SPONZA_CENTER: Vec3 = Vec3 {
    x: -572.3847 + 500.0,
    y: 233.06613,
    z: -43.05618,
};

pub struct SponzaScene<'a> {
    seconds_ellapsed: f32,
    pipeline: Pipeline<DefaultEffect>,
    pipeline_options: PipelineOptions,
    canvas_width: u32,
    canvas_height: u32,
    aspect_ratio: f32,
    field_of_view: f32,
    cameras: Vec<Camera>,
    active_camera_index: usize,
    point_light: PointLight,
    entities: &'a RwLock<Vec<Entity<'a>>>,
    materials: &'a MaterialCache,
    prev_mouse_state: MouseState,
}

impl<'a> SponzaScene<'a> {
    pub fn new(
        graphics: Graphics,
        entities: &'a RwLock<Vec<Entity<'a>>>,
        materials: &'a MaterialCache,
    ) -> Self {
        // Set up a camera for rendering our cube scene
        let camera: Camera = Camera::new(
            Vec3 {
                x: 0.0,
                y: 0.0,
                z: -5.0,
            },
            Vec3::new(),
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
                x: 0.2,
                y: 0.2,
                z: 0.2,
            },
            direction: Vec4 {
                x: 0.0,
                y: -1.0,
                z: 1.00,
                w: 1.0,
            }
            .as_normal(),
        };

        let mut point_light = PointLight::new();

        // point_light.position = atrium_aabb.center;

        point_light.intensities = Vec3 {
            x: 1.0,
            y: 1.0,
            z: 1.0,
        };

        point_light.specular_intensity = 1.0;

        point_light.constant_attenuation = 1.0;
        point_light.linear_attenuation = 0.0014;
        point_light.quadratic_attenuation = 0.000007;

        let mut spot_light = SpotLight::new();

        spot_light.position = SPONZA_CENTER
            + Vec3 {
                x: 0.0,
                y: 300.0,
                z: 0.0,
            };

        spot_light.constant_attenuation = 1.0;
        spot_light.linear_attenuation = 0.0014;
        spot_light.quadratic_attenuation = 0.000007;

        let pipeline_options = PipelineOptions {
            should_render_wireframe: false,
            should_render_shader: true,
            should_render_normals: false,
            should_cull_backfaces: true,
        };

        let graphics_buffer = &graphics.buffer;

        let screen_width = graphics_buffer.width;
        let screen_height = graphics_buffer.height;

        let world_transform = Mat4::new();

        let view_position = Vec4::new(camera.get_position(), 1.0);

        let view_inverse_transform = camera.get_view_inverse_transform();

        let aspect_ratio = graphics.buffer.width_over_height;

        let field_of_view: f32 = 75.0;

        let projection_transform = Mat4::projection_for_fov(
            field_of_view,
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

        return SponzaScene {
            seconds_ellapsed: 0.0,
            pipeline,
            pipeline_options,
            entities,
            materials,
            aspect_ratio,
            field_of_view,
            cameras: vec![camera],
            active_camera_index: 0,
            point_light,
            canvas_width: screen_width,
            canvas_height: screen_height,
            prev_mouse_state: MouseState::new(),
        };
    }
}

impl<'a> Scene for SponzaScene<'a> {
    fn update(
        &mut self,
        keyboard_state: &KeyboardState,
        mouse_state: &MouseState,
        _game_controller_state: &GameControllerState,
        seconds_since_last_update: f32,
    ) {
        self.seconds_ellapsed += seconds_since_last_update;
        // Translate relative mouse movements to NDC values (in the range [0, 1]).

        let mouse_x_delta = mouse_state.relative_motion.0 as f32 / self.canvas_width as f32;
        let mouse_y_delta = mouse_state.relative_motion.1 as f32 / self.canvas_height as f32;

        // Update camera pitch and yaw, based on mouse position deltas.

        let camera = (self.cameras[self.active_camera_index]).borrow_mut();

        let pitch = camera.get_pitch();
        let yaw = camera.get_yaw();

        camera.set_pitch(pitch - mouse_y_delta * 2.0 * PI);
        camera.set_yaw(yaw - mouse_x_delta * 2.0 * PI);

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
                Keycode::L { .. } => {
                    camera.set_target_position(self.point_light.position);
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

        if mouse_state.wheel_did_move {
            self.field_of_view -= mouse_state.wheel_y as f32;

            self.field_of_view = self.field_of_view.max(1.0).min(120.0);

            let projection_transform = Mat4::projection_for_fov(
                self.field_of_view,
                self.aspect_ratio,
                PROJECTION_Z_NEAR,
                PROJECTION_Z_FAR,
            );

            self.pipeline.effect.set_projection(projection_transform);
        }

        let orbit_radius: f32 = 250.0;

        self.point_light.position = SPONZA_CENTER
            + Vec3 {
                x: (self.seconds_ellapsed).sin() * orbit_radius,
                y: 0.0,
                z: (self.seconds_ellapsed).cos() * orbit_radius,
            };

        self.pipeline
            .effect
            .set_point_light_position(self.point_light.position);

        camera.set_target_position(self.point_light.position);

        let mut entities = self.entities.write().unwrap();

        let entity = &mut entities[0];

        let world_transform = Mat4::scaling(1.0)
            * Mat4::rotation_x(entity.rotation.x)
            * Mat4::rotation_y(entity.rotation.y)
            * Mat4::rotation_z(entity.rotation.z)
            * Mat4::translation(entity.position);

        self.pipeline.effect.set_world_transform(world_transform);

        let camera_view_inverse_transform = camera.get_view_inverse_transform();

        self.pipeline
            .effect
            .set_view_inverse_transform(camera_view_inverse_transform);

        self.prev_mouse_state = mouse_state.clone();
    }

    fn render(&mut self) {
        self.pipeline.clear_pixel_buffer();

        if self.pipeline_options.should_render_shader {
            self.pipeline.clear_z_buffer();
        }

        let r = self.entities.read().unwrap();

        for entity in r.as_slice() {
            self.pipeline
                .render_mesh(&entity.mesh, Some(self.materials));
        }
    }

    fn get_pixel_data(&self) -> &Vec<u32> {
        self.pipeline.get_pixel_data()
    }
}
