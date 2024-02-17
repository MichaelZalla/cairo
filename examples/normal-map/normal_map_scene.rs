use std::{borrow::BorrowMut, f32::consts::PI, sync::RwLock};

use cairo::{
    app::App,
    buffer::framebuffer::Framebuffer,
    device::{GameControllerState, KeyboardState, MouseState},
    entity::Entity,
    material::cache::MaterialCache,
    pipeline::{options::PipelineOptions, Pipeline},
    scene::{
        camera::Camera,
        light::{AmbientLight, DirectionalLight, PointLight, SpotLight},
        Scene,
    },
    shader::ShaderContext,
    shaders::{
        default_fragment_shader::DEFAULT_FRAGMENT_SHADER,
        // default_geometry_shader::DEFAULT_GEOMETRY_SHADER,
        default_vertex_shader::DEFAULT_VERTEX_SHADER,
    },
    vec::{vec3::Vec3, vec4::Vec4},
};

pub struct NormalMapScene<'a> {
    framebuffer_rwl: &'a RwLock<Framebuffer>,
    pipeline: Pipeline<'a>,
    cameras: Vec<Camera>,
    active_camera_index: usize,
    _ambient_light: AmbientLight,
    point_light: PointLight,
    _spot_light: SpotLight,
    entities: &'a RwLock<Vec<&'a mut Entity<'a>>>,
    materials: &'a mut MaterialCache,
    shader_context: &'a RwLock<ShaderContext>,
    seconds_ellapsed: f32,
}

impl<'a> NormalMapScene<'a> {
    pub fn new(
        framebuffer_rwl: &'a RwLock<Framebuffer>,
        entities: &'a RwLock<Vec<&'a mut Entity<'a>>>,
        materials: &'a mut MaterialCache,
        shader_context: &'a RwLock<ShaderContext>,
    ) -> Self {
        let framebuffer = framebuffer_rwl.read().unwrap();

        let vertex_shader = DEFAULT_VERTEX_SHADER;

        let fragment_shader = DEFAULT_FRAGMENT_SHADER;

        let mut context = shader_context.write().unwrap();

        let aspect_ratio = framebuffer.width_over_height;

        // Set up a camera for rendering our scene
        let mut camera: Camera = Camera::from_perspective(
            Vec3 {
                x: 0.0,
                y: 0.0,
                z: -5.0,
            },
            Default::default(),
            75.0,
            aspect_ratio,
        );

        camera.movement_speed = 5.0;

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
                x: 0.0,
                y: 1.0,
                z: 1.0,
                w: 1.0,
            }
            .as_normal(),
        };

        context.set_directional_light(directional_light);

        let mut point_light = PointLight::new();

        point_light.position.y = 0.0;
        point_light.position.z = -4.0;

        point_light.intensities = Vec3::ones() * 10.0;
        point_light.specular_intensity = 10.0;
        point_light.constant_attenuation = 1.0;
        point_light.linear_attenuation = 0.35;
        point_light.quadratic_attenuation = 0.44;

        let spot_light = SpotLight::new();

        let pipeline_options: PipelineOptions = Default::default();

        let mut pipeline = Pipeline::new(
            shader_context,
            vertex_shader,
            fragment_shader,
            pipeline_options,
        );

        pipeline.geometry_shader_options.normal_mapping_active = true;

        return NormalMapScene {
            framebuffer_rwl,
            pipeline,
            entities,
            materials,
            shader_context,
            cameras: vec![camera],
            active_camera_index: 0,
            _ambient_light: ambient_light,
            point_light,
            _spot_light: spot_light,
            seconds_ellapsed: 0.0,
        };
    }
}

impl<'a> Scene for NormalMapScene<'a> {
    fn update(
        &mut self,
        app: &App,
        keyboard_state: &KeyboardState,
        mouse_state: &MouseState,
        game_controller_state: &GameControllerState,
    ) {
        let mut context = self.shader_context.write().unwrap();

        self.seconds_ellapsed += app.timing_info.seconds_since_last_update;

        let camera = (self.cameras[self.active_camera_index]).borrow_mut();

        camera.update(
            &app.timing_info,
            keyboard_state,
            mouse_state,
            game_controller_state,
        );

        context.set_view_position(Vec4::new(camera.look_vector.get_position(), 1.0));

        context.set_projection(camera.get_projection());

        context.set_view_inverse_transform(camera.get_view_inverse_transform());

        self.pipeline
            .options
            .update(keyboard_state, mouse_state, game_controller_state);

        self.pipeline.geometry_shader_options.update(
            keyboard_state,
            mouse_state,
            game_controller_state,
        );

        context.set_point_light(0, self.point_light);

        let mut entities = self.entities.write().unwrap();

        let rotation_speed = 0.1;

        for entity in entities.as_mut_slice() {
            entity.rotation.z +=
                1.0 * rotation_speed * PI * app.timing_info.seconds_since_last_update;
            entity.rotation.z %= 2.0 * PI;

            entity.rotation.x +=
                1.0 * rotation_speed * PI * app.timing_info.seconds_since_last_update;
            entity.rotation.x %= 2.0 * PI;

            entity.rotation.y +=
                1.0 * rotation_speed * PI * app.timing_info.seconds_since_last_update;
            entity.rotation.y %= 2.0 * PI;
        }
    }

    fn render(&mut self) {
        self.pipeline.bind_framebuffer(Some(&self.framebuffer_rwl));

        let camera = self.cameras[self.active_camera_index];

        {
            let framebuffer = self.framebuffer_rwl.write().unwrap();

            match framebuffer.attachments.depth.as_ref() {
                Some(lock) => {
                    let mut depth_buffer = lock.write().unwrap();

                    depth_buffer.set_projection_z_near(camera.get_projection_z_near());
                    depth_buffer.set_projection_z_far(camera.get_projection_z_far());
                }
                None => (),
            }
        }

        self.pipeline.begin_frame();

        {
            for entity in self.entities.read().unwrap().as_slice() {
                self.pipeline.render_entity(&entity, Some(self.materials));
            }

            self.pipeline.render_point_light(
                &self.point_light,
                Some(&camera),
                Some(&mut self.materials),
            );
        }

        self.pipeline.end_frame()
    }
}
