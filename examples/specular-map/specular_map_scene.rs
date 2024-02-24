use std::{borrow::BorrowMut, cell::RefCell, f32::consts::PI, sync::RwLock};

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
    shader::context::ShaderContext,
    shaders::{
        // debug_shaders::{
        //     specular_intensity_fragment_shader::SpecularIntensityFragmentShader,
        //     specular_roughness_fragment_shader::SpecularRoughnessFragmentShader,
        // },
        default_fragment_shader::DEFAULT_FRAGMENT_SHADER,
        // default_geometry_shader::DEFAULT_GEOMETRY_SHADER,
        default_vertex_shader::DEFAULT_VERTEX_SHADER,
    },
    vec::{vec3::Vec3, vec4::Vec4},
};

pub struct SpecularMapScene<'a> {
    framebuffer_rc: &'a RefCell<Framebuffer>,
    pipeline: Pipeline<'a>,
    cameras: Vec<Camera>,
    active_camera_index: usize,
    point_light: PointLight,
    spot_light: SpotLight,
    entities: &'a RefCell<Vec<&'a mut Entity<'a>>>,
    materials: &'a MaterialCache,
    shader_context: &'a RwLock<ShaderContext>,
}

impl<'a> SpecularMapScene<'a> {
    pub fn new(
        framebuffer_rc: &'a RefCell<Framebuffer>,
        entities: &'a RefCell<Vec<&'a mut Entity<'a>>>,
        materials: &'a MaterialCache,
        shader_context: &'a RwLock<ShaderContext>,
    ) -> Self {
        let framebuffer = framebuffer_rc.borrow();

        let vertex_shader = DEFAULT_VERTEX_SHADER;

        let fragment_shader = DEFAULT_FRAGMENT_SHADER;
        // let fragment_shader = SpecularIntensityFragmentShader::new(shader_context);
        // let fragment_shader = SpecularRoughnessFragmentShader::new(shader_context);

        let aspect_ratio = framebuffer.width_over_height;

        // Set up a camera for rendering our scene
        let camera: Camera = Camera::from_perspective(
            Vec3 {
                x: 0.0,
                y: 3.0,
                z: -6.0,
            },
            Vec3 {
                x: 0.0,
                y: 1.5,
                z: 0.0,
            },
            75.0,
            aspect_ratio,
        );

        // Define lights for our scene
        let ambient_light = AmbientLight {
            intensities: Vec3 {
                x: 0.05,
                y: 0.05,
                z: 0.05,
            },
        };

        let directional_light = DirectionalLight {
            intensities: Vec3 {
                x: 0.05,
                y: 0.05,
                z: 0.05,
            },
            direction: Vec4 {
                x: -1.0,
                y: 0.0,
                z: 1.0,
                w: 1.0,
            },
        };

        let point_light = PointLight::new();

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
        // context.set_spot_light(0, spot_light);

        let mut pipeline = Pipeline::new(
            shader_context,
            vertex_shader,
            fragment_shader,
            pipeline_options,
        );

        pipeline.geometry_shader_options.specular_mapping_active = true;

        return SpecularMapScene {
            framebuffer_rc,
            pipeline,
            entities,
            materials,
            shader_context,
            cameras: vec![camera],
            active_camera_index: 0,
            point_light,
            spot_light,
        };
    }
}

impl<'a> Scene for SpecularMapScene<'a> {
    fn update(
        &mut self,
        app: &App,
        keyboard_state: &KeyboardState,
        mouse_state: &MouseState,
        game_controller_state: &GameControllerState,
    ) {
        let mut context = self.shader_context.write().unwrap();

        let uptime = app.timing_info.uptime_seconds;

        let camera = (self.cameras[self.active_camera_index]).borrow_mut();

        camera.update(
            &app.timing_info,
            keyboard_state,
            mouse_state,
            game_controller_state,
        );

        let camera_view_inverse_transform = camera.get_view_inverse_transform();

        context.set_view_inverse_transform(camera_view_inverse_transform);

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

        let phase_shift = 2.0 * PI / 3.0;
        let max_intensity: f32 = 0.6;

        self.point_light.intensities = Vec3 {
            x: (uptime + phase_shift * 0.0).sin() / 2.0 + 0.5,
            y: (uptime + phase_shift * 1.0).sin() / 2.0 + 0.5,
            z: (uptime + phase_shift * 2.0).sin() / 2.0 + 0.5,
        } * max_intensity;

        self.point_light.intensities *= 3.0;

        let orbital_radius: f32 = 3.0;

        self.point_light.position = Vec3 {
            x: orbital_radius * uptime.sin(),
            y: 1.0,
            z: orbital_radius * uptime.cos(),
        };

        context.set_point_light(0, self.point_light);

        let mut entities = self.entities.borrow_mut();

        let rotation_speed = 0.3;

        for entity in entities.as_mut_slice() {
            // Mesh rotation via our time delta

            if entity.mesh.object_name == "plane" {
                continue;
            }

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
        self.pipeline.bind_framebuffer(Some(&self.framebuffer_rc));

        let camera = self.cameras[self.active_camera_index];

        {
            let framebuffer = self.framebuffer_rc.borrow_mut();

            match framebuffer.attachments.depth.as_ref() {
                Some(lock) => {
                    let mut depth_buffer = lock.borrow_mut();

                    depth_buffer.set_projection_z_near(camera.get_projection_z_near());
                    depth_buffer.set_projection_z_far(camera.get_projection_z_far());
                }
                None => (),
            }
        }

        self.pipeline.begin_frame();

        for entity in self.entities.borrow().as_slice() {
            self.pipeline.render_entity(&entity, Some(self.materials));
        }

        self.pipeline
            .render_point_light(&self.point_light, None, None);

        self.pipeline
            .render_spot_light(&self.spot_light, None, None);

        self.pipeline.end_frame();
    }
}
