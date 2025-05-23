#![allow(non_upper_case_globals)]

use std::f32::consts::TAU;

use crate::{
    matrix::Mat4,
    render::options::tone_mapping::ToneMappingOperator,
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

pub static HdrEquirectangularProjectionVertexShader: VertexShaderFn =
    |context: &ShaderContext, v: &DefaultVertexIn| -> DefaultVertexOut {
        // Object-to-world-space vertex transform

        let mut out = DefaultVertexOut {
            position_projection_space: Vec4::position(v.position)
                * context.world_view_projection_transform,
            ..Default::default()
        };

        let world_position = Vec4::position(v.position) * context.world_transform;

        out.position_world_space = Vec3 {
            x: world_position.x,
            y: world_position.y,
            z: world_position.z,
        };

        // Computes a tangent-to-world-space transform.

        let normal = (v.normal * context.world_transform).as_normal();
        let tangent = (v.tangent * context.world_transform).as_normal();
        let bitangent = (v.bitangent * context.world_transform).as_normal();

        out.normal_world_space = normal;
        out.tangent_world_space = tangent;
        out.bitangent_world_space = bitangent;

        let (t, b, n) = (tangent, bitangent, normal);

        // Note: Reversed Z-axis for our renderer's coordinate system.

        let tbn = Mat4::tbn(t, b, n);

        let tbn_inverse = tbn.transposed();

        out.tangent_space_info = TangentSpaceInfo {
            tbn,
            tbn_inverse,
            normal: (normal * tbn_inverse),
            view_position: (context.view_position * tbn_inverse).to_vec3(),
            fragment_position: (world_position * tbn_inverse).to_vec3(),
        };

        out
    };

pub static HdrEquirectangularProjectionFragmentShader: FragmentShaderFn =
    |shader_context: &ShaderContext, resources: &SceneResources, sample: &GeometrySample| -> Vec3 {
        fn sample_spherical_to_cartesian(pos: Vec3) -> Vec2 {
            // See: http://disq.us/p/2nvby4v

            let n = pos.as_normal();

            let u = (pos.x).atan2(pos.z) / TAU + 0.5;
            let v = n.y * 0.5 + 0.5;

            Vec2 { x: u, y: v, z: 0.0 }
        }

        static TONE_MAPPING_OPERATOR: ToneMappingOperator = ToneMappingOperator::Exposure(100.0);

        let handle = shader_context.active_hdr_map.unwrap();

        match resources.texture_vec3.borrow().get(&handle) {
            Ok(entry) => {
                let map = &entry.item;

                let uv = sample_spherical_to_cartesian(sample.position_world_space.as_normal());

                let sample = sample_nearest_vec3(uv, map, None) / 255.0;

                // Exposure tone mapping

                TONE_MAPPING_OPERATOR.map(sample)
            }
            Err(_) => panic!(),
        }
    };
