use std::{borrow::BorrowMut, f32::consts::PI, sync::RwLock};

use cairo::{
    context::ApplicationRenderingContext,
    device::{GameControllerState, KeyboardState, MouseState},
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
    shader::fragment::FragmentShader,
    shader::vertex::VertexShader,
    shader::ShaderContext,
    shaders::{
        default_fragment_shader::DefaultFragmentShader, default_vertex_shader::DefaultVertexShader,
    },
    texture::cubemap::CubeMap,
    vec::{vec3::Vec3, vec4::Vec4},
};

pub struct SkyboxScene<'a> {
    pipeline: Pipeline<'a>,
    cameras: Vec<Camera>,
    active_camera_index: usize,
    point_lights: Vec<PointLight>,
    spot_lights: Vec<SpotLight>,
    entities: &'a RwLock<Vec<&'a mut Entity<'a>>>,
    material_cache: &'a mut MaterialCache,
    shader_context: &'a RwLock<ShaderContext>,
    skybox: CubeMap,
    prev_mouse_state: MouseState,
}

impl<'a> SkyboxScene<'a> {
    pub fn new(
        graphics: Graphics,
        rendering_context: &ApplicationRenderingContext,
        entities: &'a RwLock<Vec<&'a mut Entity<'a>>>,
        material_cache: &'a mut MaterialCache,
        shader_context: &'a RwLock<ShaderContext>,
    ) -> Self {
        // Set up a camera for rendering our scene
        let camera: Camera = Camera::new(
            graphics.buffer.width_over_height,
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

        let world_transform = Mat4::new();

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

        // Option 1. Skybox as a set of 6 separate textures.

        let mut skybox = CubeMap::new([
            "examples/skybox/assets/front.jpg",
            "examples/skybox/assets/back.jpg",
            "examples/skybox/assets/top.jpg",
            "examples/skybox/assets/bottom.jpg",
            "examples/skybox/assets/left.jpg",
            "examples/skybox/assets/right.jpg",
        ]);

        // Option 2. Skybox as one horizontal cross texture.

        // let mut skybox = CubeMap::from_cross("examples/skybox/assets/temple.png");

        // Option 3. Skybox as one vertical cross texture.

        // let mut skybox = CubeMap::from_cross("examples/skybox/assets/vertical_cross.png");

        skybox.load(rendering_context).unwrap();

        return SkyboxScene {
            pipeline,
            entities,
            material_cache,
            shader_context,
            skybox,
            cameras: vec![camera],
            active_camera_index: 0,
            point_lights: vec![point_light],
            spot_lights: vec![spot_light],
            prev_mouse_state: MouseState::new(),
        };
    }
}

impl<'a> Scene for SkyboxScene<'a> {
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

        let mut entities = self.entities.write().unwrap();

        let entity = &mut entities[0];

        // Mesh rotation via our time delta

        entity.rotation.z += 0.2 * PI * seconds_since_last_update;
        entity.rotation.z %= 2.0 * PI;

        entity.rotation.x += 0.2 * PI * seconds_since_last_update;
        entity.rotation.x %= 2.0 * PI;

        entity.rotation.y += 0.2 * PI * seconds_since_last_update;
        entity.rotation.y %= 2.0 * PI;

        let world_transform = Mat4::scaling(1.0)
            * Mat4::rotation_x(entity.rotation.x)
            * Mat4::rotation_y(entity.rotation.y)
            * Mat4::rotation_z(entity.rotation.z)
            * Mat4::translation(entity.position);

        context.set_world_transform(world_transform);

        let camera_view_inverse_transform = camera.get_view_inverse_transform();

        context.set_view_inverse_transform(camera_view_inverse_transform);

        self.prev_mouse_state = mouse_state.clone();
    }

    fn render(&mut self) {
        self.pipeline.begin_frame();

        {
            let mut context = self.shader_context.write().unwrap();

            let mat_raw_mut = &self.skybox as *const CubeMap;

            context.set_active_environment_map(Some(mat_raw_mut));
        }

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

        {
            let mut context = self.shader_context.write().unwrap();

            context.set_active_environment_map(None);
        }

        let camera = self.cameras[self.active_camera_index];

        self.pipeline.render_skybox(&self.skybox, &camera);

        self.pipeline.render_point_light(
            &self.point_lights[0],
            Some(&camera),
            Some(&mut self.material_cache),
        );
    }

    fn get_pixel_data(&self) -> &Vec<u32> {
        return self.pipeline.get_pixel_data();
    }
}
