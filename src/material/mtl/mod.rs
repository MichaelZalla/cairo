use std::{path::Path, str::SplitWhitespace};

use uuid::Uuid;

use crate::{
    resource::arena::Arena,
    texture::map::TextureMapStorageFormat,
    {fs::read_lines, texture::map::TextureMap, vec::vec3::Vec3},
};

use super::cache::MaterialCache;

use super::Material;

macro_rules! create_and_set_material_map {
    ($line_tokens:ident, $file_path:ident, $texture_arena:ident, $cache:ident, $current_material_name:ident,$map_field:ident) => {
        let mtl_relative_filepath = next_filepath(&mut $line_tokens, $file_path);

        let texture_map_handle = $texture_arena.insert(
            Uuid::new_v4(),
            TextureMap::new(
                &mtl_relative_filepath.as_str(),
                TextureMapStorageFormat::RGB24,
            ),
        );

        $cache
            .get_mut($current_material_name.as_ref().unwrap())
            .unwrap()
            .$map_field = Some(texture_map_handle);
    };
}

pub fn load_mtl(filepath: &str, texture_arena: &mut Arena<TextureMap>) -> MaterialCache {
    let mtl_file_path = Path::new(&filepath);
    let mtl_file_path_display = mtl_file_path.display();

    let lines = match read_lines(mtl_file_path) {
        Err(why) => panic!("Failed to open file {}: {}", mtl_file_path_display, why),
        Ok(lines) => lines,
    };

    let mut cache: MaterialCache = Default::default();

    let mut current_material_name: Option<String> = None;

    for (_, line) in lines.enumerate() {
        match line {
            Err(why) => println!("Error reading next line: {}", why),
            Ok(line) => {
                let mut line_tokens = line.split_whitespace();

                match line_tokens.next() {
                    None => (),
                    Some(first) => {
                        match first.to_lowercase().as_str() {
                            // Comment
                            "#" => (),

                            // Material entry
                            "newmtl" => {
                                // Example:
                                // newmtl cube

                                let source = mtl_file_path_display.to_string();

                                let name = line_tokens.next().unwrap().to_string();

                                current_material_name = Some(name.clone());

                                let mut material = Material::new(name.clone());

                                material.material_source = Some(source);

                                cache.insert(material);
                            }

                            // Illumination model
                            "illum" => {
                                // [0, 10] range
                                // Example:
                                // illum 2
                                let value = line_tokens.next().unwrap().parse::<u8>().unwrap();

                                cache
                                    .get_mut(current_material_name.as_ref().unwrap())
                                    .unwrap()
                                    .illumination_model = value;
                            }

                            // Ambient color
                            "ka" => {
                                // R G B
                                // Example:
                                // Ka 0.0000 0.0000 0.0000

                                cache
                                    .get_mut(current_material_name.as_ref().unwrap())
                                    .unwrap()
                                    .ambient_color = next_rgb(&mut line_tokens);
                            }

                            // Diffuse color
                            "kd" => {
                                // R G B
                                // Example:
                                // Kd 0.5880 0.5880 0.5880

                                cache
                                    .get_mut(current_material_name.as_ref().unwrap())
                                    .unwrap()
                                    .diffuse_color = next_rgb(&mut line_tokens);
                            }

                            // Specular color
                            "ks" => {
                                // R G B
                                // Example:
                                // Ks 0.0000 0.0000 0.0000

                                cache
                                    .get_mut(current_material_name.as_ref().unwrap())
                                    .unwrap()
                                    .specular_color = next_rgb(&mut line_tokens);
                            }

                            // Specular exponent
                            "ns" => {
                                // [0, 1000] range
                                // Example:
                                // Ns 10.0000

                                let value = line_tokens.next().unwrap().parse::<f32>().unwrap();

                                cache
                                    .get_mut(current_material_name.as_ref().unwrap())
                                    .unwrap()
                                    .specular_exponent = value as i32;
                            }

                            // Emissive color
                            "ke" => {
                                // R G B
                                // Example:
                                // Ke 0.0000 0.0000 0.0000

                                cache
                                    .get_mut(current_material_name.as_ref().unwrap())
                                    .unwrap()
                                    .emissive_color = next_rgb(&mut line_tokens);
                            }

                            // Dissolve (opaqueness)
                            "d" => {
                                // [0, 1] range
                                // Example:
                                // d 1.0000

                                let value = line_tokens.next().unwrap().parse::<f32>().unwrap();

                                cache
                                    .get_mut(current_material_name.as_ref().unwrap())
                                    .unwrap()
                                    .dissolve = value;
                            }

                            // Transparency
                            "tr" => {
                                // [0, 1] range
                                // Example:
                                // Tr 0.0000

                                let value = line_tokens.next().unwrap().parse::<f32>().unwrap();

                                cache
                                    .get_mut(current_material_name.as_ref().unwrap())
                                    .unwrap()
                                    .transparency = value;
                            }

                            // Translucency
                            "tf" => {
                                // R G B
                                // Example:
                                // Tf 1.0000 1.0000 1.0000

                                cache
                                    .get_mut(current_material_name.as_ref().unwrap())
                                    .unwrap()
                                    .translucency = next_rgb(&mut line_tokens);
                            }

                            // Index of refraction
                            "ni" => {
                                // [0.001, 10] range
                                // Example:
                                // Ni 1.5000

                                let value = line_tokens.next().unwrap().parse::<f32>().unwrap();

                                cache
                                    .get_mut(current_material_name.as_ref().unwrap())
                                    .unwrap()
                                    .index_of_refraction = value;
                            }

                            // Ambient texture map
                            "map_ka" => {
                                // [filepath]
                                // Example:
                                // map_Ka cube.png

                                create_and_set_material_map!(
                                    line_tokens,
                                    mtl_file_path,
                                    texture_arena,
                                    cache,
                                    current_material_name,
                                    ambient_color_map
                                );
                            }

                            // Diffuse texture map (typically identical to map_Ka)
                            "map_kd" => {
                                // [filepath]
                                // Example:
                                // map_Kd cube.png

                                create_and_set_material_map!(
                                    line_tokens,
                                    mtl_file_path,
                                    texture_arena,
                                    cache,
                                    current_material_name,
                                    diffuse_color_map
                                );
                            }

                            // Specular color map
                            "map_ks" | "map_ns" => {
                                // [filepath]
                                // Example:
                                // map_Ks cube_specular.png

                                create_and_set_material_map!(
                                    line_tokens,
                                    mtl_file_path,
                                    texture_arena,
                                    cache,
                                    current_material_name,
                                    specular_exponent_map
                                );
                            }

                            // Emissive color map
                            "map_ke" => {
                                // [filepath]
                                // Example:
                                // map_Ke cube_emissive.png

                                create_and_set_material_map!(
                                    line_tokens,
                                    mtl_file_path,
                                    texture_arena,
                                    cache,
                                    current_material_name,
                                    emissive_color_map
                                );
                            }

                            // Alpha map
                            "map_d" => {
                                create_and_set_material_map!(
                                    line_tokens,
                                    mtl_file_path,
                                    texture_arena,
                                    cache,
                                    current_material_name,
                                    alpha_map
                                );
                            }

                            // Bump map
                            "map_bump" | "bump" => {
                                println!("@TODO Implementation for token 'map_bump' and 'bump'.");
                            }

                            // Normal map
                            "map_kn" | "map_normal" | "norm" => {
                                // [filepath]
                                // Example:
                                // map_Kn cube_normal.png

                                create_and_set_material_map!(
                                    line_tokens,
                                    mtl_file_path,
                                    texture_arena,
                                    cache,
                                    current_material_name,
                                    normal_map
                                );
                            }

                            // Displacement (height) map
                            "map_disp" | "disp" => {
                                // [filepath]
                                // Example:
                                // disp cube_displacement.png

                                create_and_set_material_map!(
                                    line_tokens,
                                    mtl_file_path,
                                    texture_arena,
                                    cache,
                                    current_material_name,
                                    displacement_map
                                );
                            }

                            // Stencil (decal) map
                            "decal" => {
                                println!("@TODO Implementation for token 'decal'.");
                            }

                            // Unrecognized prefix
                            other => {
                                println!("Unrecognized MTL token: {}", other)
                            }
                        }
                    }
                }
            }
        }
    }

    let count = cache.len();

    println!(
        "Parsed {} material{} from \"{}\".",
        count,
        if count > 1 { "s" } else { "" },
        mtl_file_path_display
    );

    println!();

    for material in cache.values() {
        println!("{}", material);
    }

    cache
}

fn next_rgb(line_tokens: &mut SplitWhitespace<'_>) -> Vec3 {
    let r = line_tokens.next().unwrap().parse::<f32>().unwrap();
    let g = line_tokens.next().unwrap().parse::<f32>().unwrap();
    let b = line_tokens.next().unwrap().parse::<f32>().unwrap();

    Vec3 { x: r, y: g, z: b }
}

fn next_filepath(line_tokens: &mut SplitWhitespace<'_>, mtl_file_path: &Path) -> String {
    let filepath = line_tokens.next().unwrap().to_string();

    let mtl_relative_filepath = mtl_file_path
        .parent()
        .unwrap()
        .join(filepath)
        .into_os_string()
        .into_string()
        .unwrap();

    mtl_relative_filepath
}
