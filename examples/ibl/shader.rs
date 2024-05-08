#![allow(non_upper_case_globals)]

use std::f32::consts::PI;

use cairo::{
    color::{self, Color},
    matrix::Mat4,
    scene::resources::SceneResources,
    shader::{
        context::ShaderContext, fragment::FragmentShaderFn, geometry::sample::GeometrySample,
        vertex::VertexShaderFn,
    },
    texture::sample::sample_nearest_vec3,
    vec::{vec2::Vec2, vec3::Vec3, vec4::Vec4},
    vertex::{
        default_vertex_in::DefaultVertexIn,
        default_vertex_out::{DefaultVertexOut, TangentSpaceInfo},
    },
};

pub const HdrEquirectangularProjectionVertexShader: VertexShaderFn =
    |context: &ShaderContext, v: &DefaultVertexIn| -> DefaultVertexOut {
        // Object-to-world-space vertex transform

        let mut out = DefaultVertexOut::new();

        out.position = Vec4::new(v.position, 1.0) * context.world_view_projection_transform;

        // debug_assert!(out.position.w != 0.0);

        let world_pos = Vec4::new(v.position, 1.0) * context.world_transform;

        out.world_pos = Vec3 {
            x: world_pos.x,
            y: world_pos.y,
            z: world_pos.z,
        };

        // Compute a tangent-space to world-space transform.

        let normal = (Vec4::new(v.normal, 0.0) * context.world_transform).as_normal();
        let tangent = (Vec4::new(v.tangent, 0.0) * context.world_transform).as_normal();
        let bitangent = (Vec4::new(v.bitangent, 0.0) * context.world_transform).as_normal();

        out.normal = normal;
        out.tangent = tangent;
        out.bitangent = bitangent;

        let (t, b, n) = (tangent, bitangent, normal);

        // @NOTE(mzalla) Reversed Z-axis for our renderer's coordinate system.

        let tbn = Mat4::tbn(t.to_vec3(), b.to_vec3(), n.to_vec3());

        let tbn_inverse = tbn.transposed();

        out.tangent_space_info = TangentSpaceInfo {
            tbn,
            tbn_inverse,
            normal: (normal * tbn_inverse).to_vec3(),
            view_position: (context.view_position * tbn_inverse).to_vec3(),
            fragment_position: (world_pos * tbn_inverse).to_vec3(),
        };

        out
    };

pub const HdrEquirectangularProjectionFragmentShader: FragmentShaderFn =
    |shader_context: &ShaderContext,
     resources: &SceneResources,
     sample: &GeometrySample|
     -> Color {
        fn sample_spherical_to_cartesian(pos: Vec3) -> Vec2 {
            // See: http://disq.us/p/2nvby4v

            let n = pos.as_normal();

            let u = (pos.x).atan2(pos.z) / (2.0 * PI) + 0.5;
            let v = n.y * 0.5 + 0.5;

            Vec2 { x: u, y: v, z: 0.0 }
        }

        static HDR_EXPOSURE: f32 = 100.0;

        if let Some(handle) = shader_context.active_hdr_map {
            if let Ok(entry) = resources.hdr.borrow().get(&handle) {
                let map = &entry.item;

                let uv: Vec2 = sample_spherical_to_cartesian(sample.world_pos.as_normal());

                let mut sample = sample_nearest_vec3(uv, map) / 255.0;

                sample *= HDR_EXPOSURE;

                // return Color::from_vec3(sample);

                // Exposure tone mapping

                let color_tone_mapped_vec3 = Vec3::ones()
                    - Vec3 {
                        x: (-sample.x).exp(),
                        y: (-sample.y).exp(),
                        z: (-sample.z).exp(),
                    };

                return Color::from_vec3(color_tone_mapped_vec3);
            }
        }

        color::GREEN
    };

pub const HdrCubemapConvolutionFragmentShader: FragmentShaderFn =
    |shader_context: &ShaderContext,
     resources: &SceneResources,
     sample: &GeometrySample|
     -> Color {
        if let Some(handle) = shader_context.active_ambient_diffuse_map {
            if let Ok(entry) = resources.skybox_hdr.borrow().get(&handle) {
                let map = &entry.item;

                let normal = sample.world_pos.as_normal();

                // return Color::from_vec3(map.sample(&Vec4::new(normal, 1.0)));

                let mut irradiance = Vec3::new();

                static SAMPLE_DELTA: f32 = 0.05;
                // static SAMPLE_DELTA: f32 = 0.25;

                let mut sample_count: usize = 0;

                // Convolution.

                let mut up = Vec3 {
                    y: 1.0,
                    ..Default::default()
                };
                let right = up.cross(normal).as_normal();
                up = normal.cross(right).as_normal();

                let mut phi = 0.0;

                while phi < 2.0 * PI {
                    let mut theta = 0.0;

                    while theta < 0.5 * PI {
                        // Spherical to cartesian (in tangent space).

                        let cartesian = Vec3 {
                            x: theta.sin() * phi.cos(),
                            y: theta.sin() * phi.sin(),
                            z: theta.cos(),
                        };

                        // Tangent space to world position.

                        let world_pos =
                            right * cartesian.x + up * cartesian.y + normal * cartesian.z;

                        // Sampled radiance, scaled by the incoming light angle (theta).

                        // "Note that we scale the sampled color value by
                        // cos(theta) due to the light being weaker at larger
                        // angles and by sin(theta) to account for the smaller
                        // sample areas in the higher hemisphere areas."

                        let radiance =
                            map.sample(&Vec4::new(world_pos, 1.0))/* * theta.cos() * theta.sin()*/;

                        irradiance += radiance;

                        sample_count += 1;

                        theta += SAMPLE_DELTA;
                    }

                    phi += SAMPLE_DELTA;
                }

                irradiance = irradiance * PI * (1.0 / sample_count as f32);

                return Color::from_vec3(irradiance);
            }
        }

        color::GREEN
    };

pub const AmbientDiffuseCubemapFragmentShader: FragmentShaderFn =
    |shader_context: &ShaderContext,
     resources: &SceneResources,
     sample: &GeometrySample|
     -> Color {
        if let Some(handle) = shader_context.active_ambient_diffuse_map {
            if let Ok(entry) = resources.skybox_hdr.borrow().get(&handle) {
                let map = &entry.item;

                let normal = sample.world_pos.as_normal();

                let irradiance = map.sample(&Vec4::new(normal, 1.0));

                return Color::from_vec3(irradiance);
            }
        }
        color::GREEN
    };
