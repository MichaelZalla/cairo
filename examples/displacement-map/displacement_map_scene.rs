use std::{borrow::BorrowMut, sync::RwLock};

use cairo::{
    app::App,
    buffer::Buffer2D,
    device::{GameControllerState, KeyboardState, MouseState},
    entity::Entity,
    material::cache::MaterialCache,
    pipeline::{
        options::{PipelineFaceCullingReject, PipelineOptions},
        Pipeline,
    },
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

pub struct DisplacementMapScene<'a> {
    framebuffer_rwl: &'a RwLock<Buffer2D>,
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

impl<'a> DisplacementMapScene<'a> {
    pub fn new(
        framebuffer_rwl: &'a RwLock<Buffer2D>,
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
        let mut camera: Camera = Camera::new(
            aspect_ratio,
            Vec3 {
                x: 0.0,
                y: 0.0,
                z: -12.0,
            },
            Default::default(),
        );

        camera.movement_speed = 15.0;

        // Define lights for our scene
        let ambient_light = AmbientLight {
            intensities: Vec3 {
                x: 0.1,
                y: 0.1,
                z: 0.1,
            },
        };

        context.set_ambient_light(ambient_light);

        let directional_light = DirectionalLight {
            intensities: Vec3 {
                x: 0.2,
                y: 0.2,
                z: 0.2,
            },
            direction: Vec4 {
                x: 1.0,
                y: -1.0,
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
        pipeline.geometry_shader_options.displacement_mapping_active = true;

        return DisplacementMapScene {
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

impl<'a> Scene for DisplacementMapScene<'a> {
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

        let orbital_radius: f32 = 6.0;

        self.point_light.position = Vec3 {
            x: 4.0 + orbital_radius * self.seconds_ellapsed.sin(),
            y: orbital_radius * self.seconds_ellapsed.cos(),
            z: -4.0,
        };

        context.set_point_light(0, self.point_light);

        // let mut entities = self.entities.write().unwrap();

        // let rotation_speed = 0.1;

        // for entity in entities.as_mut_slice() {
        //     entity.rotation.z +=
        //         1.0 * rotation_speed * PI * app.timing_info.seconds_since_last_update;
        //     entity.rotation.z %= 2.0 * PI;

        //     entity.rotation.x +=
        //         1.0 * rotation_speed * PI * app.timing_info.seconds_since_last_update;
        //     entity.rotation.x %= 2.0 * PI;

        //     entity.rotation.y +=
        //         1.0 * rotation_speed * PI * app.timing_info.seconds_since_last_update;
        //     entity.rotation.y %= 2.0 * PI;
        // }
    }

    fn render(&mut self) {
        self.pipeline.options.face_culling_strategy.reject = PipelineFaceCullingReject::None;

        self.pipeline.bind_framebuffer(Some(&self.framebuffer_rwl));

        let camera = self.cameras[self.active_camera_index];

        self.pipeline
            .set_projection_z_near(camera.get_projection_z_near());

        self.pipeline
            .set_projection_z_far(camera.get_projection_z_far());

        self.pipeline.begin_frame(None);

        for entity in self.entities.read().unwrap().as_slice() {
            self.pipeline.render_entity(&entity, Some(self.materials));
        }

        self.pipeline.render_point_light(
            &self.point_light,
            Some(&camera),
            Some(&mut self.materials),
        );

        // self.pipeline.render_ground_plane(1.0);

        self.pipeline.end_frame()
    }
}