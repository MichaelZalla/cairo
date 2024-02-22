extern crate sdl2;

use std::{f32::consts::PI, sync::RwLock};

use uuid::Uuid;

use cairo::{
    app::{App, AppWindowInfo},
    buffer::framebuffer::Framebuffer,
    device::{GameControllerState, KeyboardState, MouseState},
    entity::Entity,
    material::{cache::MaterialCache, Material},
    mesh,
    pipeline::Pipeline,
    resource::arena::Arena,
    scene::{
        camera::Camera,
        light::{AmbientLight, DirectionalLight, PointLight, SpotLight},
    },
    shader::context::ShaderContext,
    shaders::{
        default_fragment_shader::DEFAULT_FRAGMENT_SHADER,
        default_vertex_shader::DEFAULT_VERTEX_SHADER,
    },
    texture::map::{TextureMap, TextureMapStorageFormat},
    vec::{vec3::Vec3, vec4::Vec4},
};

fn main() -> Result<(), String> {
    let mut window_info = AppWindowInfo {
        title: "examples/scenegraph".to_string(),
        relative_mouse_mode: true,
        ..Default::default()
    };

    let app = App::new(&mut window_info);

    // Borrow our app's rendering context, which we can use to load texture
    // files.

    let rendering_context = &app.context.rendering_context;

    // Default framebuffer

    let mut framebuffer = Framebuffer::new(window_info.canvas_width, window_info.canvas_height);

    framebuffer.complete(0.3, 100.0);

    let framebuffer_rwl = RwLock::new(framebuffer);

    // Meshes

    let mut plane_mesh = mesh::primitive::plane::generate(80.0, 80.0, 8, 8);
    let mut cube_mesh = mesh::primitive::cube::generate(2.0, 2.0, 2.0);

    // Checkerboard material

    let mut checkerboard_material = Material::new("checkerboard".to_string());

    let mut checkerboard_diffuse_map = TextureMap::new(
        &"./assets/textures/checkerboard.jpg",
        TextureMapStorageFormat::Index8,
    );

    checkerboard_diffuse_map.load(rendering_context)?;

    let checkerboard_specular_map = checkerboard_diffuse_map.clone();

    checkerboard_material.diffuse_map = Some(checkerboard_diffuse_map);

    checkerboard_material.specular_map = Some(checkerboard_specular_map);

    // Assign textures to mesh materials

    plane_mesh.material_name = Some(checkerboard_material.name.clone());
    cube_mesh.material_name = Some(checkerboard_material.name.clone());

    // Collect materials

    let mut material_cache: MaterialCache = Default::default();

    material_cache.insert(checkerboard_material);

    // Set up resource arenas for the various node types in our scene.

    let mut entity_arena: Arena<Entity> = Arena::<Entity>::new();
    let mut camera_arena: Arena<Camera> = Arena::<Camera>::new();
    let mut ambient_light_arena: Arena<AmbientLight> = Arena::<AmbientLight>::new();
    let mut directional_light_arena: Arena<DirectionalLight> = Arena::<DirectionalLight>::new();
    let mut point_light_arena: Arena<PointLight> = Arena::<PointLight>::new();
    let mut spot_light_arena: Arena<SpotLight> = Arena::<SpotLight>::new();

    // Assign the meshes to entities

    let plane_entity: Entity<'_> = Entity::new(&plane_mesh);

    let mut cube_entity = Entity::new(&cube_mesh);
    cube_entity.position.y = 3.0;

    let cube_position = cube_entity.position.clone();

    // Set up a camera for our scene.

    let aspect_ratio = framebuffer_rwl.read().unwrap().width_over_height;

    let mut camera: Camera = Camera::from_perspective(
        Vec3 {
            x: 0.0,
            y: 6.0,
            z: -12.0,
        },
        cube_position,
        75.0,
        aspect_ratio,
    );

    camera.movement_speed = 10.0;

    // Set up some lights for our scene.

    let ambient_light: AmbientLight = Default::default();

    let directional_light = DirectionalLight {
        intensities: Default::default(),
        direction: Vec4 {
            x: -1.0,
            y: -1.0,
            z: 1.0,
            w: 1.0,
        },
    };

    let point_light = PointLight::new();

    let mut spot_light = SpotLight::new();

    spot_light.look_vector.set_position(Vec3 {
        x: 2.0,
        y: 12.0,
        z: 2.0,
    });

    spot_light.intensities = Vec3::ones() * 0.15;

    spot_light.look_vector.set_position(Vec3 {
        y: 10.0,
        ..spot_light.look_vector.get_position()
    });

    // Bind initial data to the shader context.

    let shader_context_rwl: RwLock<ShaderContext> = Default::default();

    {
        let mut context = shader_context_rwl.write().unwrap();

        let view_position = Vec4::new(camera.look_vector.get_position(), 1.0);

        let view_inverse_transform = camera.get_view_inverse_transform();

        let projection_transform = camera.get_projection();

        context.set_view_position(view_position);
        context.set_view_inverse_transform(view_inverse_transform);
        context.set_projection(projection_transform);

        context.set_ambient_light(ambient_light);
        context.set_directional_light(directional_light);
        context.set_point_light(0, point_light);
        context.set_spot_light(0, spot_light);
    }

    // Pipeline

    let mut pipeline = Pipeline::new(
        &shader_context_rwl,
        DEFAULT_VERTEX_SHADER,
        DEFAULT_FRAGMENT_SHADER,
        Default::default(),
    );

    pipeline.geometry_shader_options.emissive_mapping_active = true;

    let pipeline_rwl = RwLock::new(pipeline);

    // App update and render callbacks

    let plane_entity_handle = entity_arena.insert(Uuid::new_v4(), plane_entity);
    let cube_entity_handle = entity_arena.insert(Uuid::new_v4(), cube_entity);
    let camera_handle = camera_arena.insert(Uuid::new_v4(), camera);
    let _ambient_light_handle = ambient_light_arena.insert(Uuid::new_v4(), ambient_light);
    let _directional_light_handle =
        directional_light_arena.insert(Uuid::new_v4(), directional_light);
    let point_light_handle = point_light_arena.insert(Uuid::new_v4(), point_light);
    let spot_light_handle = spot_light_arena.insert(Uuid::new_v4(), spot_light);

    let entity_arena_rwl = RwLock::new(entity_arena);
    let camera_arena_rwl = RwLock::new(camera_arena);
    // let ambient_light_arena_rwl = RwLock::new(ambient_light_arena);
    // let directional_light_arena_rwl = RwLock::new(directional_light_arena);
    let point_light_arena_rwl = RwLock::new(point_light_arena);
    let spot_light_arena_rwl = RwLock::new(spot_light_arena);

    let mut update = |app: &mut App,
                      keyboard_state: &KeyboardState,
                      mouse_state: &MouseState,
                      game_controller_state: &GameControllerState|
     -> Result<(), String> {
        let mut context = shader_context_rwl.write().unwrap();

        let uptime = app.timing_info.uptime_seconds;

        match camera_arena_rwl.write().unwrap().get_mut(&camera_handle) {
            Ok(entry) => {
                let camera = &mut entry.item;

                camera.update(
                    &app.timing_info,
                    keyboard_state,
                    mouse_state,
                    game_controller_state,
                );

                let camera_view_inverse_transform = camera.get_view_inverse_transform();

                context.set_view_position(Vec4::new(camera.look_vector.get_position(), 1.0));

                context.set_view_inverse_transform(camera_view_inverse_transform);

                context.set_projection(camera.get_projection());
            }
            Err(err) => {
                panic!(
                    "Failed to get Camera from Arena with Handle {:?}: {}",
                    camera_handle, err
                );
            }
        }

        let mut pipeline = pipeline_rwl.write().unwrap();

        pipeline
            .options
            .update(keyboard_state, mouse_state, game_controller_state);

        pipeline
            .geometry_shader_options
            .update(keyboard_state, mouse_state, game_controller_state);

        // Update point lights.

        match point_light_arena_rwl
            .write()
            .unwrap()
            .get_mut(&point_light_handle)
        {
            Ok(entry) => {
                let point_light = &mut entry.item;

                static POINT_LIGHT_INTENSITY_PHASE_SHIFT: f32 = 2.0 * PI / 3.0;
                static MAX_POINT_LIGHT_INTENSITY: f32 = 0.5;

                point_light.intensities = Vec3 {
                    x: (uptime + POINT_LIGHT_INTENSITY_PHASE_SHIFT).sin() / 2.0 + 0.5,
                    y: (uptime + POINT_LIGHT_INTENSITY_PHASE_SHIFT).sin() / 2.0 + 0.5,
                    z: (uptime + POINT_LIGHT_INTENSITY_PHASE_SHIFT).sin() / 2.0 + 0.5,
                } * MAX_POINT_LIGHT_INTENSITY;

                let orbital_radius: f32 = 3.0;

                point_light.position = Vec3 {
                    x: orbital_radius * uptime.sin(),
                    y: 3.0,
                    z: orbital_radius * uptime.cos(),
                };

                context.set_point_light(0, point_light.clone());
            }
            Err(err) => {
                panic!(
                    "Failed to get PointLight from Arena with Handle {:?}: {}",
                    point_light_handle, err
                );
            }
        }

        // Update entities.

        let mut entity_arena = entity_arena_rwl.write().unwrap();

        match entity_arena.get_mut(&cube_entity_handle) {
            Ok(entry) => {
                let cube_entity = &mut entry.item;

                static ENTITY_ROTATION_SPEED: f32 = 0.3;

                cube_entity.rotation.z +=
                    1.0 * ENTITY_ROTATION_SPEED * PI * app.timing_info.seconds_since_last_update;

                cube_entity.rotation.z %= 2.0 * PI;

                cube_entity.rotation.x +=
                    1.0 * ENTITY_ROTATION_SPEED * PI * app.timing_info.seconds_since_last_update;

                cube_entity.rotation.x %= 2.0 * PI;

                cube_entity.rotation.y +=
                    1.0 * ENTITY_ROTATION_SPEED * PI * app.timing_info.seconds_since_last_update;

                cube_entity.rotation.y %= 2.0 * PI;
            }
            Err(err) => {
                panic!(
                    "Failed to get Entity from Arena with Handle {:?}: {}",
                    cube_entity_handle, err
                );
            }
        }

        Ok(())
    };

    let mut render = || -> Result<Vec<u32>, String> {
        // Delegate the rendering to our scene.

        let mut pipeline = pipeline_rwl.write().unwrap();

        pipeline.bind_framebuffer(Some(&framebuffer_rwl));

        // Begin frame

        pipeline.begin_frame();

        // Render entities.

        let entity_arena = entity_arena_rwl.read().unwrap();

        match entity_arena.get(&plane_entity_handle) {
            Ok(entry) => {
                let plane_entity = &entry.item;

                pipeline.render_entity(plane_entity, Some(&material_cache));
            }
            Err(err) => {
                panic!(
                    "Failed to get Entity from Arena with Handle {:?}: {}",
                    plane_entity_handle, err
                );
            }
        }

        match entity_arena.get(&cube_entity_handle) {
            Ok(entry) => {
                let cube_entity = &entry.item;

                pipeline.render_entity(cube_entity, Some(&material_cache));
            }
            Err(err) => {
                panic!(
                    "Failed to get Entity from Arena with Handle {:?}: {}",
                    cube_entity_handle, err
                );
            }
        }

        // Visualize point lights.

        let point_light_arena = point_light_arena_rwl.read().unwrap();

        match point_light_arena.get(&point_light_handle) {
            Ok(entry) => {
                let point_light = &entry.item;

                pipeline.render_point_light(point_light, None, None);
            }
            Err(err) => {
                panic!(
                    "Failed to get PointLight from Arena with Handle {:?}: {}",
                    point_light_handle, err
                );
            }
        }

        // Visualize spot lights.

        let spot_light_arena = spot_light_arena_rwl.read().unwrap();

        match spot_light_arena.get(&spot_light_handle) {
            Ok(entry) => {
                let spot_light = &entry.item;

                pipeline.render_spot_light(spot_light, None, None);
            }
            Err(err) => {
                panic!(
                    "Failed to get SpotLight from Arena with Handle {:?}: {}",
                    spot_light_handle, err
                );
            }
        }

        // End frame

        pipeline.end_frame();

        // Write out.

        let framebuffer = framebuffer_rwl.read().unwrap();

        match framebuffer.attachments.color.as_ref() {
            Some(color_buffer_lock) => {
                let color_buffer = color_buffer_lock.read().unwrap();

                Ok(color_buffer.get_all().clone())
            }
            None => panic!(),
        }
    };

    app.run(&mut update, &mut render)?;

    Ok(())
}
