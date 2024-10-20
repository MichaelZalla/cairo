use std::{
    cell::RefCell,
    fmt::{self, Display},
    rc::Rc,
};

use serde::{Deserialize, Serialize};

use crate::{
    buffer::{framebuffer::Framebuffer, Buffer2D},
    color,
    render::culling::FaceCullingReject,
    resource::handle::Handle,
    scene::{camera::Camera, context::SceneContext, resources::SceneResources},
    serde::PostDeserialize,
    shader::{context::ShaderContext, geometry::sample::GeometrySample},
    shaders::shadow_shaders::point_shadows::{
        PointShadowMapFragmentShader, PointShadowMapGeometryShader, PointShadowMapVertexShader,
    },
    software_renderer::SoftwareRenderer,
    texture::{cubemap::{CubeMap, Side, CUBE_MAP_SIDES}, map::TextureMap},
    vec::{vec3::Vec3, vec4::Vec4},
};

use super::{attenuation::LightAttenuation, contribute_pbr};

pub static POINT_LIGHT_SHADOW_MAP_SIZE: u32 = 192;
pub static POINT_LIGHT_SHADOW_CAMERA_NEAR: f32 = 0.3;
pub static POINT_LIGHT_SHADOW_CAMERA_FAR: f32 = 50.0;

#[derive(Debug, Clone)]
pub struct ShadowMapRenderingContext {
    pub framebuffer: Rc<RefCell<Framebuffer>>,
    pub shader_context: Rc<RefCell<ShaderContext>>,
    pub renderer: RefCell<SoftwareRenderer>,
}

impl ShadowMapRenderingContext {
    pub fn new(scene_resources: Rc<SceneResources>) -> Self {
        // Shadow map framebuffer.

        let framebuffer = {
            let mut framebuffer =
                Framebuffer::new(POINT_LIGHT_SHADOW_MAP_SIZE, POINT_LIGHT_SHADOW_MAP_SIZE);

            framebuffer.complete(
                POINT_LIGHT_SHADOW_CAMERA_NEAR,
                POINT_LIGHT_SHADOW_CAMERA_FAR,
            );

            Rc::new(RefCell::new(framebuffer))
        };

        // Shadow map shader context.

        let shader_context = Rc::new(RefCell::new(ShaderContext::default()));

        // Shadow map renderer.

        let renderer = {
            let mut renderer = SoftwareRenderer::new(
                shader_context.clone(),
                scene_resources,
                PointShadowMapVertexShader,
                PointShadowMapFragmentShader,
                Default::default(),
            );
    
            renderer.set_geometry_shader(PointShadowMapGeometryShader);
    
            renderer
                .options
                .rasterizer_options
                .face_culling_strategy
                .reject = FaceCullingReject::Frontfaces;
    
            renderer.bind_framebuffer(Some(framebuffer.clone()));

            RefCell::new(renderer)
        };

        // Shadow map rendering context.

        Self {
            renderer,
            shader_context,
            framebuffer,
        }
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct PointLight {
    pub intensities: Vec3,
    pub position: Vec3,
    pub attenuation: LightAttenuation,
    #[serde(skip)]
    pub shadow_map: Option<Handle>,
    #[serde(skip)]
    pub shadow_map_rendering_context: Option<ShadowMapRenderingContext>,
    #[serde(skip)]
    pub influence_distance: f32,
}

impl PostDeserialize for PointLight {
    fn post_deserialize(&mut self) {
        self.influence_distance = self.attenuation.get_approximate_influence_distance();
    }
}

impl Display for PointLight {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "PointLight (intensities={}, position={})",
            self.intensities, self.position
        )
    }
}

impl PointLight {
    pub fn new() -> Self {
        let mut light = PointLight {
            intensities: color::WHITE.to_vec3() / 255.0,
            position: Vec3 {
                x: 0.0,
                y: 10.0,
                z: 0.0,
            },
            attenuation: LightAttenuation::new(1.0, 0.35, 0.44),
            shadow_map: None,
            shadow_map_rendering_context: None,
            influence_distance: 0.0,
        };

        light.post_deserialize();

        light
    }

    pub fn enable_shadow_maps(&mut self, scene_resources: Rc<SceneResources>) {
        let shadow_map_rendering_context = ShadowMapRenderingContext::new(scene_resources.clone());

        let shadow_map_handle = {
            let mut cubemap_f32_arena = scene_resources.cubemap_f32.borrow_mut();

            let shadow_map_framebuffer = shadow_map_rendering_context.framebuffer.borrow();

            cubemap_f32_arena.insert(CubeMap::<f32>::from_framebuffer(&shadow_map_framebuffer))
        };

        self.shadow_map.replace(shadow_map_handle);

        self.shadow_map_rendering_context
            .replace(shadow_map_rendering_context);
    }

    pub fn update_shadow_map(&mut self, scene_context: &SceneContext) -> Result<(), String> {
        // Re-render shadow map for the latest scene.

        let shadow_map_handle = if self.shadow_map.is_none() {
            return Err("Called PointLight::update_shadow_map() on a light with no shadow map handle created!".to_string());
        } else {
            self.shadow_map.as_ref().unwrap()
        };

        {
            let mut cubemap_f32_arena = scene_context.resources.cubemap_f32.borrow_mut();

            if let Ok(entry) = cubemap_f32_arena.get_mut(shadow_map_handle) {
                let map = &mut entry.item;

                self.render_shadow_map_into(map, scene_context)?;
            }
        }

        Ok(())
    }

    pub fn contribute(self, sample: &GeometrySample) -> Vec3 {
        let mut point_contribution: Vec3 = Vec3::new();
        let mut specular_contribution: Vec3 = Vec3::new();

        let tangent_space_info = sample.tangent_space_info;

        let normal = &tangent_space_info.normal;

        let point_light_position_tangent_space =
            (Vec4::new(self.position, 1.0) * tangent_space_info.tbn_inverse).to_vec3();

        let fragment_to_point_light_tangent_space =
            point_light_position_tangent_space - tangent_space_info.fragment_position;

        let distance_to_point_light_tangent_space = fragment_to_point_light_tangent_space.mag();

        let direction_to_point_light_tangent_space =
            fragment_to_point_light_tangent_space / distance_to_point_light_tangent_space;

        let likeness = 0.0_f32.max(normal.dot(direction_to_point_light_tangent_space));

        if likeness > 0.0 {
            let attenuation = self
                .attenuation
                .attenuate_for_distance(distance_to_point_light_tangent_space);

            point_contribution = self.intensities * attenuation * 0.0_f32.max(likeness);

            let reflected_ray = {
                // Calculate specular light intensity
                let incoming_ray = fragment_to_point_light_tangent_space * -1.0;

                // Project the incoming ray forward through the fragment/surface
                let absorbed_ray = tangent_space_info.fragment_position + incoming_ray;

                // Project the incoming light ray onto the surface normal (i.e.,
                // scaling the normal up or down)
                let w = *normal * incoming_ray.dot(*normal);

                // Combine the absorbed ray with the scaled normal to find the
                // reflected ray vector.
                let u = w * 2.0;

                u - absorbed_ray
            };

            // Get the reflected ray's direction from the surface
            let reflected_ray_normal = reflected_ray.as_normal();

            // Compute the similarity between the reflected ray's direction and
            // the direction from our fragment to the viewer.
            let fragment_to_view_tangent_space =
                tangent_space_info.view_position - tangent_space_info.fragment_position;

            let view_direction_normal = fragment_to_view_tangent_space.as_normal();

            let cosine_theta = 1.0_f32.min(reflected_ray_normal.dot(view_direction_normal * -1.0));

            let similarity = 0.0_f32.max(cosine_theta);

            specular_contribution = point_contribution
                * sample.specular_color
                * similarity.powi(sample.specular_exponent as i32);
        }

        point_contribution + specular_contribution
    }

    pub fn contribute_pbr(
        &self,
        sample: &GeometrySample,
        f0: &Vec3,
        shadow_map: Option<&CubeMap<f32>>,
    ) -> Vec3 {
        let tangent_space_info = sample.tangent_space_info;

        let point_light_position =
            (Vec4::new(self.position, 1.0) * tangent_space_info.tbn_inverse).to_vec3();

        let fragment_to_point_light = point_light_position - tangent_space_info.fragment_position;
        let distance_to_point_light = fragment_to_point_light.mag();
        let direction_to_point_light = fragment_to_point_light / distance_to_point_light;

        // Compute an enshadowing term for this fragment/sample.

        let in_shadow = if let Some(map) = shadow_map {
            self.get_shadowing(sample, map)
        } else {
            0.0
        };

        let contribution = contribute_pbr(sample, &self.intensities, &direction_to_point_light, f0)
            * self
                .attenuation
                .attenuate_for_distance(distance_to_point_light);

        contribution * (1.0 - in_shadow)
    }

    fn get_shadowing(&self, sample: &GeometrySample, shadow_map: &CubeMap<f32>) -> f32 {
        let light_to_fragment = sample.world_pos - self.position;
        let light_to_fragment_direction = light_to_fragment.as_normal();

        let current_depth = light_to_fragment.mag();

        let closest_depth = shadow_map.sample_nearest(&Vec4::new(light_to_fragment_direction, 1.0))
            * POINT_LIGHT_SHADOW_CAMERA_FAR;

        if closest_depth == 0.0 {
            return 0.0;
        }

        let likeness = sample
            .world_normal
            .dot((self.position - sample.world_pos).as_normal());

        let bias = 0.005_f32.max(0.05 * (1.0 - likeness));

        let is_in_shadow = (current_depth + bias) > closest_depth;

        if is_in_shadow {
            1.0
        } else {
            0.0
        }
    }

    fn render_shadow_map_into(
        &self,
        shadow_map: &mut CubeMap<f32>,
        scene_context: &SceneContext,
    ) -> Result<(), String> {
        let context = if self.shadow_map_rendering_context.is_none() {
            return Err("Called PointLight::render_shadow_map() on a light with no shadow map rendering context created!".to_string());
        } else {
            self.shadow_map_rendering_context.as_ref().unwrap()
        };

        let mut cubemap_face_camera = {
            let mut camera = Camera::from_perspective(self.position, Default::default(), 90.0, 1.0);
    
            // @NOTE(mzalla) Assumes the same near and far is set for the
            // framebuffer's depth attachment.
    
            camera.set_projection_z_near(POINT_LIGHT_SHADOW_CAMERA_NEAR);
            camera.set_projection_z_far(POINT_LIGHT_SHADOW_CAMERA_FAR);
    
            camera
        };
    
        {
            let mut shader_context = context.shader_context.borrow_mut();
    
            cubemap_face_camera.update_shader_context(&mut shader_context);
        }
    
        for side in CUBE_MAP_SIDES {
            if let Side::Up = &side {
                continue;
            }
    
            cubemap_face_camera
                .look_vector
                .set_target_position(self.position + side.get_direction());
    
            {
                let mut shader_context = context.shader_context.borrow_mut();
    
                shader_context
                    .set_view_inverse_transform(cubemap_face_camera.get_view_inverse_transform());
            }
    
            let resources = &scene_context.resources;
            let scenes = scene_context.scenes.borrow();
    
            let scene = &scenes[0];
    
            match scene.render(resources, &context.renderer, None) {
                Ok(()) => {
                    // Blit our framebuffer's HDR attachment buffer to our
                    // cubemap's corresponding side (texture map).
    
                    let framebuffer = context.framebuffer.borrow();
    
                    match &framebuffer.attachments.forward_or_deferred_hdr {
                        Some(hdr_attachment_rc) => {
                            let hdr_attachment = hdr_attachment_rc.borrow();
    
                            blit_hdr_attachment_to_cubemap_side(&hdr_attachment, &mut shadow_map.sides[side.get_index()]);
                        }
                        None => return Err("Called CubeMap::<f32>::render_scene() with a Framebuffer with no HDR attachment!".to_string()),
                    }
                }
                Err(e) => panic!("{}", e),
            }
        }
    
        Ok(())
    }
    
}

fn blit_hdr_attachment_to_cubemap_side(
    hdr_buffer: &Buffer2D<Vec3>,
    cubemap_side: &mut TextureMap<f32>,
) {
    let buffer = &mut cubemap_side.levels[0].0;

    for y in 0..buffer.height {
        for x in 0..buffer.width {
            buffer.set(x, y, hdr_buffer.get(x, y).x);
        }
    }
}
