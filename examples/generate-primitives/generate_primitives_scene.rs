use std::{borrow::BorrowMut, f32::consts::PI, sync::RwLock};

use sdl2::keyboard::Keycode;

use cairo::{
    device::{GameControllerState, KeyboardState, MouseState},
    effect::Effect,
    effects::default_effect::DefaultEffect,
    entity::Entity,
    graphics::{Graphics, PixelBuffer},
    material::cache::MaterialCache,
    matrix::Mat4,
    pipeline::{Pipeline, PipelineOptions},
    scene::{
        camera::Camera,
        light::{AmbientLight, DirectionalLight, PointLight, SpotLight},
        Scene,
    },
    vec::{vec3::Vec3, vec4::Vec4},
};

static PROJECTION_Z_NEAR: f32 = 0.3;
static PROJECTION_Z_FAR: f32 = 1000.0;

pub struct GeneratePrimitivesScene<'a> {
    pipeline: Pipeline<DefaultEffect>,
    pipeline_options: PipelineOptions,
    aspect_ratio: f32,
    field_of_view: f32,
    cameras: Vec<Camera>,
    active_camera_index: usize,
    // ambient_light: AmbientLight,
    directional_light: DirectionalLight,
    point_light: PointLight,
    spot_light: SpotLight,
    entities: &'a RwLock<Vec<&'a mut Entity<'a>>>,
    materials: &'a MaterialCache,
    prev_mouse_state: MouseState,
    looking_at_point_light: bool,
    seconds_ellapsed: f32,
}

impl<'a> GeneratePrimitivesScene<'a> {
    pub fn new(
        canvas_width: u32,
        canvas_height: u32,
        entities: &'a RwLock<Vec<&'a mut Entity<'a>>>,
        materials: &'a MaterialCache,
    ) -> Self {
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
                x: 0.05,
                y: 0.05,
                z: 0.05,
            },
        };

        let directional_light = DirectionalLight {
            intensities: Vec3 {
                x: 0.05,
                y: 0.05,
                z: 0.05,
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
            inner_cutoff_angle: (25.0 as f32).to_radians().cos(),
            outer_cutoff_angle: (40.0 as f32).to_radians().cos(),
            constant_attenuation: 1.0,
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

        return GeneratePrimitivesScene {
            pipeline,
            pipeline_options,
            entities,
            materials,
            aspect_ratio,
            field_of_view,
            cameras: vec![camera],
            active_camera_index: 0,
            // ambient_light,
            directional_light,
            point_light,
            spot_light,
            prev_mouse_state: MouseState::new(),
            looking_at_point_light: false,
            seconds_ellapsed: 0.0,
        };
    }

    fn regenerate_projection(&mut self) {
        let projection_transform = Mat4::projection_for_fov(
            self.field_of_view,
            self.aspect_ratio,
            PROJECTION_Z_NEAR,
            PROJECTION_Z_FAR,
        );

        self.pipeline.effect.set_projection(projection_transform);
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
            camera.set_target_position(self.point_light.position);
        } else {
            camera.update(
                keyboard_state,
                mouse_state,
                game_controller_state,
                seconds_since_last_update,
            );
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

        let phase_shift = 2.0 * PI / 3.0;
        let orbit_radius: f32 = 10.0;

        let max_point_light_intensity: f32 = 1.0;

        self.point_light.intensities = Vec3 {
            x: (self.seconds_ellapsed + phase_shift * 0.0).sin() / 2.0 + 0.5,
            y: (self.seconds_ellapsed + phase_shift * 1.0).sin() / 2.0 + 0.5,
            z: (self.seconds_ellapsed + phase_shift * 2.0).sin() / 2.0 + 0.5,
        } * max_point_light_intensity;

        self.point_light.position = Vec3 {
            x: orbit_radius * (self.seconds_ellapsed * 0.66).sin(),
            y: 1.0,
            z: orbit_radius * (self.seconds_ellapsed * 0.66).cos(),
        };

        let max_spot_light_intensity: f32 = 0.6;

        self.spot_light.intensities = Vec3 {
            x: (self.seconds_ellapsed + phase_shift * 0.0).cos() / 2.0 + 0.5,
            y: (self.seconds_ellapsed + phase_shift * 1.0).cos() / 2.0 + 0.5,
            z: (self.seconds_ellapsed + phase_shift * 2.0).cos() / 2.0 + 0.5,
        } * max_spot_light_intensity;

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

        self.pipeline
            .effect
            .set_spot_light_intensities(self.spot_light.intensities);

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
