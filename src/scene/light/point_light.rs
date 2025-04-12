use std::{
    fmt::{self, Display},
    rc::Rc,
};

use serde::{Deserialize, Serialize};

use crate::{
    render::{culling::FaceCullingReject, Renderer},
    resource::handle::Handle,
    scene::{
        camera::Camera,
        graph::{options::SceneGraphRenderOptions, SceneGraph},
        resources::SceneResources,
    },
    serde::PostDeserialize,
    shader::geometry::sample::GeometrySample,
    shaders::shadow_shaders::point_shadows::{
        PointShadowMapFragmentShader, PointShadowMapGeometryShader, PointShadowMapVertexShader,
    },
    texture::cubemap::{CubeMap, CUBE_MAP_SIDES},
    vec::{
        vec3::{self, Vec3},
        vec4::Vec4,
    },
};

use super::{
    attenuation::{LightAttenuation, LIGHT_ATTENUATION_RANGE_13_UNITS},
    contribute_pbr_tangent_space,
    shadow::{ShadowMapRenderingContext, SHADOW_MAP_CAMERA_NEAR},
};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct PointLight {
    pub intensities: Vec3,
    pub position: Vec3,
    attenuation: LightAttenuation,
    #[serde(skip)]
    pub shadow_map: Option<Handle>,
    #[serde(skip)]
    pub shadow_map_rendering_context: Option<ShadowMapRenderingContext>,
    #[serde(skip)]
    pub influence_distance: f32,
}

impl PostDeserialize for PointLight {
    fn post_deserialize(&mut self) {
        self.recompute_influence_distance();
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
            intensities: vec3::ONES,
            position: Vec3 {
                x: 0.0,
                y: 10.0,
                z: 0.0,
            },
            attenuation: LIGHT_ATTENUATION_RANGE_13_UNITS,
            shadow_map: None,
            shadow_map_rendering_context: None,
            influence_distance: 0.0,
        };

        light.post_deserialize();

        light
    }

    pub fn get_attenuation(&self) -> &LightAttenuation {
        &self.attenuation
    }

    pub fn set_attenuation(&mut self, attenuation: LightAttenuation) {
        self.attenuation = attenuation;

        self.recompute_influence_distance();
    }

    fn recompute_influence_distance(&mut self) {
        self.influence_distance = self.attenuation.get_approximate_influence_distance();
    }

    pub fn enable_shadow_maps(
        &mut self,
        shadow_map_size: u32,
        projection_z_far: f32,
        scene_resources: Rc<SceneResources>,
    ) {
        let shadow_map_rendering_context = ShadowMapRenderingContext::new(
            shadow_map_size,
            projection_z_far,
            FaceCullingReject::Frontfaces,
            PointShadowMapVertexShader,
            PointShadowMapGeometryShader,
            PointShadowMapFragmentShader,
            scene_resources.clone(),
        );

        let shadow_map_handle = {
            let mut cubemap_f32_arena = scene_resources.cubemap_f32.borrow_mut();

            let shadow_map_framebuffer = shadow_map_rendering_context.framebuffer.borrow();

            cubemap_f32_arena.insert(CubeMap::<f32>::from_framebuffer(&shadow_map_framebuffer))
        };

        self.shadow_map.replace(shadow_map_handle);

        self.shadow_map_rendering_context
            .replace(shadow_map_rendering_context);
    }

    pub fn update_shadow_map(
        &mut self,
        resources: &SceneResources,
        scene: &SceneGraph,
    ) -> Result<(), String> {
        // Re-render shadow map based on the scene's current state.

        let shadow_map_handle = if self.shadow_map.is_none() {
            return Err(
                "Called PointLight::update_shadow_map() on a light with no shadow map handle!"
                    .to_string(),
            );
        } else {
            self.shadow_map.as_ref().unwrap()
        };

        let rendering_context = if self.shadow_map_rendering_context.is_none() {
            return Err("Called PointLight::update_shadow_map() on a light with no shadow map rendering context!".to_string());
        } else {
            self.shadow_map_rendering_context.as_ref().unwrap()
        };

        {
            let mut shader_context = rendering_context.shader_context.borrow_mut();

            shader_context
                .projection_z_far
                .replace(rendering_context.projection_z_far);
        }

        {
            let mut cubemap_f32_arena = resources.cubemap_f32.borrow_mut();

            if let Ok(entry) = cubemap_f32_arena.get_mut(shadow_map_handle) {
                let map = &mut entry.item;

                self.render_shadow_map_into(map, rendering_context, resources, scene)?;
            }
        }

        Ok(())
    }

    pub fn contribute(self, sample: &GeometrySample) -> Vec3 {
        let mut point_contribution = Vec3::default();
        let mut specular_contribution = Vec3::default();

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

        let direction_to_light_tangent_space = fragment_to_point_light / distance_to_point_light;

        let light_intensities = &self.intensities;

        let contribution = contribute_pbr_tangent_space(
            sample,
            light_intensities,
            &direction_to_light_tangent_space,
            f0,
        );

        let attenuation = self
            .attenuation
            .attenuate_for_distance(distance_to_point_light);

        // Compute an enshadowing term for this fragment/sample.

        let in_shadow = if let Some(map) = shadow_map {
            self.get_shadowing(sample, map)
        } else {
            0.0
        };

        contribution * attenuation * (1.0 - in_shadow)
    }

    fn pcf_3x3(
        &self,
        near: f32,
        far: f32,
        current_depth: f32,
        sample: &GeometrySample,
        map: &CubeMap<f32>,
        light_to_fragment_direction: Vec3,
    ) -> f32 {
        let mut accumulated_shadow = 0.0;

        static SAMPLES: f32 = 3.0;
        static SAMPLES_OVER_2: f32 = SAMPLES / 2.0;

        static OFFSET: f32 = 0.01;

        static STEP_SIZE: f32 = OFFSET / SAMPLES_OVER_2;

        static STEPS: usize = (OFFSET * 2.0 / STEP_SIZE) as usize;

        for i_x in 0..(STEPS + 1_usize) {
            let x = -OFFSET + STEP_SIZE * i_x as f32;

            for i_y in 0..(STEPS + 1_usize) {
                let y = -OFFSET + STEP_SIZE * i_y as f32;

                for i_z in 0..(STEPS + 1_usize) {
                    let z = -OFFSET + STEP_SIZE * i_z as f32;

                    let perturbed_light_to_fragment_direction =
                        light_to_fragment_direction + Vec3 { x, y, z };

                    let closest_depth_sample =
                        map.sample_nearest(&Vec4::new(perturbed_light_to_fragment_direction, 1.0));

                    let closest_depth = near + closest_depth_sample * (far - near);

                    if closest_depth == 0.0 {
                        continue;
                    }

                    let likeness = sample
                        .normal_world_space
                        .dot((self.position - sample.position_world_space).as_normal());

                    let bias = 0.005_f32.max(0.05 * (1.0 - likeness));

                    if current_depth + bias > closest_depth {
                        accumulated_shadow += 1.0;
                    }
                }
            }
        }

        accumulated_shadow / (STEPS as f32 * 3.0)
    }

    fn pcf_disk(
        &self,
        near: f32,
        far: f32,
        current_depth: f32,
        sample: &GeometrySample,
        map: &CubeMap<f32>,
        light_to_fragment_direction: Vec3,
    ) -> f32 {
        static SAMPLE_OFFSET_DIRECTIONS: [Vec3; 20] = [
            Vec3 {
                x: 1.0,
                y: 1.0,
                z: 1.0,
            },
            Vec3 {
                x: 1.0,
                y: -1.0,
                z: 1.0,
            },
            Vec3 {
                x: -1.0,
                y: -1.0,
                z: 1.0,
            },
            Vec3 {
                x: -1.0,
                y: 1.0,
                z: 1.0,
            },
            Vec3 {
                x: 1.0,
                y: 1.0,
                z: -1.0,
            },
            Vec3 {
                x: 1.0,
                y: -1.0,
                z: -1.0,
            },
            Vec3 {
                x: -1.0,
                y: -1.0,
                z: -1.0,
            },
            Vec3 {
                x: -1.0,
                y: 1.0,
                z: -1.0,
            },
            Vec3 {
                x: 1.0,
                y: 1.0,
                z: 0.0,
            },
            Vec3 {
                x: 1.0,
                y: -1.0,
                z: 0.0,
            },
            Vec3 {
                x: -1.0,
                y: -1.0,
                z: 0.0,
            },
            Vec3 {
                x: -1.0,
                y: 1.0,
                z: 0.0,
            },
            Vec3 {
                x: 1.0,
                y: 0.0,
                z: 1.0,
            },
            Vec3 {
                x: -1.0,
                y: 0.0,
                z: 1.0,
            },
            Vec3 {
                x: 1.0,
                y: 0.0,
                z: -1.0,
            },
            Vec3 {
                x: -1.0,
                y: 0.0,
                z: -1.0,
            },
            Vec3 {
                x: 0.0,
                y: 1.0,
                z: 1.0,
            },
            Vec3 {
                x: 0.0,
                y: -1.0,
                z: 1.0,
            },
            Vec3 {
                x: 0.0,
                y: -1.0,
                z: -1.0,
            },
            Vec3 {
                x: 0.0,
                y: 1.0,
                z: -1.0,
            },
        ];

        static DISK_RADIUS: f32 = 0.01;

        let mut accumulated_shadow = 0.0;

        for sample_offset in SAMPLE_OFFSET_DIRECTIONS {
            let offset = sample_offset * DISK_RADIUS;

            let perturbed_light_to_fragment_direction = light_to_fragment_direction + offset;

            let closest_depth_sample =
                map.sample_nearest(&Vec4::new(perturbed_light_to_fragment_direction, 1.0));

            let closest_depth = near + closest_depth_sample * (far - near);

            if closest_depth == 0.0 {
                continue;
            }

            let likeness = sample
                .normal_world_space
                .dot((self.position - sample.position_world_space).as_normal());

            let bias = 0.005_f32.max(0.05 * (1.0 - likeness));

            if current_depth + bias > closest_depth {
                accumulated_shadow += 1.0;
            }
        }

        accumulated_shadow / SAMPLE_OFFSET_DIRECTIONS.len() as f32
    }

    fn get_shadowing(&self, sample: &GeometrySample, map: &CubeMap<f32>) -> f32 {
        let context = self.shadow_map_rendering_context.as_ref().unwrap();

        let (near, far) = (SHADOW_MAP_CAMERA_NEAR, context.projection_z_far);

        let light_to_fragment = sample.position_world_space - self.position;
        let light_to_fragment_direction = light_to_fragment.as_normal();

        let current_depth = light_to_fragment.mag();

        self.pcf_disk(
            near,
            far,
            current_depth,
            sample,
            map,
            light_to_fragment_direction,
        )
    }

    fn render_shadow_map_into(
        &self,
        shadow_map: &mut CubeMap<f32>,
        rendering_context: &ShadowMapRenderingContext,
        resources: &SceneResources,
        scene: &SceneGraph,
    ) -> Result<(), String> {
        let mut cubemap_face_camera = {
            let mut camera = Camera::from_perspective(self.position, Default::default(), 90.0, 1.0);

            // @NOTE(mzalla) Assumes the same near and far is set for the
            // framebuffer's depth attachment.

            camera.set_projection_z_near(SHADOW_MAP_CAMERA_NEAR);
            camera.set_projection_z_far(rendering_context.projection_z_far);

            camera
        };

        {
            let mut shader_context = rendering_context.shader_context.borrow_mut();

            cubemap_face_camera.update_shader_context(&mut shader_context);
        }

        for side in CUBE_MAP_SIDES {
            let side_index = side.get_index();

            cubemap_face_camera
                .look_vector
                .set_target(self.position + side.get_direction());

            {
                let mut shader_context = rendering_context.shader_context.borrow_mut();

                // Rebinds view-inverse transform after setting new look-vector target.

                shader_context
                    .set_view_inverse_transform(cubemap_face_camera.get_view_inverse_transform());
            }

            self.render_shadow_map_side(
                side_index,
                cubemap_face_camera,
                shadow_map,
                rendering_context,
                resources,
                scene,
            )?;
        }

        Ok(())
    }

    fn render_shadow_map_side(
        &self,
        side_index: usize,
        camera: Camera,
        shadow_map: &mut CubeMap<f32>,
        rendering_context: &ShadowMapRenderingContext,
        resources: &SceneResources,
        scene: &SceneGraph,
    ) -> Result<(), String> {
        {
            let mut renderer = rendering_context.renderer.borrow_mut();

            renderer.set_clipping_frustum(*camera.get_frustum());

            renderer.begin_frame();
        }

        // Render scene.

        scene.render(
            resources,
            &rendering_context.renderer,
            Some(SceneGraphRenderOptions {
                is_shadow_map_render: true,
                ..Default::default()
            }),
        )?;

        {
            let mut renderer = rendering_context.renderer.borrow_mut();

            renderer.end_frame();
        }

        // Blit our framebuffer's HDR attachment buffer to our cubemap's
        // corresponding side (texture map).

        let framebuffer = rendering_context.framebuffer.borrow();

        match &framebuffer.attachments.deferred_hdr {
            Some(hdr_attachment_rc) => {
                let hdr_buffer = hdr_attachment_rc.borrow();

                let cubemap_side = &mut shadow_map.sides[side_index];

                let target = &mut cubemap_side.levels[0].0;

                for (index, hdr_color) in hdr_buffer.iter().enumerate() {
                    target.set_at(index, hdr_color.x);
                }
            }
            None => return Err(
                "Called PointLight::update_shadow_maps() with no HDR attachment on the rendering context's framebuffer!"
                    .to_string(),
            ),
        }

        Ok(())
    }
}
