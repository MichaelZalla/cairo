use std::{borrow::BorrowMut, f32::consts::PI, sync::RwLock};

use cairo::{
    device::{GameControllerState, KeyboardState, MouseState},
    effect::Effect,
    effects::default_effect::DefaultEffect,
    entity::Entity,
    graphics::Graphics,
    matrix::Mat4,
    pipeline::{options::PipelineOptions, Pipeline},
    scene::{
        camera::Camera,
        light::{AmbientLight, DirectionalLight, PointLight, SpotLight},
        Scene,
    },
    vec::{vec3::Vec3, vec4::Vec4},
};

pub struct MultipleScenesScene<'a> {
    pipeline: Pipeline<DefaultEffect>,
    cameras: Vec<Camera>,
    active_camera_index: usize,
    ambient_light: AmbientLight,
    directional_light: DirectionalLight,
    point_light: PointLight,
    entities: &'a RwLock<Vec<&'a mut Entity<'a>>>,
    prev_mouse_state: MouseState,
}

impl<'a> MultipleScenesScene<'a> {
    pub fn new(graphics: Graphics, entities: &'a RwLock<Vec<&'a mut Entity<'a>>>) -> Self {
        // Set up a camera for rendering our scenes
        let aspect_ratio = graphics.buffer.width_over_height;

        let camera: Camera = Camera::new(
            aspect_ratio,
            Vec3 {
                x: 0.0,
                y: 0.0,
                z: -5.0,
            },
            Default::default(),
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

        let mut point_light = PointLight::new();

        point_light.intensities = Vec3 {
            x: 0.4,
            y: 0.4,
            z: 0.4,
        };

        let spot_light = SpotLight {
            intensities: Vec3 {
                x: 0.4,
                y: 0.4,
                z: 0.4,
            },
            position: Vec3 {
                x: 0.0,
                y: -5.0,
                z: 0.0,
            },
            direction: Vec3 {
                x: 0.0,
                y: 1.0,
                z: 0.0,
            },
            inner_cutoff_angle: (PI / 18.0).cos(),
            outer_cutoff_angle: (PI / 12.0).cos(),
            constant_attenuation: 1.0,
            linear_attenuation: 0.35,
            quadratic_attenuation: 0.44,
        };

        let pipeline_options = PipelineOptions {
            should_render_wireframe: false,
            should_render_shader: true,
            should_render_normals: false,
            should_cull_backfaces: true,
        };

        let world_transform = Mat4::scaling(1.0);

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

        return MultipleScenesScene {
            pipeline,
            entities,
            cameras: vec![camera],
            active_camera_index: 0,
            ambient_light,
            directional_light,
            point_light,
            prev_mouse_state: MouseState::new(),
        };
    }
}

impl<'a> Scene for MultipleScenesScene<'a> {
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
            .options
            .update(keyboard_state, mouse_state, game_controller_state);

        self.pipeline
            .effect
            .set_camera_position(Vec4::new(camera.get_position(), 1.0));

        self.pipeline.effect.set_projection(camera.get_projection());

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

        let camera_view_inverse_transform = camera.get_view_inverse_transform();

        self.pipeline
            .effect
            .set_view_inverse_transform(camera_view_inverse_transform);

        self.pipeline.effect.set_world_transform(world_transform);

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
        self.pipeline.begin_frame();

        let r = self.entities.read().unwrap();

        for entity in r.as_slice() {
            self.pipeline.render_mesh(&entity.mesh, None);
        }
    }

    fn get_pixel_data(&self) -> &Vec<u32> {
        return self.pipeline.get_pixel_data();
    }
}
