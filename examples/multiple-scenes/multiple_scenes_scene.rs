use std::{borrow::BorrowMut, f32::consts::PI, sync::RwLock};

use cairo::{
    device::{GameControllerState, KeyboardState, MouseState},
    entity::Entity,
    graphics::Graphics,
    matrix::Mat4,
    pipeline::{options::PipelineOptions, Pipeline},
    scene::{
        camera::Camera,
        light::{AmbientLight, DirectionalLight, PointLight, SpotLight},
        Scene,
    },
    shader::fragment::FragmentShader,
    shader::vertex::VertexShader,
    shader::ShaderContext,
    shaders::{
        default_fragment_shader::DefaultFragmentShader, default_vertex_shader::DefaultVertexShader,
    },
    vec::{vec3::Vec3, vec4::Vec4},
};

pub struct MultipleScenesScene<'a> {
    pipeline: Pipeline<'a>,
    cameras: Vec<Camera>,
    active_camera_index: usize,
    entities: &'a RwLock<Vec<&'a mut Entity<'a>>>,
    shader_context: &'a RwLock<ShaderContext>,
    prev_mouse_state: MouseState,
}

impl<'a> MultipleScenesScene<'a> {
    pub fn new(
        graphics: Graphics,
        entities: &'a RwLock<Vec<&'a mut Entity<'a>>>,
        shader_context: &'a RwLock<ShaderContext>,
    ) -> Self {
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

        let spot_light = SpotLight::new();

        let pipeline_options: PipelineOptions = Default::default();

        let world_transform = Mat4::scaling(1.0);

        let view_position = Vec4::new(camera.get_position(), 1.0);

        let view_inverse_transform = camera.get_view_inverse_transform();

        let projection_transform = camera.get_projection();

        let mut context = shader_context.write().unwrap();

        context.set_world_transform(world_transform);
        context.set_camera_position(view_position);
        context.set_view_inverse_transform(view_inverse_transform);
        context.set_projection(projection_transform);

        context.set_ambient_light(ambient_light);
        context.set_directional_light(directional_light);
        context.set_point_light(0, point_light);
        context.set_spot_light(0, spot_light);

        let vertex_shader = DefaultVertexShader::new(shader_context);

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

        return MultipleScenesScene {
            pipeline,
            entities,
            shader_context,
            cameras: vec![camera],
            active_camera_index: 0,
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
        let mut context = self.shader_context.write().unwrap();

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
            .fragment_shader
            .update(keyboard_state, mouse_state, game_controller_state);

        context.set_camera_position(Vec4::new(camera.get_position(), 1.0));

        context.set_projection(camera.get_projection());

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

        context.set_view_inverse_transform(camera_view_inverse_transform);

        context.set_world_transform(world_transform);

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
            let world_transform = Mat4::scaling(1.0)
                * Mat4::rotation_x(entity.rotation.x)
                * Mat4::rotation_y(entity.rotation.y)
                * Mat4::rotation_z(entity.rotation.z)
                * Mat4::translation(entity.position);

            {
                let mut context = self.shader_context.write().unwrap();

                context.set_world_transform(world_transform);
            }

            self.pipeline.render_mesh(&entity.mesh, None);
        }
    }

    fn get_pixel_data(&self) -> &Vec<u32> {
        return self.pipeline.get_pixel_data();
    }
}
