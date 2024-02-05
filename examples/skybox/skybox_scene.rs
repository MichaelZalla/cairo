use std::{borrow::BorrowMut, f32::consts::PI, sync::RwLock};

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
    shader::ShaderContext,
    shaders::{
        default_fragment_shader::DEFAULT_FRAGMENT_SHADER,
        // default_geometry_shader::DEFAULT_GEOMETRY_SHADER,
        default_vertex_shader::DEFAULT_VERTEX_SHADER,
    },
    texture::{cubemap::CubeMap, map::TextureMapStorageFormat},
    vec::{vec3::Vec3, vec4::Vec4},
};

pub struct SkyboxScene<'a> {
    framebuffer_rwl: &'a RwLock<Buffer2D>,
    pipeline: Pipeline<'a>,
    cameras: Vec<Camera>,
    active_camera_index: usize,
    point_lights: Vec<PointLight>,
    _spot_lights: Vec<SpotLight>,
    entities: &'a RwLock<Vec<&'a mut Entity<'a>>>,
    material_cache: &'a mut MaterialCache,
    shader_context: &'a RwLock<ShaderContext>,
    skybox: CubeMap,
}

impl<'a> SkyboxScene<'a> {
    pub fn new(
        framebuffer_rwl: &'a RwLock<Buffer2D>,
        rendering_context: &ApplicationRenderingContext,
        entities: &'a RwLock<Vec<&'a mut Entity<'a>>>,
        material_cache: &'a mut MaterialCache,
        shader_context: &'a RwLock<ShaderContext>,
    ) -> Self {
        let framebuffer = framebuffer_rwl.read().unwrap();

        let vertex_shader = DEFAULT_VERTEX_SHADER;

        let fragment_shader = DEFAULT_FRAGMENT_SHADER;

        let aspect_ratio = framebuffer.width_over_height;

        // Set up a camera for rendering our scene
        let camera: Camera = Camera::from_perspective(
            Vec3 {
                x: 0.0,
                y: 0.0,
                z: -5.0,
            },
            Default::default(),
            75.0,
            aspect_ratio,
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

        let mut spot_light = SpotLight::new();

        spot_light.look_vector.set_position(Vec3 {
            y: 10.0,
            ..spot_light.look_vector.get_position()
        });

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
            fragment_shader,
            pipeline_options,
        );

        // Option 1. Skybox as a set of 6 separate textures.

        let mut skybox = CubeMap::new(
            [
                "examples/skybox/assets/front.jpg",
                "examples/skybox/assets/back.jpg",
                "examples/skybox/assets/top.jpg",
                "examples/skybox/assets/bottom.jpg",
                "examples/skybox/assets/left.jpg",
                "examples/skybox/assets/right.jpg",
            ],
            TextureMapStorageFormat::RGB24,
        );

        // Option 2. Skybox as one horizontal cross texture.

        // let mut skybox = CubeMap::from_cross("examples/skybox/assets/temple.png");

        // Option 3. Skybox as one vertical cross texture.

        // let mut skybox = CubeMap::from_cross("examples/skybox/assets/vertical_cross.png");

        skybox.load(rendering_context).unwrap();

        return SkyboxScene {
            framebuffer_rwl,
            pipeline,
            entities,
            material_cache,
            shader_context,
            skybox,
            cameras: vec![camera],
            active_camera_index: 0,
            point_lights: vec![point_light],
            _spot_lights: vec![spot_light],
        };
    }
}

impl<'a> Scene for SkyboxScene<'a> {
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

        self.pipeline.geometry_shader_options.update(
            keyboard_state,
            mouse_state,
            game_controller_state,
        );

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

        {
            let mut context = self.shader_context.write().unwrap();

            let cubemap_raw_mut = &self.skybox as *const CubeMap;

            context.set_active_environment_map(Some(cubemap_raw_mut));
        }

        for entity in self.entities.read().unwrap().as_slice() {
            self.pipeline
                .render_entity(&entity, Some(self.material_cache));
        }

        {
            let mut context = self.shader_context.write().unwrap();

            context.set_active_environment_map(None);
        }

        let camera = self.cameras[self.active_camera_index];

        self.pipeline.render_skybox(&self.skybox, &camera);

        // self.pipeline.render_ground_plane(10.0);

        self.pipeline.render_point_light(
            &self.point_lights[0],
            Some(&camera),
            Some(&mut self.material_cache),
        );

        self.pipeline.end_frame();
    }
}
