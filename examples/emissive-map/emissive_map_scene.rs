use std::{borrow::BorrowMut, f32::consts::PI, sync::RwLock};

use cairo::{
    device::{GameControllerState, KeyboardState, MouseState},
    entity::Entity,
    graphics::{pixelbuffer::PixelBuffer, Graphics},
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
        debug_shaders::emissive_fragment_shader::EmissiveFragmentShader,
        default_fragment_shader::DefaultFragmentShader,
        default_geometry_shader::DefaultGeometryShader, default_vertex_shader::DefaultVertexShader,
    },
    vec::{vec3::Vec3, vec4::Vec4},
};

pub struct EmissiveMapScene<'a> {
    pipeline: Pipeline<'a, DefaultFragmentShader<'a>>,
    cameras: Vec<Camera>,
    active_camera_index: usize,
    directional_light: DirectionalLight,
    point_light: PointLight,
    spot_light: SpotLight,
    entities: &'a RwLock<Vec<&'a mut Entity<'a>>>,
    materials: &'a MaterialCache,
    shader_context: &'a RwLock<ShaderContext>,
    prev_mouse_state: MouseState,
    seconds_ellapsed: f32,
}

impl<'a> EmissiveMapScene<'a> {
    pub fn new(
        canvas_width: u32,
        canvas_height: u32,
        entities: &'a RwLock<Vec<&'a mut Entity<'a>>>,
        materials: &'a MaterialCache,
        shader_context: &'a RwLock<ShaderContext>,
    ) -> Self {
        let graphics = Graphics {
            buffer: PixelBuffer::new(canvas_width, canvas_height),
        };

        // Set up a camera for rendering our scene
        let camera: Camera = Camera::new(
            graphics.buffer.width_over_height,
            Vec3 {
                x: 0.0,
                y: 2.0,
                z: -8.0,
            },
            Default::default(),
        );

        // Define lights for our scene
        let ambient_light = AmbientLight {
            intensities: Vec3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
        };

        let directional_light = DirectionalLight {
            intensities: Vec3 {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            direction: Vec4 {
                x: -1.0,
                y: 0.0,
                z: 1.0,
                w: 1.0,
            },
        };

        let point_light = PointLight::new();

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

        // let fragment_shader = EmissiveFragmentShader::new(shader_context);

        let pipeline = Pipeline::new(
            graphics,
            camera.get_projection_z_near(),
            camera.get_projection_z_far(),
            shader_context,
            vertex_shader,
            geometry_shader,
            fragment_shader,
            pipeline_options,
        );

        return EmissiveMapScene {
            pipeline,
            entities,
            materials,
            shader_context,
            cameras: vec![camera],
            active_camera_index: 0,
            // ambient_light,
            directional_light,
            point_light,
            spot_light,
            prev_mouse_state: MouseState::new(),
            seconds_ellapsed: 0.0,
        };
    }
}

impl<'a> Scene for EmissiveMapScene<'a> {
    fn update(
        &mut self,
        keyboard_state: &KeyboardState,
        mouse_state: &MouseState,
        game_controller_state: &GameControllerState,
        seconds_since_last_update: f32,
    ) {
        let mut context = self.shader_context.write().unwrap();

        self.seconds_ellapsed += seconds_since_last_update;

        let camera = (self.cameras[self.active_camera_index]).borrow_mut();

        camera.update(
            keyboard_state,
            mouse_state,
            game_controller_state,
            seconds_since_last_update,
        );

        let camera_view_inverse_transform = camera.get_view_inverse_transform();

        context.set_view_inverse_transform(camera_view_inverse_transform);

        context.set_directional_light(DirectionalLight {
            intensities: self.directional_light.intensities,
            direction: (self.directional_light.direction * camera_view_inverse_transform)
                .as_normal(),
        });

        self.pipeline
            .options
            .update(keyboard_state, mouse_state, game_controller_state);

        self.pipeline
            .geometry_shader
            .update(keyboard_state, mouse_state, game_controller_state);

        context.set_camera_position(Vec4::new(camera.get_position(), 1.0));

        context.set_projection(camera.get_projection());

        let phase_shift = 2.0 * PI / 3.0;
        let max_intensity: f32 = 0.5;

        self.point_light.intensities = Vec3 {
            x: (self.seconds_ellapsed + phase_shift).sin() / 2.0 + 0.5,
            y: (self.seconds_ellapsed + phase_shift).sin() / 2.0 + 0.5,
            z: (self.seconds_ellapsed + phase_shift).sin() / 2.0 + 0.5,
        } * max_intensity;

        let orbital_radius: f32 = 3.0;

        self.point_light.position = Vec3 {
            x: orbital_radius * self.seconds_ellapsed.sin(),
            y: 3.0,
            z: orbital_radius * self.seconds_ellapsed.cos(),
        };

        context.set_point_light(0, self.point_light);

        let mut entities = self.entities.write().unwrap();

        let rotation_speed = 0.3;

        for entity in entities.as_mut_slice() {
            // Mesh rotation via our time delta

            if entity.mesh.object_name == "point_light" {
                entity.position = self.point_light.position;
            }

            if entity.mesh.object_name == "plane" || entity.mesh.object_name == "point_light" {
                continue;
            }

            entity.rotation.z += 1.0 * rotation_speed * PI * seconds_since_last_update;
            entity.rotation.z %= 2.0 * PI;

            entity.rotation.x += 1.0 * rotation_speed * PI * seconds_since_last_update;
            entity.rotation.x %= 2.0 * PI;

            entity.rotation.y += 1.0 * rotation_speed * PI * seconds_since_last_update;
            entity.rotation.y %= 2.0 * PI;
        }

        self.prev_mouse_state = mouse_state.clone();
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
