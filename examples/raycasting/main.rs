extern crate sdl2;

use std::{cell::RefCell, cmp::Ordering, f32::consts::TAU, rc::Rc};

use sdl2::keyboard::Keycode;

use cairo::{
    app::{
        resolution::{Resolution, RESOLUTION_1280_BY_720},
        App, AppWindowInfo,
    },
    buffer::framebuffer::Framebuffer,
    color::Color,
    device::{game_controller::GameControllerState, keyboard::KeyboardState, mouse::MouseState},
    geometry::{
        accelerator::static_triangle_bvh::{StaticTriangleBVH, StaticTriangleBVHInstance},
        intersect::{intersect_ray_bvh, intersect_ray_triangle},
        primitives::ray,
    },
    matrix::Mat4,
    mesh::{
        obj::load::{load_obj, LoadObjResult, ProcessGeometryFlag},
        Mesh,
    },
    render::{options::RenderOptions, Renderer},
    resource::handle::Handle,
    scene::{
        context::SceneContext,
        node::{SceneNode, SceneNodeType},
        resources::SceneResources,
    },
    shader::context::ShaderContext,
    shaders::{
        default_fragment_shader::DEFAULT_FRAGMENT_SHADER,
        default_vertex_shader::DEFAULT_VERTEX_SHADER,
    },
    software_renderer::SoftwareRenderer,
    transform::quaternion::Quaternion,
    vec::vec3,
};

use scene::make_raycasting_scene;

mod scene;

fn main() -> Result<(), String> {
    let mut window_info = AppWindowInfo {
        title: "examples/raycasting".to_string(),
        relative_mouse_mode: true,
        canvas_resolution: RESOLUTION_1280_BY_720,
        window_resolution: RESOLUTION_1280_BY_720,
        ..Default::default()
    };

    // Render callback

    let render_to_window_canvas = |_frame_index: Option<u32>,
                                   _new_resolution: Option<Resolution>,
                                   _canvas: &mut [u8]|
     -> Result<(), String> { Ok(()) };

    let (app, _event_watch) = App::new(&mut window_info, &render_to_window_canvas);

    // Pipeline framebuffer

    let mut framebuffer = Framebuffer::new(
        window_info.canvas_resolution.width,
        window_info.canvas_resolution.height,
    );

    framebuffer.complete(0.3, 100.0);

    let camera_aspect_ratio = framebuffer.width_over_height;

    // Load some level (collision) geometry.

    let scene_context = SceneContext::default();

    let mut level_meshes = {
        let resources = &scene_context.resources;

        let mut material_arena = resources.material.borrow_mut();
        let mut texture_u8_arena = resources.texture_u8.borrow_mut();

        let LoadObjResult(_level_geometry, level_meshes) = load_obj(
            "./data/blender/collision-level/collision-level_004.obj",
            &mut material_arena,
            &mut texture_u8_arena,
            Some(ProcessGeometryFlag::Null | ProcessGeometryFlag::Center),
        );

        level_meshes
    };

    for mesh in level_meshes.iter_mut() {
        let bvh = StaticTriangleBVH::new(mesh);

        mesh.collider.replace(Rc::new(bvh));
    }

    // Scene context

    let mut level_mesh_handle = Handle::default();

    let (scene, shader_context) = {
        let resources = &scene_context.resources;

        let mut camera_arena = resources.camera.borrow_mut();
        let mut environment_arena = resources.environment.borrow_mut();
        let mut ambient_light_arena = resources.ambient_light.borrow_mut();
        let mut directional_light_arena = resources.directional_light.borrow_mut();
        let mut mesh_arena = resources.mesh.borrow_mut();
        let mut entity_arena = resources.entity.borrow_mut();

        make_raycasting_scene(
            resources,
            &mut camera_arena,
            camera_aspect_ratio,
            &mut environment_arena,
            &mut ambient_light_arena,
            &mut directional_light_arena,
            &mut mesh_arena,
            &mut entity_arena,
            level_meshes,
            &mut level_mesh_handle,
        )
    }?;

    {
        let mut scenes = scene_context.scenes.borrow_mut();

        scenes.push(scene);
    }

    // Shader context

    let shader_context_rc = Rc::new(RefCell::new(shader_context));

    // Renderer

    let mut renderer = SoftwareRenderer::new(
        shader_context_rc.clone(),
        scene_context.resources.clone(),
        DEFAULT_VERTEX_SHADER,
        DEFAULT_FRAGMENT_SHADER,
        RenderOptions {
            draw_normals_scale: 0.25,
            ..Default::default()
        },
    );

    let framebuffer_rc = Rc::new(RefCell::new(framebuffer));

    renderer.bind_framebuffer(Some(framebuffer_rc.clone()));

    let renderer_rc = RefCell::new(renderer);

    // App update and render callbacks

    let view_camera_handle: &'static RefCell<Option<Handle>> =
        Box::leak(Box::new(RefCell::new(Default::default())));

    let ray_grid_rotation = Quaternion::new(vec3::UP, 0.0);
    let ray_grid_rotation_rc = RefCell::new(ray_grid_rotation);

    let bvh_maximum_visible_node_depth = 0_u8;
    let bvh_maximum_visible_node_depth_rc = RefCell::new(bvh_maximum_visible_node_depth);

    let draw_ray_grid_rc = RefCell::new(false);

    let mut update = |app: &mut App,
                      keyboard_state: &mut KeyboardState,
                      mouse_state: &mut MouseState,
                      game_controller_state: &mut GameControllerState|
     -> Result<(), String> {
        let uptime = app.timing_info.uptime_seconds;

        // Use Shift + Scroll wheel to increment or decrement minimum depth of visible BVH nodes.

        let is_shift_pressed = keyboard_state.pressed_keycodes.contains(&Keycode::LShift)
            || keyboard_state.pressed_keycodes.contains(&Keycode::RShift);

        if let Some(event) = mouse_state.wheel_event.as_ref() {
            let mut maximum_depth = bvh_maximum_visible_node_depth_rc.borrow_mut();

            if is_shift_pressed {
                match event.delta.cmp(&0) {
                    Ordering::Greater => *maximum_depth += event.delta as u8,
                    Ordering::Less => {
                        *maximum_depth -= (event.delta.unsigned_abs() as u8).min(*maximum_depth)
                    }
                    Ordering::Equal => (),
                }
            }
        }

        // Use the 'G' key to toggle rendering of the ray (g)rid.

        if keyboard_state.newly_pressed_keycodes.contains(&Keycode::G) {
            let mut draw_ray_grid = draw_ray_grid_rc.borrow_mut();

            *draw_ray_grid = !*draw_ray_grid;
        }

        let resources = &scene_context.resources;

        let mut shader_context = (*shader_context_rc).borrow_mut();

        let mut scenes = scene_context.scenes.borrow_mut();

        let scene = &mut scenes[0];

        // Traverse the scene graph and update its nodes.

        let update_node_rc = Rc::new(
            |_current_world_transform: &Mat4,
             node: &mut SceneNode,
             _resources: &SceneResources,
             app: &App,
             _mouse_state: &MouseState,
             _keyboard_state: &KeyboardState,
             _game_controller_state: &GameControllerState,
             _shader_context: &mut ShaderContext|
             -> Result<bool, String> {
                let _uptime = app.timing_info.uptime_seconds;

                let (node_type, handle) = (node.get_type(), node.get_handle());

                match node_type {
                    SceneNodeType::Camera => match handle {
                        Some(handle) => {
                            view_camera_handle.borrow_mut().replace(*handle);

                            Ok(false)
                        }
                        None => panic!("Encountered a `Camera` node with no resource handle!"),
                    },
                    _ => Ok(false),
                }
            },
        );

        scene.update(
            resources,
            &mut shader_context,
            app,
            mouse_state,
            keyboard_state,
            game_controller_state,
            Some(update_node_rc),
        )?;

        let theta = (uptime / 10.0).rem_euclid(TAU);

        *ray_grid_rotation_rc.borrow_mut() = Quaternion::new(vec3::UP, theta);

        let mut renderer = renderer_rc.borrow_mut();

        renderer.update(keyboard_state);

        Ok(())
    };

    let render = |_frame_index: Option<u32>,
                  _new_resolution: Option<Resolution>,
                  canvas: &mut [u8]|
     -> Result<(), String> {
        let resources = &scene_context.resources;

        let scenes = scene_context.scenes.borrow();

        let scene = &scenes[0];

        {
            let mut renderer = renderer_rc.borrow_mut();

            renderer.begin_frame();
        }

        // Render scene.

        scene.render(resources, &renderer_rc, None)?;

        {
            let mut renderer = renderer_rc.borrow_mut();

            // Render the nodes of our level mesh's BHV.

            let mesh_arena = resources.mesh.borrow();

            if let Ok(entry) = mesh_arena.get(&level_mesh_handle) {
                let mesh = &entry.item;

                if let Some(collider) = mesh.collider.as_ref() {
                    // Render the BVH root's AABB.

                    let maximum_depth = bvh_maximum_visible_node_depth_rc.borrow();

                    renderer.render_bvh(collider, *maximum_depth);

                    if *draw_ray_grid_rc.borrow() {
                        let grid_rotation = ray_grid_rotation_rc.borrow();

                        render_rotated_ray_grid(&mut renderer, &grid_rotation, mesh);
                    }
                }
            }

            renderer.end_frame();
        }

        // Write out.

        let framebuffer = framebuffer_rc.borrow();

        match framebuffer.attachments.color.as_ref() {
            Some(color_buffer_lock) => {
                let color_buffer = color_buffer_lock.borrow();

                color_buffer.copy_to(canvas);

                Ok(())
            }
            None => panic!(),
        }
    };

    app.run(&mut update, &render)?;

    Ok(())
}

fn render_rotated_ray_grid(
    renderer: &mut SoftwareRenderer,
    grid_rotation: &Quaternion,
    level_mesh: &Mesh,
) {
    // Build a grid of downward-facing rays that begins above the level geometry.

    static COLUMNS: usize = 16;
    static COLUMN_ALPHA_STEP: f32 = 1.0 / COLUMNS as f32;

    static ROWS: usize = 16;
    static ROW_ALPHA_STEP: f32 = 1.0 / ROWS as f32;

    let level_mesh_extent = level_mesh.aabb.extent();

    let grid_size = level_mesh_extent
        .x
        .max(level_mesh_extent.y)
        .max(level_mesh_extent.z);

    let mut rays = ray::make_ray_grid(ROWS, COLUMNS, grid_size);

    for (ray_index, ray) in rays.iter_mut().enumerate() {
        let z = ray_index / COLUMNS;
        let x = ray_index % COLUMNS;

        let z_alpha = COLUMN_ALPHA_STEP * z as f32;
        let x_alpha = ROW_ALPHA_STEP * x as f32;

        let ray_color = Color::rgb((255.0 * z_alpha) as u8, (255.0 * x_alpha) as u8, 0);

        ray.origin.y = level_mesh.aabb.max.y + 1.0;

        ray.origin *= grid_rotation.mat();

        ray.direction = (-vec3::UP + vec3::FORWARD).as_normal();

        ray.one_over_direction = vec3::ONES / ray.direction;

        ray.t = f32::MAX;

        let bvh = level_mesh.collider.as_ref().unwrap();

        let bvh_instance = StaticTriangleBVHInstance::new(bvh, Mat4::identity(), Mat4::identity());

        if false {
            // Brute-force intersections.

            for (triangle_index, triangle) in bvh.tris.iter().enumerate() {
                let (v0, v1, v2) = (
                    &level_mesh.geometry.vertices[triangle.vertices[0]],
                    &level_mesh.geometry.vertices[triangle.vertices[1]],
                    &level_mesh.geometry.vertices[triangle.vertices[2]],
                );

                intersect_ray_triangle(ray, 0, triangle_index, v0, v1, v2);
            }
        } else {
            // Accelerated BVH intersections.

            intersect_ray_bvh(ray, 0, &bvh_instance);
        }

        renderer.render_ray(ray, ray_color);

        if let Some(index) = &ray.colliding_primitive {
            let triangle = &bvh.tris[*index];

            let (v0, v1, v2) = (
                &level_mesh.geometry.vertices[triangle.vertices[0]],
                &level_mesh.geometry.vertices[triangle.vertices[1]],
                &level_mesh.geometry.vertices[triangle.vertices[2]],
            );

            renderer.render_line(*v0, *v1, ray_color);
            renderer.render_line(*v1, *v2, ray_color);
            renderer.render_line(*v0, *v2, ray_color);
        }
    }
}
