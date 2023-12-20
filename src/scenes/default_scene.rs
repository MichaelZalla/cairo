use std::{borrow::BorrowMut, f32::consts::PI, sync::RwLock};

use sdl2::keyboard::Keycode;

use crate::{
    context::ApplicationRenderingContext,
    device::{GameControllerState, KeyboardState, MouseState},
    effects::default_effect::DefaultEffect,
    entity::Entity,
    graphics::Graphics,
    matrix::Mat4,
    pipeline::{Pipeline, PipelineOptions},
    scene::{
        camera::Camera,
        light::{AmbientLight, DirectionalLight, PointLight},
        Scene,
    },
    vec::{vec2::Vec2, vec3::Vec3, vec4::Vec4},
};

static FIELD_OF_VIEW: f32 = 100.0;
static PROJECTION_Z_NEAR: f32 = 0.3;
static PROJECTION_Z_FAR: f32 = 10.0;

pub struct DefaultScene<'a> {
    pipeline: Pipeline<DefaultEffect>,
    pipeline_options: PipelineOptions,
    screen_width: u32,
    screen_height: u32,
    cameras: Vec<Camera>,
    active_camera_index: usize,
    ambient_light: AmbientLight,
    directional_light: DirectionalLight,
    point_light: PointLight,
    entities: &'a RwLock<Vec<&'a mut Entity<'a>>>,

    prev_mouse_state: MouseState,
}

impl<'a> DefaultScene<'a> {
    pub fn new(
        graphics: Graphics,
        rendering_context: Option<&ApplicationRenderingContext>,
        entities: &'a RwLock<Vec<&'a mut Entity<'a>>>,
    ) -> Self {
        // Set up a camera for rendering our scenes
        let camera: Camera = Camera::new(
            Vec4::new(
                Vec3 {
                    x: 0.0,
                    y: 0.0,
                    z: -5.0,
                },
                1.0,
            ),
            Mat4::identity(),
            150.0,
            0.0,
            6.0,
        );

        // Define (shared) lights for our scenes
        let ambient_light = AmbientLight {
            intensities: Vec3 {
                x: 0.1,
                y: 0.1,
                z: 0.1,
            },
        };

        let directional_light = DirectionalLight {
            intensities: Vec3 {
                x: 0.3,
                y: 0.3,
                z: 0.3,
            },
            direction: Vec4 {
                x: 0.25,
                y: -1.0,
                z: -0.25,
                w: 1.0,
            },
        };

        let point_light = PointLight {
            intensities: Vec3 {
                x: 0.4,
                y: 0.4,
                z: 0.4,
            },
            position: Default::default(),
            constant_attenuation: 0.382,
            linear_attenuation: 1.0,
            quadratic_attenuation: 2.619,
        };

        let pipeline_options = crate::pipeline::PipelineOptions {
            should_render_wireframe: false,
            should_render_shader: true,
            should_render_normals: false,
            should_cull_backfaces: true,
        };

        let buffer = &graphics.buffer;

        let screen_width = buffer.width;
        let screen_height = buffer.height;

        let world_transform = Mat4::scaling(1.0);

        let view_transform = Mat4::translation(Vec3 {
            x: camera.position_inverse.x,
            y: camera.position_inverse.y,
            z: camera.position_inverse.z,
        });

        let world_view_transform = world_transform * view_transform;

        let aspect_ratio = graphics.buffer.width_over_height;

        let projection_transform = Mat4::projection_for_fov(
            FIELD_OF_VIEW,
            aspect_ratio,
            PROJECTION_Z_NEAR,
            PROJECTION_Z_FAR,
        );

        let pipeline = Pipeline::new(
            graphics,
            DefaultEffect::new(
                world_view_transform,
                projection_transform,
                ambient_light,
                directional_light,
                point_light,
                rendering_context,
            ),
            pipeline_options,
        );

        return DefaultScene {
            pipeline,
            pipeline_options,
            entities,
            cameras: vec![camera],
            active_camera_index: 0,
            ambient_light,
            directional_light,
            point_light,
            screen_width,
            screen_height,
            prev_mouse_state: MouseState::new(),
        };
    }
}

impl<'a> Scene for DefaultScene<'a> {
    fn update(
        &mut self,
        keyboard_state: &KeyboardState,
        mouse_state: &MouseState,
        game_controller_state: &GameControllerState,
        seconds_since_last_update: f32,
    ) {
        // Calculate mouse position delta

        let mouse_position = mouse_state.position;

        let ndc_mouse_x = mouse_position.0 as f32 / self.screen_width as f32;
        let ndc_mouse_y = mouse_position.1 as f32 / self.screen_height as f32;

        let prev_ndc_mouse_x = self.prev_mouse_state.position.0 as f32 / self.screen_width as f32;
        let prev_ndc_mouse_y = self.prev_mouse_state.position.1 as f32 / self.screen_height as f32;

        let mouse_x_delta = ndc_mouse_x - prev_ndc_mouse_x;
        let mouse_y_delta = ndc_mouse_y - prev_ndc_mouse_y;

        // Apply camera rotation based on mouse position delta

        let camera = self.cameras[self.active_camera_index].borrow_mut();

        camera.rotation_inverse_transform = camera.rotation_inverse_transform
            * Mat4::rotation_y(-mouse_x_delta * 2.0 * PI)
            * Mat4::rotation_x(-mouse_y_delta * 2.0 * PI);

        let camera_movement_step = camera.movement_speed * seconds_since_last_update;
        let camera_roll_step = camera.roll_speed * seconds_since_last_update;

        camera.rotation_inverse_transposed = camera.rotation_inverse_transform.transposed();

        // Apply camera movement based on keyboard or gamepad input

        let up = Vec4::new(
            Vec3 {
                x: 0.0,
                y: -1.0,
                z: 0.0,
            },
            1.0,
        );
        let left = Vec4::new(
            Vec3 {
                x: -1.0,
                y: 0.0,
                z: 0.0,
            },
            1.0,
        );
        let forward = Vec4::new(
            Vec3 {
                x: 0.0,
                y: 0.0,
                z: 1.0,
            },
            1.0,
        );

        for keycode in &keyboard_state.keys_pressed {
            match keycode {
                Keycode::Up | Keycode::W { .. } => {
                    camera.position +=
                        forward * camera_movement_step * camera.rotation_inverse_transposed;
                }
                Keycode::Down | Keycode::S { .. } => {
                    camera.position -=
                        forward * camera_movement_step * camera.rotation_inverse_transposed;
                }
                Keycode::Left | Keycode::A { .. } => {
                    camera.position +=
                        left * camera_movement_step * camera.rotation_inverse_transposed;
                }
                Keycode::Right | Keycode::D { .. } => {
                    camera.position -=
                        left * camera_movement_step * camera.rotation_inverse_transposed;
                }
                Keycode::Q { .. } => {
                    camera.position -=
                        up * camera_movement_step * camera.rotation_inverse_transposed;
                }
                Keycode::E { .. } => {
                    camera.position +=
                        up * camera_movement_step * camera.rotation_inverse_transposed;
                }
                Keycode::Z { .. } => {
                    camera.rotation_inverse_transform =
                        camera.rotation_inverse_transform * Mat4::rotation_z(-camera_roll_step);
                }
                Keycode::C { .. } => {
                    camera.rotation_inverse_transform =
                        camera.rotation_inverse_transform * Mat4::rotation_z(camera_roll_step);
                }
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
                _ => {}
            }
        }

        if game_controller_state.buttons.x {
            self.pipeline_options.should_render_wireframe =
                !self.pipeline_options.should_render_wireframe;

            self.pipeline.set_options(self.pipeline_options);
        } else if game_controller_state.buttons.y {
            self.pipeline_options.should_render_normals =
                !self.pipeline_options.should_render_normals;

            self.pipeline.set_options(self.pipeline_options);
        }

        if game_controller_state.buttons.dpad_up {
            camera.position += forward * camera_movement_step * camera.rotation_inverse_transposed;
        } else if game_controller_state.buttons.dpad_down {
            camera.position -= forward * camera_movement_step * camera.rotation_inverse_transposed;
        } else if game_controller_state.buttons.dpad_left {
            camera.rotation_inverse_transform =
                camera.rotation_inverse_transform * Mat4::rotation_z(-camera_roll_step);
        } else if game_controller_state.buttons.dpad_right {
            camera.rotation_inverse_transform =
                camera.rotation_inverse_transform * Mat4::rotation_z(camera_roll_step);
        }

        let left_joystick_position_normalized = Vec2 {
            x: game_controller_state.joysticks.left.position.x as f32 / std::i16::MAX as f32,
            y: game_controller_state.joysticks.left.position.y as f32 / std::i16::MAX as f32,
            z: 1.0,
        };

        if left_joystick_position_normalized.x > 0.5 {
            camera.position -= left * camera_movement_step * camera.rotation_inverse_transposed;
        } else if left_joystick_position_normalized.x < -0.5 {
            camera.position += left * camera_movement_step * camera.rotation_inverse_transposed;
        }

        if left_joystick_position_normalized.y > 0.5 {
            camera.position -= forward * camera_movement_step * camera.rotation_inverse_transposed;
        } else if left_joystick_position_normalized.y < -0.5 {
            camera.position += forward * camera_movement_step * camera.rotation_inverse_transposed;
        }

        let right_joystick_position_normalized = Vec2 {
            x: game_controller_state.joysticks.right.position.x as f32 / std::i16::MAX as f32,
            y: game_controller_state.joysticks.right.position.y as f32 / std::i16::MAX as f32,
            z: 1.0,
        };

        let yaw_delta = -right_joystick_position_normalized.x * PI / 32.0;
        let pitch_delta = -right_joystick_position_normalized.y * PI / 32.0;
        let roll_delta = -yaw_delta * 0.5;

        camera.roll += roll_delta;
        camera.roll = camera.roll % (2.0 * PI);

        camera.rotation_inverse_transform = camera.rotation_inverse_transform
            * Mat4::rotation_y(yaw_delta)
            * Mat4::rotation_x(pitch_delta)
            * Mat4::rotation_z(-yaw_delta * 0.5);

        let mut w = self.entities.write().unwrap();

        let entity = &mut w[0];

        // Mesh rotation via time delta

        entity.rotation.z += 0.2 * PI * seconds_since_last_update;
        entity.rotation.z %= 2.0 * PI;

        entity.rotation.x += 0.2 * PI * seconds_since_last_update;
        entity.rotation.x %= 2.0 * PI;

        entity.rotation.y += 0.2 * PI * seconds_since_last_update;
        entity.rotation.y %= 2.0 * PI;

        let world_transform = Mat4::scaling(0.5)
            * Mat4::rotation_x(entity.rotation.x)
            * Mat4::rotation_y(entity.rotation.y)
            * Mat4::rotation_z(entity.rotation.z)
            * Mat4::translation(entity.position);

        let camera_translation_inverse = camera.position * -1.0;

        let camera_translation_inverse_transform = Mat4::translation(Vec3 {
            x: camera_translation_inverse.x,
            y: camera_translation_inverse.y,
            z: camera_translation_inverse.z,
        });

        let view_transform =
            camera_translation_inverse_transform * camera.rotation_inverse_transform;

        let world_view_transform = world_transform * view_transform;

        self.pipeline
            .effect
            .set_world_view_transform(world_view_transform);

        // // Diffuse light direction rotation via mouse input

        // let mut rotated_diffuse_light_direction = Vec3{
        // 	x: 0.0,
        // 	y: 0.0,
        // 	z: 1.0,
        // };

        // rotated_diffuse_light_direction.rotate_along_x(-2.0 * PI * ndc_mouse_y * -1.0);
        // rotated_diffuse_light_direction.rotate_along_y(-2.0 * PI * ndc_mouse_x);

        // self.pipeline.effect.set_diffuse_light_direction(
        // 	rotated_diffuse_light_direction
        // );

        self.prev_mouse_state = mouse_state.clone();
    }

    fn render(&mut self) {
        self.pipeline.clear_pixel_buffer();

        if self.pipeline_options.should_render_shader {
            self.pipeline.clear_z_buffer();
        }

        let r = self.entities.read().unwrap();

        for entity in r.as_slice() {
            self.pipeline.render_mesh(&entity.mesh);
        }
    }

    fn get_pixel_data(&self) -> &Vec<u32> {
        return self.pipeline.get_pixel_data();
    }
}
