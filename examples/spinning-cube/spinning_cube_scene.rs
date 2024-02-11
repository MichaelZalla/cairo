use std::{borrow::BorrowMut, f32::consts::PI, sync::RwLock};

use cairo::{
    app::App,
    buffer::Buffer2D,
    device::{GameControllerState, KeyboardState, MouseState},
    entity::Entity,
    pipeline::{options::PipelineOptions, Pipeline},
    scene::{
        camera::Camera,
        light::{AmbientLight, DirectionalLight, PointLight, SpotLight},
        Scene,
    },
    shader::geometry::GeometryShader,
    shader::ShaderContext,
    shaders::{
        default_fragment_shader::DEFAULT_FRAGMENT_SHADER,
        default_geometry_shader::DefaultGeometryShader,
        default_vertex_shader::DEFAULT_VERTEX_SHADER,
    },
    vec::{vec3::Vec3, vec4::Vec4},
};

pub struct SpinningCubeScene<'a> {
    framebuffer_rwl: &'a RwLock<Buffer2D>,
    pipeline: Pipeline<'a>,
    cameras: Vec<Camera>,
    active_camera_index: usize,
    entities: &'a RwLock<Vec<&'a mut Entity<'a>>>,
    shader_context: &'a RwLock<ShaderContext>,
}

impl<'a> SpinningCubeScene<'a> {
    pub fn new(
        framebuffer_rwl: &'a RwLock<Buffer2D>,
        entities: &'a RwLock<Vec<&'a mut Entity<'a>>>,
        shader_context: &'a RwLock<ShaderContext>,
    ) -> Self {
        let framebuffer = framebuffer_rwl.read().unwrap();

        let vertex_shader = DEFAULT_VERTEX_SHADER;

        let geometry_shader = DefaultGeometryShader::new(shader_context, None);

        let fragment_shader = DEFAULT_FRAGMENT_SHADER;

        let aspect_ratio = framebuffer.width_over_height;

        // Set up a camera for rendering our scene
        let camera: Camera = Camera::new(
            aspect_ratio,
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

        let view_position = Vec4::new(camera.look_vector.get_position(), 1.0);

        let view_inverse_transform = camera.get_view_inverse_transform();

        let projection_transform = camera.get_projection();

        let mut context = shader_context.write().unwrap();

        context.set_view_position(view_position);
        context.set_view_inverse_transform(view_inverse_transform);
        context.set_projection(projection_transform);

        context.set_ambient_light(ambient_light);
        context.set_directional_light(directional_light);
        context.set_point_light(0, point_light);
        context.set_spot_light(0, spot_light);

        let pipeline = Pipeline::new(
            shader_context,
            vertex_shader,
            geometry_shader,
            fragment_shader,
            pipeline_options,
        );

        return SpinningCubeScene {
            framebuffer_rwl,
            pipeline,
            entities,
            shader_context,
            cameras: vec![camera],
            active_camera_index: 0,
        };
    }
}

impl<'a> Scene for SpinningCubeScene<'a> {
    fn update(
        &mut self,
        app: &App,
        keyboard_state: &KeyboardState,
        mouse_state: &MouseState,
        game_controller_state: &GameControllerState,
    ) {
        let mut context = self.shader_context.write().unwrap();

        // Apply camera rotation based on mouse position delta

        let camera = (self.cameras[self.active_camera_index]).borrow_mut();

        camera.update(
            &app.timing_info,
            keyboard_state,
            mouse_state,
            game_controller_state,
        );

        self.pipeline
            .options
            .update(keyboard_state, mouse_state, game_controller_state);

        self.pipeline
            .geometry_shader
            .update(keyboard_state, mouse_state, game_controller_state);

        context.set_view_position(Vec4::new(camera.look_vector.get_position(), 1.0));

        context.set_projection(camera.get_projection());

        let mut entities = self.entities.write().unwrap();

        let entity = &mut entities[0];

        // Mesh rotation via our time delta

        entity.rotation.z += 0.2 * PI * app.timing_info.seconds_since_last_update;
        entity.rotation.z %= 2.0 * PI;

        entity.rotation.x += 0.2 * PI * app.timing_info.seconds_since_last_update;
        entity.rotation.x %= 2.0 * PI;

        entity.rotation.y += 0.2 * PI * app.timing_info.seconds_since_last_update;
        entity.rotation.y %= 2.0 * PI;

        let camera_view_inverse_transform = camera.get_view_inverse_transform();

        context.set_view_inverse_transform(camera_view_inverse_transform);
    }

    fn render(&mut self) {
        self.pipeline.bind_framebuffer(Some(&self.framebuffer_rwl));

        self.pipeline.begin_frame(None);

        for entity in self.entities.read().unwrap().as_slice() {
            self.pipeline.render_entity(&entity, None);
        }

        self.pipeline.end_frame();
    }
}
