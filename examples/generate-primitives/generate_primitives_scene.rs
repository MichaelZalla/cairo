use std::{borrow::BorrowMut, f32::consts::PI, sync::RwLock};

use sdl2::keyboard::Keycode;

use cairo::{
    device::{GameControllerState, KeyboardState, MouseState},
    effects::default_effect::DefaultEffect,
    entity::Entity,
    graphics::{Graphics, PixelBuffer},
    matrix::Mat4,
    pipeline::{Pipeline, PipelineOptions},
    scene::{
        camera::Camera,
        light::{AmbientLight, DirectionalLight, PointLight},
        Scene,
    },
    vec::{
        vec3::Vec3,
        vec4::{self, Vec4},
    },
};

static FIELD_OF_VIEW: f32 = 100.0;

static PROJECTION_Z_NEAR: f32 = 0.3;
static PROJECTION_Z_FAR: f32 = 100.0;

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
    prev_mouse_state: MouseState,
}

impl<'a> GeneratePrimitivesScene<'a> {
    pub fn new(
        canvas_width: u32,
        canvas_height: u32,
        entities: &'a RwLock<Vec<&'a mut Entity<'a>>>,
    ) -> Self {
        let aspect_ratio = canvas_width as f32 / canvas_height as f32;

        let graphics = Graphics {
            buffer: PixelBuffer::new(canvas_width, canvas_height),
        };

        // Set up a camera for rendering our cube scene
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
            50.0,
            0.0,
            6.0,
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
                y: 0.0,
                z: 1.0,
                w: 1.0,
            },
        };

        let point_light = PointLight {
            intensities: Vec3 {
                x: 0.15,
                y: 0.1,
                z: 0.75,
            },
            position: Vec3 {
                x: 4.0,
                y: -1.5,
                z: 4.0,
            },
            constant_attenuation: 0.001,
            linear_attenuation: 0.03,
            quadratic_attenuation: 0.06,
        };

        // @TODO Pipeline to store a reference to PipelineOptions
        let pipeline_options = PipelineOptions {
            should_render_wireframe: false,
            should_render_shader: true,
            should_render_normals: false,
            should_cull_backfaces: true,
        };

        let world_transform = Mat4::scaling(1.0);

        let view_inverse_transform = Mat4::translation(Vec3 {
            x: camera.position_inverse.x,
            y: camera.position_inverse.y,
            z: camera.position_inverse.z,
        });

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
                view_inverse_transform,
                projection_transform,
                ambient_light,
                directional_light,
                point_light,
            ),
            pipeline_options,
        );

        return GeneratePrimitivesScene {
            pipeline,
            pipeline_options,
            entities,
            cameras: vec![camera],
            active_camera_index: 0,
            // ambient_light,
            directional_light,
            point_light,
            canvas_width,
            canvas_height,
            prev_mouse_state: MouseState::new(),
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
        // Calculate mouse position delta

        let mouse_position = mouse_state.position;

        let ndc_mouse_x = mouse_position.0 as f32 / self.canvas_width as f32;
        let ndc_mouse_y = mouse_position.1 as f32 / self.canvas_height as f32;

        let prev_ndc_mouse_x = self.prev_mouse_state.position.0 as f32 / self.canvas_width as f32;
        let prev_ndc_mouse_y = self.prev_mouse_state.position.1 as f32 / self.canvas_height as f32;

        let mouse_x_delta = ndc_mouse_x - prev_ndc_mouse_x;
        let mouse_y_delta = ndc_mouse_y - prev_ndc_mouse_y;

        // Apply camera rotation based on mouse position delta

        let camera = (self.cameras[self.active_camera_index]).borrow_mut();

        camera.rotation_inverse_transform = camera.rotation_inverse_transform
            * Mat4::rotation_y(-mouse_x_delta * 2.0 * PI)
            * Mat4::rotation_x(-mouse_y_delta * 2.0 * PI);

        camera.rotation_inverse_transposed = camera.rotation_inverse_transform.transposed();

        // Apply camera movement based on keyboard or gamepad input

        let camera_movement_step = camera.movement_speed * seconds_since_last_update;

        for keycode in &keyboard_state.keys_pressed {
            match keycode {
                Keycode::Up | Keycode::W { .. } => {
                    camera.position +=
                        vec4::FORWARD * camera_movement_step * camera.rotation_inverse_transposed;
                }
                Keycode::Down | Keycode::S { .. } => {
                    camera.position -=
                        vec4::FORWARD * camera_movement_step * camera.rotation_inverse_transposed;
                }
                Keycode::Left | Keycode::A { .. } => {
                    camera.position +=
                        vec4::LEFT * camera_movement_step * camera.rotation_inverse_transposed;
                }
                Keycode::Right | Keycode::D { .. } => {
                    camera.position -=
                        vec4::LEFT * camera_movement_step * camera.rotation_inverse_transposed;
                }
                Keycode::Q { .. } => {
                    camera.position -=
                        vec4::UP * camera_movement_step * camera.rotation_inverse_transposed;
                }
                Keycode::E { .. } => {
                    camera.position +=
                        vec4::UP * camera_movement_step * camera.rotation_inverse_transposed;
                }
                _ => {}
            }
        }

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

        // let mut entities = self.entities.write().unwrap();

        // let rotation_speed = 0.3;

        // for entity in entities.as_mut_slice() {
        //     // Mesh rotation via our time delta

        //     entity.rotation.z += 1.0 * rotation_speed * PI * seconds_since_last_update;
        //     entity.rotation.z %= 2.0 * PI;

        //     entity.rotation.x += 1.0 * rotation_speed * PI * seconds_since_last_update;
        //     entity.rotation.x %= 2.0 * PI;

        //     entity.rotation.y += 1.0 * rotation_speed * PI * seconds_since_last_update;
        //     entity.rotation.y %= 2.0 * PI;
        // }

        self.prev_mouse_state = mouse_state.clone();
    }

    fn render(&mut self) {
        self.pipeline.clear_pixel_buffer();

        if self.pipeline_options.should_render_shader {
            self.pipeline.clear_z_buffer();
        }

        let r = self.entities.read().unwrap();

        let camera = (self.cameras[self.active_camera_index]).borrow_mut();

        let camera_translation_inverse = camera.position * -1.0;

        let camera_translation_inverse_transform = Mat4::translation(Vec3 {
            x: camera_translation_inverse.x,
            y: camera_translation_inverse.y,
            z: camera_translation_inverse.z,
        });

        let camera_view_inverse_transform =
            camera_translation_inverse_transform * camera.rotation_inverse_transform;

        self.pipeline
            .effect
            .set_view_inverse_transform(camera_view_inverse_transform);

        self.pipeline.effect.set_directional_light_direction(
            (self.directional_light.direction * camera_view_inverse_transform).as_normal(),
        );

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

            self.pipeline.render_mesh(&entity.mesh);
        }
    }

    fn get_pixel_data(&self) -> &Vec<u32> {
        return self.pipeline.get_pixel_data();
    }
}
