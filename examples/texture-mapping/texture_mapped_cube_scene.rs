use std::{borrow::BorrowMut, f32::consts::PI, sync::RwLock};

use sdl2::keyboard::Keycode;

use cairo::{
    context::ApplicationRenderingContext,
    device::{GameControllerState, KeyboardState, MouseState},
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

static FIELD_OF_VIEW: f32 = 100.0;

static PROJECTION_Z_NEAR: f32 = 0.15;
static PROJECTION_Z_FAR: f32 = 100.0;

pub struct TextureMappedCubeScene<'a> {
    pipeline: Pipeline<DefaultEffect>,
    pipeline_options: PipelineOptions,
    cameras: Vec<Camera>,
    active_camera_index: usize,
    // ambient_light: AmbientLight,
    // directional_light: DirectionalLight,
    point_light: PointLight,
    entities: &'a RwLock<Vec<&'a mut Entity<'a>>>,
    materials: &'a MaterialCache,
    prev_mouse_state: MouseState,
}

impl<'a> TextureMappedCubeScene<'a> {
    pub fn new(
        rendering_context: &ApplicationRenderingContext,
        entities: &'a RwLock<Vec<&'a mut Entity<'a>>>,
        materials: &'a MaterialCache,
    ) -> Self {
        let canvas_output_size = rendering_context
            .canvas
            .read()
            .unwrap()
            .output_size()
            .unwrap();

        let canvas_width = canvas_output_size.0;
        let canvas_height = canvas_output_size.1;

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
                x: 0.4,
                y: 0.4,
                z: 0.4,
            },
        };

        let directional_light = DirectionalLight {
            intensities: Vec3 {
                x: 0.3,
                y: 0.3,
                z: 0.3,
            },
            direction: Vec4 {
                x: 0.0,
                y: -1.0,
                z: 0.0,
                w: 1.0,
            },
        };

        let mut point_light = PointLight::new();

        point_light.intensities = Vec3 {
            x: 0.7,
            y: 0.7,
            z: 0.7,
        };

        point_light.position = Vec3 {
            x: 0.0,
            y: 4.0,
            z: 0.0,
        };

        let spot_light = SpotLight {
            intensities: Vec3 {
                x: 0.4,
                y: 0.4,
                z: 0.4,
            },
            position: Vec3 {
                x: 0.0,
                y: 5.0,
                z: 0.0,
            },
            direction: Vec3 {
                x: 0.0,
                y: -1.0,
                z: 0.0,
            },
            inner_cutoff_angle: (PI / 18.0).cos(),
            outer_cutoff_angle: (PI / 12.0).cos(),
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

        return TextureMappedCubeScene {
            pipeline,
            pipeline_options,
            entities,
            materials,
            cameras: vec![camera],
            active_camera_index: 0,
            // ambient_light,
            // directional_light,
            point_light,
            prev_mouse_state: MouseState::new(),
        };
    }
}

impl<'a> Scene for TextureMappedCubeScene<'a> {
    fn update(
        &mut self,
        keyboard_state: &KeyboardState,
        mouse_state: &MouseState,
        game_controller_state: &GameControllerState,
        seconds_since_last_update: f32,
    ) {
        let camera = (self.cameras[self.active_camera_index]).borrow_mut();

        camera.update(
            keyboard_state,
            mouse_state,
            game_controller_state,
            seconds_since_last_update,
        );

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

        let mut entities = self.entities.write().unwrap();

        let rotation_speed = 0.1 * seconds_since_last_update;

        for entity in entities.as_mut_slice() {
            // Mesh rotation via our time delta

            entity.rotation.z += 1.0 * rotation_speed * PI;
            entity.rotation.z %= 2.0 * PI;

            entity.rotation.x += 1.0 * rotation_speed * PI;
            entity.rotation.x %= 2.0 * PI;

            entity.rotation.y += 1.0 * rotation_speed * PI;
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
