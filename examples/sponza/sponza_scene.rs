use std::{borrow::BorrowMut, sync::RwLock};

use sdl2::keyboard::Keycode;

use cairo::{
    context::ApplicationRenderingContext,
    device::{GameControllerState, KeyboardState, MouseState},
    effect::Effect,
    effects::default_effect::DefaultEffect,
    entity::Entity,
    graphics::Graphics,
    material::cache::MaterialCache,
    matrix::Mat4,
    pipeline::{options::PipelineOptions, Pipeline},
    scene::{
        camera::Camera,
        light::{AmbientLight, DirectionalLight, PointLight, SpotLight},
        Scene,
    },
    texture::cubemap::CubeMap,
    vec::{vec3::Vec3, vec4::Vec4},
};

static SPONZA_CENTER: Vec3 = Vec3 {
    x: -572.3847 + 500.0,
    y: 233.06613,
    z: -43.05618,
};

pub struct SponzaScene<'a> {
    seconds_ellapsed: f32,
    pipeline: Pipeline<DefaultEffect>,
    pipeline_options: PipelineOptions,
    bilinear_active: bool,
    cameras: Vec<Camera>,
    active_camera_index: usize,
    point_light: PointLight,
    entities: &'a RwLock<Vec<Entity<'a>>>,
    skybox: CubeMap,
    materials: &'a MaterialCache,
    prev_mouse_state: MouseState,
}

impl<'a> SponzaScene<'a> {
    pub fn new(
        graphics: Graphics,
        rendering_context: &ApplicationRenderingContext,
        entities: &'a RwLock<Vec<Entity<'a>>>,
        materials: &'a MaterialCache,
    ) -> Self {
        // Set up a camera for rendering our scene
        let mut camera: Camera = Camera::new(
            graphics.buffer.width_over_height,
            Vec3 {
                x: 0.0,
                y: 0.0,
                z: -5.0,
            },
            Vec3::new(),
        );

        camera.movement_speed = 300.0;

        camera.set_projection_z_far(10000.0);

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

        let mut skybox = CubeMap::from_cross("examples/skybox/assets/grass_sky.jpg");

        skybox.load(rendering_context).unwrap();

        let pipeline_options = PipelineOptions {
            should_render_wireframe: false,
            should_render_shader: true,
            should_render_normals: false,
            should_cull_backfaces: true,
        };

        let world_transform = Mat4::new();

        let view_position = Vec4::new(camera.get_position(), 1.0);

        let view_inverse_transform = camera.get_view_inverse_transform();

        let projection_transform = camera.get_projection();

        let pipeline = Pipeline::new(
            graphics,
            camera.get_projection_z_near(),
            camera.get_projection_z_far(),
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
            bilinear_active: false,
            entities,
            skybox,
            materials,
            cameras: vec![camera],
            active_camera_index: 0,
            point_light,
            prev_mouse_state: MouseState::new(),
        };
    }
}

impl<'a> Scene for SponzaScene<'a> {
    fn update(
        &mut self,
        keyboard_state: &KeyboardState,
        mouse_state: &MouseState,
        game_controller_state: &GameControllerState,
        seconds_since_last_update: f32,
    ) {
        self.seconds_ellapsed += seconds_since_last_update;

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

        self.pipeline.effect.set_projection(camera.get_projection());

        for keycode in &keyboard_state.keys_pressed {
            match keycode {
                Keycode::B { .. } => {
                    self.bilinear_active = !self.bilinear_active;
                    self.pipeline
                        .effect
                        .set_bilinear_active(self.bilinear_active);
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
                Keycode::Num4 { .. } => {
                    self.pipeline_options.should_cull_backfaces =
                        !self.pipeline_options.should_cull_backfaces;

                    self.pipeline.set_options(self.pipeline_options);
                }
                _ => {}
            }
        }

        self.pipeline
            .effect
            .set_point_light_position(self.point_light.position);

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
        self.pipeline.begin_frame();

        let r = self.entities.read().unwrap();

        for entity in r.as_slice() {
            self.pipeline
                .render_mesh(&entity.mesh, Some(self.materials));
        }

        let camera = self.cameras[self.active_camera_index];

        self.pipeline.render_skybox(&self.skybox, &camera);
    }

    fn get_pixel_data(&self) -> &Vec<u32> {
        self.pipeline.get_pixel_data()
    }
}
