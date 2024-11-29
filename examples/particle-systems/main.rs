extern crate sdl2;

use std::{cell::RefCell, rc::Rc};

use cairo::{
    app::{
        resolution::{self, Resolution},
        App, AppWindowInfo,
    },
    buffer::framebuffer::Framebuffer,
    color,
    device::{game_controller::GameControllerState, keyboard::KeyboardState, mouse::MouseState},
    geometry::accelerator::{
        static_triangle_bvh::StaticTriangleBVHInstance, static_triangle_tlas::StaticTriangleTLAS,
    },
    material::Material,
    matrix::Mat4,
    mesh::{primitive::cube, Mesh},
    physics::simulation::particle::generator::ParticleGeneratorKind,
    random::sampler::RandomSampler,
    render::{
        culling::{FaceCullingReject, FaceCullingStrategy},
        options::{rasterizer::RasterizerOptions, RenderOptions},
        Renderer,
    },
    resource::handle::Handle,
    scene::{
        context::SceneContext,
        node::{SceneNode, SceneNodeGlobalTraversalMethod, SceneNodeType},
        resources::SceneResources,
    },
    shaders::{
        default_fragment_shader::DEFAULT_FRAGMENT_SHADER,
        default_vertex_shader::DEFAULT_VERTEX_SHADER,
    },
    software_renderer::SoftwareRenderer,
    transform::Transform3D,
    vec::vec3,
};

use make_simulation::make_simulation;
use scene::make_particles_scene;
use simulation::Simulation;

pub mod integrate;
pub mod intersect;
pub mod make_simulation;
pub mod scene;
pub mod simulation;

pub const SAMPLER_SEED_SIZE: usize = 2048;

static PARTICLE_MESH_SCALE: f32 = 0.25;
static PARTICLE_MESH_SCALE_OVER_2: f32 = PARTICLE_MESH_SCALE / 2.0;

fn main() -> Result<(), String> {
    let mut window_info = AppWindowInfo {
        title: "examples/particle-systems".to_string(),
        window_resolution: resolution::RESOLUTION_640_BY_320 * 2.0,
        canvas_resolution: resolution::RESOLUTION_640_BY_320,
        relative_mouse_mode: true,
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

    framebuffer.complete(0.3, 500.0);

    let camera_aspect_ratio = framebuffer.width_over_height;

    // Scene context

    let scene_context = SceneContext::default();

    // Scene

    let (scene, shader_context) = {
        let resources = &scene_context.resources;

        let mut camera_arena = resources.camera.borrow_mut();
        let mut environment_arena = resources.environment.borrow_mut();
        let mut ambient_light_arena = resources.ambient_light.borrow_mut();
        let mut directional_light_arena = resources.directional_light.borrow_mut();
        let mut spot_light_arena = resources.spot_light.borrow_mut();

        let mut mesh_arena = resources.mesh.borrow_mut();
        let mut entity_arena = resources.entity.borrow_mut();
        let mut material_arena = resources.material.borrow_mut();

        make_particles_scene(
            resources,
            &mut camera_arena,
            camera_aspect_ratio,
            &mut environment_arena,
            &mut ambient_light_arena,
            &mut directional_light_arena,
            &mut spot_light_arena,
            &mut mesh_arena,
            &mut material_arena,
            &mut entity_arena,
        )
    }?;

    {
        let mut scenes = scene_context.scenes.borrow_mut();

        scenes.push(scene);
    }

    // Particle mesh handle.

    let particle_mesh_handle = {
        let mut mesh_arena = scene_context.resources.mesh.borrow_mut();

        let particle_mesh = cube::generate(
            PARTICLE_MESH_SCALE,
            PARTICLE_MESH_SCALE,
            PARTICLE_MESH_SCALE,
        );

        mesh_arena.insert(particle_mesh)
    };

    // Particle material handle.

    let particle_material_handle = {
        let mut material_arena = scene_context.resources.material.borrow_mut();

        let particle_material = Material {
            albedo: color::RED.to_vec3() / 255.0,
            ..Default::default()
        };

        material_arena.insert(particle_material)
    };

    // Shader context

    let shader_context_rc = Rc::new(RefCell::new(shader_context));

    // Renderer

    let mut renderer = SoftwareRenderer::new(
        shader_context_rc.clone(),
        scene_context.resources.clone(),
        DEFAULT_VERTEX_SHADER,
        DEFAULT_FRAGMENT_SHADER,
        RenderOptions {
            rasterizer_options: RasterizerOptions {
                face_culling_strategy: FaceCullingStrategy {
                    reject: FaceCullingReject::None,
                    ..Default::default()
                },
            },
            ..Default::default()
        },
    );

    let framebuffer_rc = Rc::new(RefCell::new(framebuffer));

    renderer.bind_framebuffer(Some(framebuffer_rc.clone()));

    let renderer_rc = RefCell::new(renderer);

    // Set up our particle simulation.

    let sampler_rc = {
        let mut sampler: RandomSampler<SAMPLER_SEED_SIZE> = Default::default();

        // Seed the simulation's random number sampler.

        sampler.seed().unwrap();

        RefCell::new(sampler)
    };

    let simulation = make_simulation(sampler_rc);

    // App update and render callbacks

    let bvh_instances_rc = RefCell::new(vec![]);

    let static_triangle_tlas_rc: RefCell<Option<StaticTriangleTLAS>> = Default::default();

    let mut update = |app: &mut App,
                      keyboard_state: &mut KeyboardState,
                      mouse_state: &mut MouseState,
                      game_controller_state: &mut GameControllerState|
     -> Result<(), String> {
        // Advance our particle simulation by delta time.

        let h = app.timing_info.seconds_since_last_update;

        let uptime = app.timing_info.uptime_seconds;

        let resources = &scene_context.resources;

        // Recompile a list of BVH instances with their associated transforms.

        {
            let mut bvh_instances = bvh_instances_rc.borrow_mut();

            bvh_instances.clear();
        }

        let mut scenes = scene_context.scenes.borrow_mut();

        let scene = &mut scenes[0];

        scene.root.visit_mut(
            SceneNodeGlobalTraversalMethod::DepthFirst,
            None,
            &mut |_current_depth: usize, _current_world_transform: Mat4, node: &mut SceneNode| {
                let (node_type, handle) = (*node.get_type(), *node.get_handle());

                match node_type {
                    SceneNodeType::Entity => {
                        let node_transform = node.get_transform();

                        let entity_arena = resources.entity.borrow();

                        let entity_handle = handle.unwrap();

                        if let Ok(entry) = entity_arena.get(&entity_handle) {
                            let entity = &entry.item;

                            let mut mesh_arena = resources.mesh.borrow_mut();

                            if let Ok(entry) = mesh_arena.get_mut(&entity.mesh) {
                                let mesh = &mut entry.item;

                                // Update transforms for all mesh BVHs, for all scene entities.

                                if let Some(bvh) = mesh.collider.as_ref() {
                                    let mut bvh_instances = bvh_instances_rc.borrow_mut();

                                    let transform = *node_transform.mat();
                                    let inverse_transform = *node_transform.inverse_mat();

                                    let new_instance = StaticTriangleBVHInstance::new(
                                        bvh,
                                        transform,
                                        inverse_transform,
                                    );

                                    bvh_instances.push(new_instance);
                                }
                            }
                        }

                        Ok(())
                    }
                    _ => Ok(()),
                }
            },
        )?;

        // Rebuild the top-level acceleration structure (TLAS) to organize the instances.

        {
            let bvh_instances = bvh_instances_rc.borrow();

            let mut static_triangle_tlas = static_triangle_tlas_rc.borrow_mut();

            let tlas = StaticTriangleTLAS::new(bvh_instances.clone());

            if h > 0.0 {
                simulation.tick(h, uptime, &tlas)?;
            }

            static_triangle_tlas.replace(tlas);
        }

        // Traverse the scene graph and update its nodes.

        let mut shader_context = (*shader_context_rc).borrow_mut();

        scene.update(
            resources,
            &mut shader_context,
            app,
            mouse_state,
            keyboard_state,
            game_controller_state,
            None,
        )?;

        let mut renderer = renderer_rc.borrow_mut();

        renderer.options.update(keyboard_state);

        renderer.shader_options.update(keyboard_state);

        let camera_handle = scene
            .root
            .find(|node| *node.get_type() == SceneNodeType::Camera)
            .unwrap()
            .unwrap();

        let camera_arena = resources.camera.borrow();

        if let Ok(entry) = camera_arena.get(&camera_handle) {
            let camera = &entry.item;

            renderer.set_clipping_frustum(*camera.get_frustum());
        }

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
            let mesh_arena = resources.mesh.borrow();

            let mut renderer = renderer_rc.borrow_mut();

            renderer.begin_frame();

            renderer.render_ground_plane(30);

            if let Ok(entry) = mesh_arena.get(&particle_mesh_handle) {
                let particle_mesh = &entry.item;

                let optional_tlas = static_triangle_tlas_rc.borrow();

                let tlas = optional_tlas.as_ref().unwrap();

                draw_simulation(
                    &simulation,
                    tlas,
                    resources,
                    particle_mesh,
                    particle_material_handle,
                    &mut renderer,
                );
            }
        }

        {
            // Render scene.

            scene.render(resources, &renderer_rc, None)?;
        }

        {
            let mut renderer = renderer_rc.borrow_mut();

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

fn draw_tlas_node(tlas: &StaticTriangleTLAS, node_index: usize, renderer: &mut SoftwareRenderer) {
    let node = &tlas.nodes[node_index];

    if node.is_leaf() {
        let bvh_instance = &tlas.bvh_instances[node.bvh_instance_index as usize];

        renderer.render_aabb(&bvh_instance.world_aabb, None, color::ORANGE);

        return;
    }

    renderer.render_aabb(&node.aabb, None, color::YELLOW);

    renderer.render_line(
        node.aabb.center(),
        tlas.nodes[node.left_child_index as usize].aabb.center(),
        color::DARK_GRAY,
    );

    draw_tlas_node(tlas, node.left_child_index as usize, renderer);

    if node.right_child_index > 0 {
        renderer.render_line(
            node.aabb.center(),
            tlas.nodes[node.right_child_index as usize].aabb.center(),
            color::DARK_GRAY,
        );

        draw_tlas_node(tlas, node.right_child_index as usize, renderer);
    }
}

fn draw_simulation(
    simulation: &Simulation<SAMPLER_SEED_SIZE>,
    tlas: &StaticTriangleTLAS,
    _resources: &SceneResources,
    particle_mesh: &Mesh,
    particle_material_handle: Handle,
    renderer: &mut SoftwareRenderer,
) {
    static DRAW_TLAS_NODES: bool = false;

    if DRAW_TLAS_NODES {
        draw_tlas_node(tlas, 0, renderer);
    }

    let pool = simulation.pool.borrow();
    let generators = simulation.generators.borrow();

    for generator in generators.iter() {
        match generator.kind {
            ParticleGeneratorKind::Omnidirectional(origin) => {
                renderer.render_axes(Some(origin), None);
            }
            ParticleGeneratorKind::Directed(origin, _direction) => {
                renderer.render_axes(Some(origin), None);
            }
        }
    }

    for particle in pool.iter() {
        if particle.alive {
            // Render particle

            let mut transform = Transform3D::default();

            transform.set_translation(particle.position + vec3::UP * PARTICLE_MESH_SCALE_OVER_2);

            renderer.render_point(
                transform.mat(),
                None,
                Some(particle_mesh),
                Some(particle_material_handle),
            );
        }
    }
}
