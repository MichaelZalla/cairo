use std::{borrow::BorrowMut, sync::RwLock};

use cairo::{
    app::App,
    buffer::Buffer2D,
    context::ApplicationRenderingContext,
    device::{GameControllerState, KeyboardState, MouseState},
    entity::Entity,
    material::cache::MaterialCache,
    pipeline::{options::PipelineOptions, Pipeline},
    scene::{
        camera::Camera,
        light::{AmbientLight, DirectionalLight, PointLight, SpotLight},
        Scene,
    },
    shader::fragment::FragmentShader,
    shader::geometry::GeometryShader,
    shader::vertex::VertexShader,
    shader::ShaderContext,
    shaders::{
        default_fragment_shader::DefaultFragmentShader,
        default_geometry_shader::DefaultGeometryShader, default_vertex_shader::DefaultVertexShader,
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
    framebuffer_rwl: &'a RwLock<Buffer2D>,
    pipeline: Pipeline<'a, DefaultFragmentShader<'a>>,
    cameras: Vec<Camera>,
    active_camera_index: usize,
    point_lights: Vec<PointLight>,
    spot_lights: Vec<SpotLight>,
    entities: &'a RwLock<Vec<Entity<'a>>>,
    skybox: CubeMap,
    materials: &'a mut MaterialCache,
    shader_context: &'a RwLock<ShaderContext>,
}

impl<'a> SponzaScene<'a> {
    pub fn new(
        framebuffer_rwl: &'a RwLock<Buffer2D>,
        rendering_context: &ApplicationRenderingContext,
        entities: &'a RwLock<Vec<Entity<'a>>>,
        materials: &'a mut MaterialCache,
        shader_context: &'a RwLock<ShaderContext>,
    ) -> Self {
        let framebuffer = framebuffer_rwl.read().unwrap();

        let vertex_shader = DefaultVertexShader::new(shader_context);

        let geometry_shader = DefaultGeometryShader::new(shader_context, None);

        let fragment_shader = DefaultFragmentShader::new(shader_context);

        let aspect_ratio = framebuffer.width_over_height;

        // Set up a camera for rendering our scene
        let mut camera: Camera = Camera::new(
            aspect_ratio,
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

        let pipeline_options: PipelineOptions = Default::default();

        let view_position = Vec4::new(camera.get_position(), 1.0);

        let view_inverse_transform = camera.get_view_inverse_transform();

        let projection_transform = camera.get_projection();

        let mut context = shader_context.write().unwrap();

        context.set_camera_position(view_position);
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

        return SponzaScene {
            framebuffer_rwl,
            pipeline,
            entities,
            skybox,
            materials,
            shader_context,
            cameras: vec![camera],
            active_camera_index: 0,
            point_lights: vec![point_light],
            spot_lights: vec![spot_light],
        };
    }
}

impl<'a> Scene for SponzaScene<'a> {
    fn update(
        &mut self,
        app: &App,
        keyboard_state: &KeyboardState,
        mouse_state: &MouseState,
        game_controller_state: &GameControllerState,
    ) {
        let mut context = self.shader_context.write().unwrap();

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

        context.set_camera_position(Vec4::new(camera.get_position(), 1.0));

        context.set_projection(camera.get_projection());

        context.set_point_light(0, self.point_lights[0]);

        let camera_view_inverse_transform = camera.get_view_inverse_transform();

        context.set_view_inverse_transform(camera_view_inverse_transform);
    }

    fn render(&mut self) {
        self.pipeline.bind_framebuffer(Some(&self.framebuffer_rwl));

        self.pipeline.begin_frame();

        let camera = self.cameras[self.active_camera_index];

        for entity in self.entities.read().unwrap().as_slice() {
            self.pipeline.render_entity(&entity, Some(self.materials));
        }

        self.pipeline.render_world_axes(300.0);

        self.pipeline.render_point_light(
            &self.point_lights[0],
            Some(&camera),
            Some(&mut self.materials),
        );

        self.pipeline.render_spot_light(
            &self.spot_lights[0],
            Some(&camera),
            Some(&mut self.materials),
        );

        self.pipeline.render_skybox(&self.skybox, &camera);

        self.pipeline.end_frame();
    }
}
