use std::{
    f32::consts::PI,
    fmt::{self, Display},
    rc::Rc,
};

use serde::{Deserialize, Serialize};

use crate::{
    animation::lerp,
    buffer::Buffer2D,
    matrix::Mat4,
    render::{culling::FaceCullingReject, Renderer},
    resource::{arena::Arena, handle::Handle},
    scene::{
        camera::{frustum::Frustum, Camera, CameraOrthographicExtent},
        graph::{options::SceneGraphRenderOptions, SceneGraph},
        resources::SceneResources,
    },
    serde::PostDeserialize,
    shader::{context::ShaderContext, geometry::sample::GeometrySample},
    shaders::{
        shadow_shaders::directional_shadows::DirectionalShadowMapFragmentShader,
        shadow_shaders::directional_shadows::DirectionalShadowMapGeometryShader,
        shadow_shaders::directional_shadows::DirectionalShadowMapVertexShader,
    },
    texture::{
        map::{TextureMap, TextureMapWrapping},
        sample::sample_nearest_f32,
    },
    transform::quaternion::Quaternion,
    vec::{
        vec2::Vec2,
        vec3::{self, Vec3},
        vec4::{self, Vec4},
    },
};

use super::{
    contribute_pbr_world_space,
    shadow::{ShadowMapRenderingContext, SHADOW_MAP_CAMERA_NEAR},
};

pub const SHADOW_MAP_CAMERA_COUNT: usize = 2;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectionalLight {
    pub intensities: Vec3,
    rotation: Quaternion,
    direction: Vec4,
    #[serde(skip)]
    pub shadow_maps: Option<Vec<Handle>>,
    #[serde(skip)]
    pub shadow_map_cameras: Option<Vec<(f32, Camera)>>,
    #[serde(skip)]
    pub shadow_map_rendering_context: Option<ShadowMapRenderingContext>,
}

impl Default for DirectionalLight {
    fn default() -> Self {
        let mut result = Self {
            intensities: Vec3::ones() * 0.15,
            rotation: Default::default(),
            direction: vec4::FORWARD,
            shadow_maps: None,
            shadow_map_cameras: None,
            shadow_map_rendering_context: None,
        };

        result.set_direction(
            Quaternion::new(vec3::UP, PI / 4.0) * Quaternion::new(vec3::RIGHT, -PI / 2.0),
        );

        result
    }
}

impl PostDeserialize for DirectionalLight {
    fn post_deserialize(&mut self) {
        // Nothing to do.
    }
}

impl Display for DirectionalLight {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "DirectionalLight(intensities={}, rotation={}, direction={})",
            self.intensities, self.rotation, self.direction
        )
    }
}

impl DirectionalLight {
    pub fn get_direction(&self) -> &Vec4 {
        &self.direction
    }

    pub fn set_direction(&mut self, rotation: Quaternion) {
        let rotation_mat = *rotation.mat();

        self.direction = vec4::FORWARD * rotation_mat;
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
            FaceCullingReject::None,
            DirectionalShadowMapVertexShader,
            DirectionalShadowMapGeometryShader,
            DirectionalShadowMapFragmentShader,
            scene_resources.clone(),
        );

        let (width, height) = (shadow_map_size, shadow_map_size);

        let mut blank_texture = TextureMap::from(Buffer2D::<f32>::new(width, height, None));

        blank_texture.sampling_options.wrapping = TextureMapWrapping::ClampToEdge;

        let mut handles: Vec<Handle> = vec![];

        {
            let mut texture_f32_arena = scene_resources.texture_f32.borrow_mut();

            for _ in 0..SHADOW_MAP_CAMERA_COUNT {
                handles.push(texture_f32_arena.insert(blank_texture.clone()))
            }
        }

        self.shadow_maps.replace(handles);

        self.shadow_map_rendering_context
            .replace(shadow_map_rendering_context);
    }

    pub fn update_shadow_maps(
        &mut self,
        resources: &SceneResources,
        scene: &SceneGraph,
    ) -> Result<(), String> {
        if let (Some(handles), Some(cameras), Some(rendering_context)) = (
            self.shadow_maps.as_ref(),
            self.shadow_map_cameras.as_ref(),
            self.shadow_map_rendering_context.as_ref(),
        ) {
            let mut texture_f32_arena = resources.texture_f32.borrow_mut();

            for (depth_index, (_far_z, camera)) in cameras.iter().enumerate() {
                let (near, far) = (
                    camera.get_projection_z_near(),
                    camera.get_projection_z_far(),
                );

                let shadow_map_handle = &handles[depth_index];

                if let Ok(entry) = texture_f32_arena.get_mut(shadow_map_handle) {
                    let map = &mut entry.item;

                    {
                        let framebuffer = rendering_context.framebuffer.borrow_mut();

                        match framebuffer.attachments.depth.as_ref() {
                            Some(attachment) => {
                                let mut zbuffer = attachment.borrow_mut();

                                zbuffer.set_projection_z_near(camera.get_projection_z_near());
                                zbuffer.set_projection_z_far(camera.get_projection_z_far());
                            }
                            None => panic!(),
                        }
                    }

                    {
                        let mut shader_context = rendering_context.shader_context.borrow_mut();

                        shader_context.projection_z_near.replace(near);
                        shader_context.projection_z_far.replace(far);

                        shader_context
                            .directional_light_view_projection_index
                            .replace(depth_index);

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

                    // Blit our framebuffer's color attachment buffer to our
                    // cubemap face texture.

                    let framebuffer = rendering_context.framebuffer.borrow();

                    match &framebuffer.attachments.deferred_hdr {
                        Some(hdr_attachment_rc) => {
                            let hdr_attachment = hdr_attachment_rc.borrow();

                            let buffer = &mut map.levels[0].0;

                            for (index, hdr_color) in hdr_attachment.data.iter().enumerate() {
                                buffer.set_at(index, hdr_color.x);
                            }
                        }
                        None => return Err(
                            "Called DirectionalLight::update_shadow_maps() with no HDR attachment on the rendering context's framebuffer!".to_string()
                        ),
                    }
                }
            }
        }

        Ok(())
    }

    pub fn update_shadow_map_cameras(&mut self, view_camera: &Camera) {
        let forward = self.direction.as_normal().to_vec3();
        let right = vec3::UP.cross(forward).as_normal();
        let up = forward.cross(right).as_normal();

        let alpha_step = 1.0 / SHADOW_MAP_CAMERA_COUNT as f32;

        let view_camera_projection_depth =
            view_camera.get_projection_z_far() - view_camera.get_projection_z_near();

        let projection_depth_step = view_camera_projection_depth / SHADOW_MAP_CAMERA_COUNT as f32;

        let frustum = view_camera.get_frustum();

        let subfrustum_cameras: Vec<(f32, Camera)> = (0..SHADOW_MAP_CAMERA_COUNT)
            .map(|subfrustum_index| {
                let near_alpha = alpha_step * subfrustum_index as f32;

                let far_alpha = alpha_step * (subfrustum_index + 1) as f32;

                let near = [
                    lerp(frustum.near[0], frustum.far[0], near_alpha),
                    lerp(frustum.near[1], frustum.far[1], near_alpha),
                    lerp(frustum.near[2], frustum.far[2], near_alpha),
                    lerp(frustum.near[3], frustum.far[3], near_alpha),
                ];

                let far = [
                    lerp(frustum.near[0], frustum.far[0], far_alpha),
                    lerp(frustum.near[1], frustum.far[1], far_alpha),
                    lerp(frustum.near[2], frustum.far[2], far_alpha),
                    lerp(frustum.near[3], frustum.far[3], far_alpha),
                ];

                let subfrustum = Frustum::new(forward, near, far);

                let subfrustum_far_z = projection_depth_step * (subfrustum_index + 1) as f32;

                let subfrustum_center = subfrustum.get_center();

                let mut min_z = f32::MAX;
                let mut max_z = f32::MIN;

                let light_extent = {
                    let mut min_x = f32::MAX;
                    let mut max_x = f32::MIN;
                    let mut min_y = f32::MAX;
                    let mut max_y = f32::MIN;

                    let light_view_inverse_transform =
                        Mat4::look_at(subfrustum_center, forward, right, up);

                    for coord in subfrustum.near.iter().chain(subfrustum.far.iter()) {
                        let view_space_coord =
                            Vec4::position(*coord) * light_view_inverse_transform;

                        min_x = min_x.min(view_space_coord.x);
                        max_x = max_x.max(view_space_coord.x);

                        min_y = min_y.min(view_space_coord.y);
                        max_y = max_y.max(view_space_coord.y);

                        min_z = min_z.min(view_space_coord.z);
                        max_z = max_z.max(view_space_coord.z);
                    }

                    CameraOrthographicExtent {
                        left: min_x,
                        right: max_x,
                        top: max_y,
                        bottom: min_y,
                    }
                };

                let depth_range = max_z - min_z;

                let camera_position = subfrustum_center - forward * depth_range;

                let mut camera = Camera::orthographic(
                    camera_position,
                    camera_position + self.direction.to_vec3(),
                    light_extent,
                );

                camera.set_projection_z_near(SHADOW_MAP_CAMERA_NEAR);
                camera.set_projection_z_far(depth_range * 2.0);

                (subfrustum_far_z, camera)
            })
            .collect();

        self.shadow_map_cameras = Some(subfrustum_cameras);
    }

    pub fn contribute(self, sample: &GeometrySample) -> Vec3 {
        let tangent_space_info = sample.tangent_space_info;

        let normal = &tangent_space_info.normal;

        let direction_to_light = (self.direction * -1.0 * tangent_space_info.tbn_inverse)
            .to_vec3()
            .as_normal();

        self.intensities * 0.0_f32.max((*normal * -1.0).dot(direction_to_light))
    }

    pub fn contribute_pbr(
        &self,
        sample: &GeometrySample,
        f0: &Vec3,
        texture_f32_arena: &Arena<TextureMap<f32>>,
        context: &ShaderContext,
        shadow_map_handles: Option<&Vec<Handle>>,
    ) -> Vec3 {
        let direction_to_light_world_space = (self.direction * -1.0).as_normal().to_vec3();

        // Compute an enshadowing term for this fragment/sample.

        let in_shadow = if let Some(maps) = shadow_map_handles {
            self.get_shadowing(sample, texture_f32_arena, context, maps)
        } else {
            0.0
        };

        let intensity = self.intensities;

        let contribution = contribute_pbr_world_space(
            sample,
            &intensity,
            &direction_to_light_world_space,
            f0,
            &context.view_position,
        );

        contribution * (1.0 - in_shadow)
    }

    fn pcf_3x3(current_depth_ndc_space: f32, map: &TextureMap<f32>, uv: Vec2) -> f32 {
        let mut shadow = 0.0;

        for y in -1..1 {
            for x in -1..1 {
                let perturbed_uv = uv
                    + Vec2 {
                        x: x as f32,
                        y: y as f32,
                        z: 0.0,
                    } * map.levels[0].0.texel_size;

                if perturbed_uv.x < 0.0
                    || perturbed_uv.x > 1.0
                    || perturbed_uv.y < 0.0
                    || perturbed_uv.y > 1.0
                {
                    continue;
                }

                let closest_depth_ndc_space = sample_nearest_f32(perturbed_uv, map);

                if closest_depth_ndc_space == 0.0 {
                    continue;
                }

                let bias = 0.0025;

                let is_in_shadow = current_depth_ndc_space - bias > closest_depth_ndc_space;

                if is_in_shadow {
                    shadow += 1.0;
                }
            }
        }

        shadow / 9.0
    }

    fn poisson_3x3(current_depth_ndc_space: f32, map: &TextureMap<f32>, uv: Vec2) -> f32 {
        static POISSON_DISK_SAMPLES: [Vec2; 4] = [
            Vec2 {
                x: -0.942_016_24,
                y: -0.399_062_16,
                z: 0.0,
            },
            Vec2 {
                x: 0.945_586_1,
                y: -0.768_907_25,
                z: 0.0,
            },
            Vec2 {
                x: -0.094_184_1,
                y: -0.929_388_7,
                z: 0.0,
            },
            Vec2 {
                x: 0.344_959_38,
                y: 0.293_877_6,
                z: 0.0,
            },
        ];

        let mut shadow = 0.0;

        for sample in &POISSON_DISK_SAMPLES {
            let poisson_uv = uv + (*sample / 700.0);

            shadow += Self::pcf_3x3(current_depth_ndc_space, map, poisson_uv);
        }

        shadow / POISSON_DISK_SAMPLES.len() as f32
    }

    fn get_shadowing_for_map(
        &self,
        sample: &GeometrySample,
        map: &TextureMap<f32>,
        transform: &Mat4,
    ) -> f32 {
        let sample_position_light_view_projection_space =
            Vec4::position(sample.position_world_space) * *transform;

        let sample_position_light_ndc_space = sample_position_light_view_projection_space
            / sample_position_light_view_projection_space.w;

        let current_depth_ndc_space = sample_position_light_ndc_space.z;

        let uv = sample_position_light_ndc_space.ndc_to_uv();

        Self::poisson_3x3(current_depth_ndc_space, map, uv)
    }

    fn get_shadowing(
        &self,
        sample: &GeometrySample,
        texture_f32_arena: &Arena<TextureMap<f32>>,
        context: &ShaderContext,
        shadow_map_handles: &[Handle],
    ) -> f32 {
        match &context.directional_light_view_projections {
            Some(transforms) => {
                let fragment_position_view_space =
                    Vec4::position(sample.position_world_space) * context.view_inverse_transform;

                let index = {
                    let mut index = SHADOW_MAP_CAMERA_COUNT - 1;

                    for (i, transform) in transforms.iter().enumerate() {
                        let (far_z, _transform) = transform;

                        if fragment_position_view_space.z.abs() < *far_z {
                            index = i;

                            break;
                        }
                    }

                    index
                };

                let shadow_map_handle = &shadow_map_handles[index];

                if let Ok(entry) = texture_f32_arena.get(shadow_map_handle) {
                    let map = &entry.item;

                    let transform = &transforms[index].1;

                    self.get_shadowing_for_map(sample, map, transform)
                } else {
                    0.0
                }
            }
            _ => 0.0,
        }
    }
}
