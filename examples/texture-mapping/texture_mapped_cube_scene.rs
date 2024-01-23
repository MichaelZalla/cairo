use std::{borrow::BorrowMut, f32::consts::PI, sync::RwLock};

use cairo::{
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
    time::TimingInfo,
    vec::{vec3::Vec3, vec4::Vec4},
};

pub struct TextureMappedCubeScene<'a> {
    pipeline: Pipeline<'a>,
    cameras: Vec<Camera>,
    active_camera_index: usize,
    entities: &'a RwLock<Vec<&'a mut Entity<'a>>>,
    materials: &'a MaterialCache,
    shader_context: &'a RwLock<ShaderContext>,
}

impl<'a> TextureMappedCubeScene<'a> {
    pub fn new(
        canvas_width: u32,
        canvas_height: u32,
        entities: &'a RwLock<Vec<&'a mut Entity<'a>>>,
        materials: &'a MaterialCache,
        shader_context: &'a RwLock<ShaderContext>,
    ) -> Self {
        // Set up a camera for rendering our scene
        let camera: Camera = Camera::new(
            canvas_width as f32 / canvas_height as f32,
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
                x: 0.4,
                y: 0.4,
                z: 0.4,
            },
        };

        let directional_light = DirectionalLight {
            intensities: Vec3 {
                x: 0.3,
                y: 0.3,
                z: 0.3,
            },
            direction: Vec4 {
                x: 0.0,
                y: -1.0,
                z: 0.0,
                w: 1.0,
            },
        };

        let mut point_light = PointLight::new();

        point_light.intensities = Vec3 {
            x: 0.7,
            y: 0.7,
            z: 0.7,
        };

        point_light.position = Vec3 {
            x: 0.0,
            y: 4.0,
            z: 0.0,
        };

        let spot_light = SpotLight::new();

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

        let vertex_shader = DefaultVertexShader::new(shader_context);

        let geometry_shader = DefaultGeometryShader::new(shader_context, None);

        let fragment_shader = DefaultFragmentShader::new(shader_context);

        let pipeline = Pipeline::new(
            canvas_width,
            canvas_height,
            camera.get_projection_z_near(),
            camera.get_projection_z_far(),
            shader_context,
            vertex_shader,
            geometry_shader,
            fragment_shader,
            pipeline_options,
        );

        return TextureMappedCubeScene {
            pipeline,
            entities,
            materials,
            shader_context,
            cameras: vec![camera],
            active_camera_index: 0,
        };
    }
}

impl<'a> Scene for TextureMappedCubeScene<'a> {
    fn update(
        &mut self,
        timing_info: &TimingInfo,
        keyboard_state: &KeyboardState,
        mouse_state: &MouseState,
        game_controller_state: &GameControllerState,
    ) {
        let mut context = self.shader_context.write().unwrap();

        let camera = (self.cameras[self.active_camera_index]).borrow_mut();

        camera.update(
            timing_info,
            keyboard_state,
            mouse_state,
            game_controller_state,
        );

        let camera_view_inverse_transform = camera.get_view_inverse_transform();

        context.set_view_inverse_transform(camera_view_inverse_transform);

        self.pipeline
            .options
            .update(keyboard_state, mouse_state, game_controller_state);

        self.pipeline
            .geometry_shader
            .update(keyboard_state, mouse_state, game_controller_state);

        context.set_camera_position(Vec4::new(camera.get_position(), 1.0));

        context.set_projection(camera.get_projection());

        let mut entities = self.entities.write().unwrap();

        let rotation_speed = 0.1 * timing_info.seconds_since_last_update;

        for entity in entities.as_mut_slice() {
            // Mesh rotation via our time delta

            entity.rotation.z += 1.0 * rotation_speed * PI;
            entity.rotation.z %= 2.0 * PI;

            entity.rotation.x += 1.0 * rotation_speed * PI;
            entity.rotation.x %= 2.0 * PI;

            entity.rotation.y += 1.0 * rotation_speed * PI;
            entity.rotation.y %= 2.0 * PI;
        }
    }

    fn render(&mut self) {
        self.pipeline.begin_frame();

        for entity in self.entities.read().unwrap().as_slice() {
            self.pipeline.render_entity(&entity, Some(self.materials));
        }

        self.pipeline.end_frame();
    }

    fn get_pixel_data(&self) -> &Vec<u32> {
        return self.pipeline.get_pixel_data();
    }
}
