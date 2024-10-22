use std::{fmt, path::Path};

use serde::{Deserialize, Serialize};

use crate::{
    physics::pbr::bake::{
        bake_diffuse_and_specular_from_hdri, brdf::generate_specular_brdf_integration_map,
    },
    resource::{arena::Arena, handle::Handle},
    serde::PostDeserialize,
    texture::{cubemap::CubeMap, map::TextureMap},
    vec::{vec2::Vec2, vec3::Vec3},
};

#[derive(Default, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct Skybox {
    pub is_hdr: bool,
    pub radiance: Option<Handle>,
    pub irradiance: Option<Handle>,
    pub specular_prefiltered_environment: Option<Handle>,
    pub ambient_specular_brdf_integration: Option<Handle>,
}

impl PostDeserialize for Skybox {
    fn post_deserialize(&mut self) {
        // Nothing to do.
    }
}

impl fmt::Display for Skybox {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Skybox (is_hdr={})", self.is_hdr)
    }
}

impl Skybox {
    pub fn load_hdr(
        &mut self,
        texture_vec2_arena: &mut Arena<TextureMap<Vec2>>,
        cubemap_vec3_arena: &mut Arena<CubeMap<Vec3>>,
        hdr_path: &Path,
    ) {
        if self.ambient_specular_brdf_integration.is_none() {
            // Generate a common BRDF integration map, approximating the
            // integral formed by our Geometry function over varying angles and
            // roughness values.

            let specular_brdf_integration_map_handle = {
                let specular_brdf_integration_map = generate_specular_brdf_integration_map(512);

                texture_vec2_arena.insert(specular_brdf_integration_map.to_owned())
            };

            self.ambient_specular_brdf_integration
                .replace(specular_brdf_integration_map_handle);
        }

        // Generate the radiance-irradiance cubemap pair.

        let bake_result = bake_diffuse_and_specular_from_hdri(hdr_path).unwrap();

        let radiance_cubemap_handle = cubemap_vec3_arena.insert(bake_result.radiance.to_owned());

        let irradiance_cubemap_handle =
            cubemap_vec3_arena.insert(bake_result.diffuse_irradiance.to_owned());

        self.radiance.replace(radiance_cubemap_handle);

        self.irradiance.replace(irradiance_cubemap_handle);

        // Generates the specular environment map.

        let specular_prefiltered_environment_cubemap_handle =
            cubemap_vec3_arena.insert(bake_result.specular_prefiltered_environment.to_owned());

        self.specular_prefiltered_environment
            .replace(specular_prefiltered_environment_cubemap_handle);
    }
}
