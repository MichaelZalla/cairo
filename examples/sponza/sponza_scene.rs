use std::{borrow::BorrowMut, sync::RwLock};

use cairo::{
    app::App,
    buffer::Buffer2D,
    color,
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
    shader::{geometry::GeometryShader, ShaderContext},
    shaders::{
        default_fragment_shader::DEFAULT_FRAGMENT_SHADER,
        default_geometry_shader::DefaultGeometryShader,
        default_vertex_shader::DEFAULT_VERTEX_SHADER,
    },
    texture::cubemap::CubeMap,
    vec::{
        vec3::{self, Vec3},
        vec4::Vec4,
    },
};

static SPONZA_CENTER: Vec3 = Vec3 {
    x: -572.3847 + 500.0,
    y: 233.06613,
    z: -43.05618,
};

pub struct SponzaScene<'a> {
    framebuffer_rwl: &'a RwLock<Buffer2D>,
    pipeline: Pipeline<'a>,
    cameras: Vec<Camera>,
    active_camera_index: usize,
    directional_light: DirectionalLight,
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

        let vertex_shader = DEFAULT_VERTEX_SHADER;

        let geometry_shader = DefaultGeometryShader::new(shader_context, None);

        let fragment_shader = DEFAULT_FRAGMENT_SHADER;

        let aspect_ratio = framebuffer.width_over_height;

        // Set up a camera for rendering our scene
        let camera_position = Vec3 {
            x: 1000.0,
            y: 300.0,
            z: 0.0,
        };

        let mut camera: Camera =
            Camera::new(aspect_ratio, camera_position, camera_position + vec3::LEFT);

        camera.movement_speed = 300.0;

        camera.set_projection_z_far(10000.0);

        // Define lights for our scene
        let ambient_light = AmbientLight {
            intensities: Vec3::ones() * 0.05,
        };

        let directional_light = DirectionalLight {
            intensities: Vec3::ones() * 0.05,
            direction: Vec4::new(vec3::UP * -1.0, 1.0).as_normal(),
        };

        let mut point_light = PointLight::new();

        point_light.intensities = color::BLUE.to_vec3() / 255.0 * 15.0;

        point_light.specular_intensity = 1.0;

        point_light.constant_attenuation = 1.0;
        point_light.linear_attenuation = 0.007;
        point_light.quadratic_attenuation = 0.0002;

        let mut spot_light = SpotLight::new();

        spot_light.intensities = color::RED.to_vec3() / 255.0 * 15.0;

        spot_light.constant_attenuation = 1.0;
        spot_light.linear_attenuation = 0.007;
        spot_light.quadratic_attenuation = 0.0002;

        let mut skybox = CubeMap::from_cross("examples/skybox/assets/grass_sky.jpg");

        skybox.load(rendering_context).unwrap();

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

        return SponzaScene {
            framebuffer_rwl,
            pipeline,
            entities,
            skybox,
            materials,
            shader_context,
            cameras: vec![camera],
            active_camera_index: 0,
            directional_light,
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

        context.set_view_position(Vec4::new(camera.look_vector.get_position(), 1.0));

        context.set_projection(camera.get_projection());

        let uptime = app.timing_info.uptime_seconds;

        self.directional_light.direction = Vec4::new(
            Vec3 {
                x: uptime.sin(),
                y: -1.0,
                z: uptime.cos(),
            },
            1.0,
        )
        .as_normal();

        context.set_directional_light(self.directional_light);

        self.point_lights[0].position = SPONZA_CENTER
            + Vec3 {
                x: 1000.0 * (uptime).sin(),
                y: 300.0,
                z: 0.0,
            };

        context.set_point_light(0, self.point_lights[0]);

        self.spot_lights[0].look_vector.set_position(
            SPONZA_CENTER
                + Vec3 {
                    x: -1000.0 * (uptime).sin(),
                    y: 500.0,
                    z: 0.0,
                },
        );

        context.set_spot_light(0, self.spot_lights[0]);

        let camera_view_inverse_transform = camera.get_view_inverse_transform();

        context.set_view_inverse_transform(camera_view_inverse_transform);
    }

    fn render(&mut self) {
        self.pipeline.bind_framebuffer(Some(&self.framebuffer_rwl));

        self.pipeline.begin_frame(None);

        let camera = self.cameras[self.active_camera_index];

        for entity in self.entities.read().unwrap().as_slice() {
            self.pipeline.render_entity(&entity, Some(self.materials));
        }

        self.pipeline.render_ground_plane(25.0);

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
