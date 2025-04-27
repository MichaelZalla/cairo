use std::{
    f32::consts::PI,
    fmt::{self, Display},
    rc::Rc,
};

use serde::{Deserialize, Serialize};

use crate::{
    buffer::Buffer2D,
    color::Color,
    matrix::Mat4,
    render::{culling::FaceCullingReject, Renderer},
    resource::handle::Handle,
    scene::{
        camera::Camera,
        graph::{options::SceneGraphRenderOptions, SceneGraph},
        resources::SceneResources,
    },
    serde::PostDeserialize,
    shader::geometry::sample::GeometrySample,
    shaders::shadow_shaders::perspective_shadows::{
        PerspectiveShadowMapFragmentShader, PerspectiveShadowMapGeometryShader,
        PerspectiveShadowMapVertexShader,
    },
    texture::{
        map::TextureMap,
        sample::{sample_nearest_f32, sample_nearest_u8},
    },
    transform::look_vector::LookVector,
    vec::{
        vec2::Vec2,
        vec3::{self, Vec3},
        vec4::Vec4,
    },
};

use super::{
    attenuation::{LightAttenuation, LIGHT_ATTENUATION_RANGE_50_UNITS},
    contribute_pbr_world_space,
    shadow::{ShadowMapRenderingContext, SHADOW_MAP_CAMERA_NEAR},
};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct SpotLight {
    pub intensities: Vec3,
    pub look_vector: LookVector,
    inner_cutoff_angle: f32,
    #[serde(skip)]
    inner_cutoff_angle_cos: f32,
    outer_cutoff_angle: f32,
    #[serde(skip)]
    outer_cutoff_angle_cos: f32,
    #[serde(skip)]
    epsilon: f32,
    attenuation: LightAttenuation,
    pub projector_map: Option<TextureMap>,
    #[serde(skip)]
    pub shadow_map: Option<Handle>,
    #[serde(skip)]
    pub shadow_map_rendering_context: Option<ShadowMapRenderingContext>,
    #[serde(skip)]
    pub shadow_map_camera: Option<Camera>,
    #[serde(skip)]
    pub world_to_shadow_map_camera_projection: Option<Mat4>,
    #[serde(skip)]
    pub influence_distance: f32,
}

impl PostDeserialize for SpotLight {
    fn post_deserialize(&mut self) {
        self.recompute_influence_distance();
    }
}

impl Display for SpotLight {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "SpotLight (intensities={}, look_vector={})",
            self.intensities, self.look_vector
        )
    }
}

impl SpotLight {
    pub fn new() -> Self {
        let mut light = SpotLight {
            intensities: vec3::ONES,
            look_vector: LookVector::new(Vec3 {
                x: 0.0,
                y: 10.0,
                z: 0.0,
            }),
            attenuation: LIGHT_ATTENUATION_RANGE_50_UNITS,
            ..Default::default()
        };

        light.set_inner_cutoff_angle(PI / 12.0);
        light.set_outer_cutoff_angle(PI / 8.0);

        light.look_vector.set_target(-vec3::UP);

        light.post_deserialize();

        light
    }

    pub fn get_inner_cutoff_angle(&self) -> f32 {
        self.inner_cutoff_angle
    }

    pub fn get_inner_cutoff_angle_cos(&self) -> f32 {
        self.inner_cutoff_angle_cos
    }

    pub fn set_inner_cutoff_angle(&mut self, angle: f32) {
        self.inner_cutoff_angle = angle;

        self.inner_cutoff_angle_cos = angle.cos();

        self.epsilon = self.inner_cutoff_angle_cos - self.outer_cutoff_angle_cos;
    }

    pub fn get_outer_cutoff_angle(&self) -> f32 {
        self.outer_cutoff_angle
    }

    pub fn get_outer_cutoff_angle_cos(&self) -> f32 {
        self.outer_cutoff_angle_cos
    }

    pub fn set_outer_cutoff_angle(&mut self, angle: f32) {
        self.outer_cutoff_angle = angle;

        self.outer_cutoff_angle_cos = angle.cos();

        self.epsilon = self.inner_cutoff_angle_cos - self.outer_cutoff_angle_cos;
    }

    pub fn get_attenuation(&self) -> &LightAttenuation {
        &self.attenuation
    }

    pub fn set_attenuation(&mut self, attenuation: LightAttenuation) {
        self.attenuation = attenuation;

        self.recompute_influence_distance();
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
            PerspectiveShadowMapVertexShader,
            PerspectiveShadowMapGeometryShader,
            PerspectiveShadowMapFragmentShader,
            scene_resources.clone(),
        );

        let shadow_map_handle = {
            let mut texture_f32_arena = scene_resources.texture_f32.borrow_mut();

            let shadow_map_framebuffer = shadow_map_rendering_context.framebuffer.borrow();

            let shadow_map_size = shadow_map_framebuffer.width;

            texture_f32_arena.insert(TextureMap::<f32>::from_buffer(
                shadow_map_size,
                shadow_map_size,
                Buffer2D::<f32>::new(shadow_map_size, shadow_map_size, None),
            ))
        };

        self.shadow_map.replace(shadow_map_handle);

        self.shadow_map_rendering_context
            .replace(shadow_map_rendering_context);

        self.update_shadow_map_camera();
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
            let mut texture_f32_arena = resources.texture_f32.borrow_mut();

            if let Ok(entry) = texture_f32_arena.get_mut(shadow_map_handle) {
                let map = &mut entry.item;

                self.render_shadow_map_into(map, rendering_context, resources, scene)?;
            }
        }

        Ok(())
    }

    fn get_shadow_map_uv(
        &self,
        world_to_shadow_map_camera_projection: &Mat4,
        sample: &GeometrySample,
    ) -> Option<Vec2> {
        // Project the sample's world space position into the shadow map camera's NDC space.

        let position_shadow_camera_projection =
            Vec4::position(sample.position_world_space) * *world_to_shadow_map_camera_projection;

        let position_shadow_camera_ndc = {
            let mut result = position_shadow_camera_projection;

            result *= 1.0 / result.w;

            result.x = (result.x + 1.0) / 2.0;
            result.y = (-result.y + 1.0) / 2.0;

            result
        };

        // If the sample lies outside of shadow map camera NDC space, then this
        // point can't be seen by the shadow map camera.

        if position_shadow_camera_ndc.x < 0.0
            || position_shadow_camera_ndc.x > 1.0
            || position_shadow_camera_ndc.y < 0.0
            || position_shadow_camera_ndc.y > 1.0
        {
            return None;
        }

        // Otherwise, convert the NDC space coordinate to a UV coordinate, and perform a depth lookup.

        Some(Vec2 {
            x: position_shadow_camera_ndc.x,
            y: 1.0 - position_shadow_camera_ndc.y,
            z: 0.0,
        })
    }

    fn get_shadowing(&self, sample: &GeometrySample, map: &TextureMap<f32>) -> f32 {
        let context = self.shadow_map_rendering_context.as_ref().unwrap();

        let (near, far) = (SHADOW_MAP_CAMERA_NEAR, context.projection_z_far);

        let light_to_fragment = sample.position_world_space - self.look_vector.get_position();

        let current_depth = light_to_fragment.mag();

        let world_to_shadow_map_camera_projection =
            self.world_to_shadow_map_camera_projection.as_ref().unwrap();

        if let Some(uv) = self.get_shadow_map_uv(world_to_shadow_map_camera_projection, sample) {
            let closest_depth_sample = sample_nearest_f32(uv, map);

            let closest_depth = near + closest_depth_sample * (far - near);

            if closest_depth == 0.0 {
                return 0.0;
            }

            let bias = 0.005;

            if current_depth + bias > closest_depth {
                1.0
            } else {
                0.0
            }
        } else {
            0.0
        }
    }

    pub fn update_shadow_map_camera(&mut self) {
        let rendering_context = self.shadow_map_rendering_context.as_ref().unwrap();

        let z_far = rendering_context.projection_z_far;

        let field_of_view = self.outer_cutoff_angle * 2.0;

        let field_of_view_degrees = field_of_view.to_degrees();

        let aspect_ratio = 1.0;

        let mut camera = Camera::from_perspective(
            self.look_vector.get_position(),
            self.look_vector.get_target(),
            field_of_view_degrees,
            aspect_ratio,
        );

        camera.set_projection_z_near(SHADOW_MAP_CAMERA_NEAR);
        camera.set_projection_z_far(z_far);

        let world_to_camera_projection_space =
            camera.get_view_inverse_transform() * camera.get_projection();

        self.shadow_map_camera.replace(camera);

        self.world_to_shadow_map_camera_projection
            .replace(world_to_camera_projection_space);
    }

    fn render_shadow_map_into(
        &self,
        shadow_map: &mut TextureMap<f32>,
        rendering_context: &ShadowMapRenderingContext,
        resources: &SceneResources,
        scene: &SceneGraph,
    ) -> Result<(), String> {
        let camera = self.shadow_map_camera.as_ref().unwrap();

        {
            let mut shader_context = rendering_context.shader_context.borrow_mut();

            camera.update_shader_context(&mut shader_context);
        }

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

        // Blit our framebuffer's HDR attachment buffer (depth) to the shadow
        // texture map.

        let framebuffer = rendering_context.framebuffer.borrow();

        match &framebuffer.attachments.deferred_hdr {
            Some(hdr_attachment_rc) => {
                let hdr_buffer = hdr_attachment_rc.borrow();

                let target = &mut shadow_map.levels[0].0;

                for (index, hdr_color) in hdr_buffer.iter().enumerate() {
                    target.set_at(index, hdr_color.x);
                }
            }
            None => return Err(
                "Called SpotLight::update_shadow_maps() with no HDR attachment on the rendering context's framebuffer!"
                    .to_string(),
            ),
        }

        Ok(())
    }

    fn recompute_influence_distance(&mut self) {
        self.influence_distance = self.attenuation.get_approximate_influence_distance();
    }

    pub fn contribute(self, world_pos: Vec3) -> Vec3 {
        let fragment_to_light = self.look_vector.get_position() - world_pos;

        let direction_to_light = fragment_to_light.as_normal();

        let theta_angle =
            0.0_f32.max((self.look_vector.get_forward()).dot(direction_to_light * -1.0));

        let spot_attenuation =
            ((theta_angle - self.outer_cutoff_angle_cos) / self.epsilon).clamp(0.0, 1.0);

        if theta_angle > self.outer_cutoff_angle_cos {
            self.intensities * spot_attenuation
        } else {
            Default::default()
        }
    }

    pub fn contribute_pbr(
        &self,
        sample: &GeometrySample,
        f0: &Vec3,
        view_position: &Vec4,
        shadow_map: Option<&TextureMap<f32>>,
    ) -> Vec3 {
        let fragment_to_light = self.look_vector.get_position() - sample.position_world_space;

        let direction_to_light_world_space = fragment_to_light.as_normal();

        let theta_angle = 0.0_f32
            .max((self.look_vector.get_forward()).dot(direction_to_light_world_space * -1.0));

        let light_intensities = &self.intensities;

        let mut contribution = if theta_angle > self.outer_cutoff_angle_cos {
            contribute_pbr_world_space(
                sample,
                light_intensities,
                &direction_to_light_world_space,
                f0,
                view_position,
            )
        } else {
            return Default::default();
        };

        if let (Some(projector_map), Some(projection)) = (
            &self.projector_map,
            &self.world_to_shadow_map_camera_projection,
        ) {
            if let Some(uv) = self.get_shadow_map_uv(projection, sample) {
                let (r, g, b) = sample_nearest_u8(uv, projector_map, None);

                let mut color = Color::rgb(r, g, b).to_vec3() / 255.0;

                color.srgb_to_linear();

                contribution *= color;
            }
        }

        let attenuation =
            ((theta_angle - self.outer_cutoff_angle_cos) / self.epsilon).clamp(0.0, 1.0);

        if attenuation == 0.0 {
            return Default::default();
        }

        // Compute an enshadowing term for this fragment/sample.

        let in_shadow = if let Some(map) = shadow_map {
            self.get_shadowing(sample, map)
        } else {
            0.0
        };

        contribution * attenuation * (1.0 - in_shadow)
    }
}
