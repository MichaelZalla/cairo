use std::{path::Path, str::SplitWhitespace};

use crate::{
    fs::read_lines,
    resource::{arena::Arena, handle::Handle},
    texture::map::{TextureMap, TextureMapStorageFormat},
    vec::vec3::Vec3,
};

use super::Material;

pub fn load_mtl(
    filepath: &str,
    material_arena: &mut Arena<Material>,
    texture_arena: &mut Arena<TextureMap>,
) {
    let mtl_file_path = Path::new(&filepath);
    let mtl_file_path_display = mtl_file_path.display();

    let lines = match read_lines(mtl_file_path) {
        Err(why) => panic!("Failed to open file {}: {}", mtl_file_path_display, why),
        Ok(lines) => lines,
    };

    let mut parsed_materials_count: usize = 0;

    let mut current_material: Option<Handle> = None;

    for line in lines {
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

                                let mut material = Material::new(name.clone());

                                material.material_source = Some(source);

                                current_material.replace(material_arena.insert(material));

                                parsed_materials_count += 1;
                            }

                            // Illumination model
                            "illum" => {
                                // [0, 10] range
                                // Example:
                                // illum 2
                                let value = line_tokens.next().unwrap().parse::<u8>().unwrap();

                                if let Ok(entry) =
                                    material_arena.get_mut(&current_material.unwrap())
                                {
                                    let material = &mut entry.item;

                                    material.illumination_model = value;
                                }
                            }

                            // Common attributes
                            //
                            // See: https://benhouston3d.com/blog/extended-wavefront-obj-mtl-for-pbr/
                            //

                            // Albedo
                            "kd" => {
                                // (r,g,b)
                                // Example:
                                // Kd 0.5880 0.5880 0.5880

                                if let Ok(entry) =
                                    material_arena.get_mut(&current_material.unwrap())
                                {
                                    let material = &mut entry.item;

                                    let color = next_rgb(&mut line_tokens);

                                    material.albedo = color;
                                }
                            }

                            // Albedo map
                            "map_kd" => {
                                // [filepath]
                                // Example:
                                // map_Kd cube.png

                                let mtl_relative_filepath =
                                    next_filepath(&mut line_tokens, mtl_file_path);

                                let texture_map_handle = texture_arena.insert(TextureMap::new(
                                    mtl_relative_filepath.as_str(),
                                    TextureMapStorageFormat::RGB24,
                                ));

                                if let Ok(entry) =
                                    material_arena.get_mut(&current_material.unwrap())
                                {
                                    let material = &mut entry.item;

                                    material.albedo_map = Some(texture_map_handle);
                                }
                            }

                            // Emissive color
                            "ke" => {
                                // (r,g,b)
                                // Example:
                                // Ke 0.0000 0.0000 0.0000

                                if let Ok(entry) =
                                    material_arena.get_mut(&current_material.unwrap())
                                {
                                    let material = &mut entry.item;

                                    material.emissive_color = next_rgb(&mut line_tokens);
                                }
                            }

                            // Emissive color map
                            "map_ke" => {
                                // [filepath]
                                // Example:
                                // map_Ke cube_emissive.png

                                let mtl_relative_filepath =
                                    next_filepath(&mut line_tokens, mtl_file_path);

                                let texture_map_handle = texture_arena.insert(TextureMap::new(
                                    mtl_relative_filepath.as_str(),
                                    TextureMapStorageFormat::RGB24,
                                ));

                                if let Ok(entry) =
                                    material_arena.get_mut(&current_material.unwrap())
                                {
                                    let material = &mut entry.item;

                                    material.emissive_color_map = Some(texture_map_handle);
                                }
                            }

                            // Dissolve (opaqueness)
                            "d" => {
                                // [0, 1] range
                                // Example:
                                // d 1.0000

                                let value = line_tokens.next().unwrap().parse::<f32>().unwrap();

                                if let Ok(entry) =
                                    material_arena.get_mut(&current_material.unwrap())
                                {
                                    let material = &mut entry.item;

                                    material.dissolve = value;
                                }
                            }

                            // Alpha map
                            "map_d" => {
                                // [filepath]
                                // Example:
                                // map_d cube_alpha.png

                                let mtl_relative_filepath =
                                    next_filepath(&mut line_tokens, mtl_file_path);

                                let texture_map_handle = texture_arena.insert(TextureMap::new(
                                    mtl_relative_filepath.as_str(),
                                    TextureMapStorageFormat::Index8(0),
                                ));

                                if let Ok(entry) =
                                    material_arena.get_mut(&current_material.unwrap())
                                {
                                    let material = &mut entry.item;

                                    material.alpha_map = Some(texture_map_handle);
                                }
                            }

                            // Transparency
                            "tr" => {
                                // [0, 1] range
                                // Example:
                                // Tr 0.0000

                                let value = line_tokens.next().unwrap().parse::<f32>().unwrap();

                                if let Ok(entry) =
                                    material_arena.get_mut(&current_material.unwrap())
                                {
                                    let material = &mut entry.item;

                                    material.transparency = value;
                                }
                            }

                            // Transparency map
                            "map_tr" => {
                                // [filepath]
                                // Example:
                                // map_d cube_transparency.png

                                let mtl_relative_filepath =
                                    next_filepath(&mut line_tokens, mtl_file_path);

                                let texture_map_handle = texture_arena.insert(TextureMap::new(
                                    mtl_relative_filepath.as_str(),
                                    TextureMapStorageFormat::Index8(0),
                                ));

                                if let Ok(entry) =
                                    material_arena.get_mut(&current_material.unwrap())
                                {
                                    let material = &mut entry.item;

                                    material.transparency_map = Some(texture_map_handle);
                                }
                            }

                            // Translucency (transmission filter color)
                            "tf" => {
                                // (r,g,b)
                                // Example:
                                // Tf 1.0000 1.0000 1.0000

                                if let Ok(entry) =
                                    material_arena.get_mut(&current_material.unwrap())
                                {
                                    let material = &mut entry.item;

                                    material.translucency = next_rgb(&mut line_tokens);
                                }
                            }

                            // Bump map
                            "map_bump" | "bump" => {
                                // [filepath]
                                // Example:
                                // map_bump cube_bump.png

                                let mtl_relative_filepath =
                                    next_filepath(&mut line_tokens, mtl_file_path);

                                let texture_map_handle = texture_arena.insert(TextureMap::new(
                                    mtl_relative_filepath.as_str(),
                                    TextureMapStorageFormat::Index8(0),
                                ));

                                if let Ok(entry) =
                                    material_arena.get_mut(&current_material.unwrap())
                                {
                                    let material = &mut entry.item;

                                    material.bump_map = Some(texture_map_handle);
                                }
                            }

                            // Normal map
                            "map_kn" | "map_normal" | "norm" => {
                                // [filepath]
                                // Example:
                                // map_Kn cube_normal.png

                                let mtl_relative_filepath =
                                    next_filepath(&mut line_tokens, mtl_file_path);

                                let texture_map_handle = texture_arena.insert(TextureMap::new(
                                    mtl_relative_filepath.as_str(),
                                    TextureMapStorageFormat::RGB24,
                                ));

                                if let Ok(entry) =
                                    material_arena.get_mut(&current_material.unwrap())
                                {
                                    let material = &mut entry.item;

                                    material.normal_map = Some(texture_map_handle);
                                }
                            }

                            // Displacement (height) map
                            "disp" | "map_disp" => {
                                // [filepath]
                                // Example:
                                // disp cube_displacement.png

                                let mtl_relative_filepath =
                                    next_filepath(&mut line_tokens, mtl_file_path);

                                let texture_map_handle = texture_arena.insert(TextureMap::new(
                                    mtl_relative_filepath.as_str(),
                                    TextureMapStorageFormat::Index8(0),
                                ));

                                if let Ok(entry) =
                                    material_arena.get_mut(&current_material.unwrap())
                                {
                                    let material = &mut entry.item;

                                    material.displacement_map = Some(texture_map_handle);
                                }
                            }

                            //
                            // Blinn-Phong attributes
                            //

                            // Ambient color
                            "ka" => {
                                // (r,g,b)
                                // Example:
                                // Ka 0.0000 0.0000 0.0000

                                if let Ok(entry) =
                                    material_arena.get_mut(&current_material.unwrap())
                                {
                                    let material = &mut entry.item;

                                    material.ambient_color = next_rgb(&mut line_tokens);
                                }
                            }

                            // Ambient color map
                            "map_ka" => {
                                // [filepath]
                                // Example:
                                // map_Ka cube.png

                                let mtl_relative_filepath =
                                    next_filepath(&mut line_tokens, mtl_file_path);

                                let texture_map_handle = texture_arena.insert(TextureMap::new(
                                    mtl_relative_filepath.as_str(),
                                    TextureMapStorageFormat::RGB24,
                                ));

                                if let Ok(entry) =
                                    material_arena.get_mut(&current_material.unwrap())
                                {
                                    let material = &mut entry.item;

                                    material.ambient_color_map = Some(texture_map_handle);
                                }
                            }

                            // Specular exponent
                            "ns" => {
                                // [0, 1000] range
                                // Example:
                                // Ns 10.0000

                                let value = line_tokens.next().unwrap().parse::<f32>().unwrap();

                                if let Ok(entry) =
                                    material_arena.get_mut(&current_material.unwrap())
                                {
                                    let material = &mut entry.item;

                                    material.specular_exponent = value as u8;
                                }
                            }

                            // Specular exponent map
                            "map_ns" => {
                                // [filepath]
                                // Example:
                                // map_Ns cube_shininess.png

                                let mtl_relative_filepath =
                                    next_filepath(&mut line_tokens, mtl_file_path);

                                let texture_map_handle = texture_arena.insert(TextureMap::new(
                                    mtl_relative_filepath.as_str(),
                                    TextureMapStorageFormat::Index8(0),
                                ));

                                if let Ok(entry) =
                                    material_arena.get_mut(&current_material.unwrap())
                                {
                                    let material = &mut entry.item;

                                    material.specular_exponent_map = Some(texture_map_handle);
                                }
                            }

                            // Specular color
                            "ks" => {
                                // (r,g,b)
                                // Example:
                                // Ks 0.0000 0.0000 0.0000

                                if let Ok(entry) =
                                    material_arena.get_mut(&current_material.unwrap())
                                {
                                    let material = &mut entry.item;

                                    material.specular_color = next_rgb(&mut line_tokens);
                                }
                            }

                            // Specular color map
                            "map_ks" => {
                                // [filepath]
                                // Example:
                                // map_Ks cube_specular_color.png

                                let mtl_relative_filepath =
                                    next_filepath(&mut line_tokens, mtl_file_path);

                                let texture_map_handle = texture_arena.insert(TextureMap::new(
                                    mtl_relative_filepath.as_str(),
                                    TextureMapStorageFormat::RGB24,
                                ));

                                if let Ok(entry) =
                                    material_arena.get_mut(&current_material.unwrap())
                                {
                                    let material = &mut entry.item;

                                    material.specular_color_map = Some(texture_map_handle);
                                }
                            }

                            //
                            // PBR extensions.
                            //

                            // Roughness
                            "pr" => {
                                // [0.0, 1.0] range
                                // Example:
                                // Pr 0.2

                                let value = line_tokens.next().unwrap().parse::<f32>().unwrap();

                                if let Ok(entry) =
                                    material_arena.get_mut(&current_material.unwrap())
                                {
                                    let material = &mut entry.item;

                                    material.roughness = value;
                                }
                            }

                            // Roughness (map)
                            "map_pr" => {
                                // [filepath]
                                // Example:
                                // map_Pr cube_roughness.png

                                let mtl_relative_filepath =
                                    next_filepath(&mut line_tokens, mtl_file_path);

                                let texture_map_handle = texture_arena.insert(TextureMap::new(
                                    mtl_relative_filepath.as_str(),
                                    TextureMapStorageFormat::Index8(0),
                                ));

                                if let Ok(entry) =
                                    material_arena.get_mut(&current_material.unwrap())
                                {
                                    let material = &mut entry.item;

                                    material.roughness_map = Some(texture_map_handle);
                                }
                            }

                            // Metallic
                            "pm" => {
                                // [0.0, 1.0] range
                                // Example:
                                // Pm 0.0

                                let value = line_tokens.next().unwrap().parse::<f32>().unwrap();

                                if let Ok(entry) =
                                    material_arena.get_mut(&current_material.unwrap())
                                {
                                    let material = &mut entry.item;

                                    material.metallic = value;
                                }
                            }

                            // Metallic (map)
                            "map_pm" => {
                                // [filepath]
                                // Example:
                                // map_Pr cube_metallic.png

                                let mtl_relative_filepath =
                                    next_filepath(&mut line_tokens, mtl_file_path);

                                let texture_map_handle = texture_arena.insert(TextureMap::new(
                                    mtl_relative_filepath.as_str(),
                                    TextureMapStorageFormat::Index8(0),
                                ));

                                if let Ok(entry) =
                                    material_arena.get_mut(&current_material.unwrap())
                                {
                                    let material = &mut entry.item;

                                    material.metallic_map = Some(texture_map_handle);
                                }
                            }

                            // Sheen
                            "ps" => {
                                // [0.0, 1.0] range
                                // Example:
                                // Ps 0.05

                                let value = line_tokens.next().unwrap().parse::<f32>().unwrap();

                                if let Ok(entry) =
                                    material_arena.get_mut(&current_material.unwrap())
                                {
                                    let material = &mut entry.item;

                                    material.sheen = value;
                                }
                            }

                            // Sheen (map)
                            "map_ps" => {
                                // [filepath]
                                // Example:
                                // map_Pr cube_sheen.png

                                let mtl_relative_filepath =
                                    next_filepath(&mut line_tokens, mtl_file_path);

                                let texture_map_handle = texture_arena.insert(TextureMap::new(
                                    mtl_relative_filepath.as_str(),
                                    TextureMapStorageFormat::Index8(0),
                                ));

                                if let Ok(entry) =
                                    material_arena.get_mut(&current_material.unwrap())
                                {
                                    let material = &mut entry.item;

                                    material.sheen_map = Some(texture_map_handle);
                                }
                            }

                            // Clearcoat thickness
                            "pc" => {
                                // [0.0, 1.0] range
                                // Example:
                                // Pc 0.05

                                let value = line_tokens.next().unwrap().parse::<f32>().unwrap();

                                if let Ok(entry) =
                                    material_arena.get_mut(&current_material.unwrap())
                                {
                                    let material = &mut entry.item;

                                    material.clearcoat_thickness = value;
                                }
                            }

                            // Clearcoat roughness
                            "pcr" => {
                                // [0.0, 1.0] range
                                // Example:
                                // Pcr 0.05

                                let value = line_tokens.next().unwrap().parse::<f32>().unwrap();

                                if let Ok(entry) =
                                    material_arena.get_mut(&current_material.unwrap())
                                {
                                    let material = &mut entry.item;

                                    material.clearcoat_roughness = value;
                                }
                            }

                            // Anisotropy
                            "aniso" => {
                                // [0.0, 1.0] range
                                // Example:
                                // aniso 0.05

                                let value = line_tokens.next().unwrap().parse::<f32>().unwrap();

                                if let Ok(entry) =
                                    material_arena.get_mut(&current_material.unwrap())
                                {
                                    let material = &mut entry.item;

                                    material.anisotropy = value;
                                }
                            }

                            // Anisotropy rotation
                            "anisor" => {
                                // [0.0, 1.0] range
                                // Example:
                                // anisor 0.05

                                let value = line_tokens.next().unwrap().parse::<f32>().unwrap();

                                if let Ok(entry) =
                                    material_arena.get_mut(&current_material.unwrap())
                                {
                                    let material = &mut entry.item;

                                    material.anisotropy_rotation = value;
                                }
                            }

                            //
                            // Miscellaneous
                            //

                            // Index of refraction
                            "ni" => {
                                // [0.001, 10] range
                                // Example:
                                // Ni 1.5000

                                let value = line_tokens.next().unwrap().parse::<f32>().unwrap();

                                if let Ok(entry) =
                                    material_arena.get_mut(&current_material.unwrap())
                                {
                                    let material = &mut entry.item;

                                    material.index_of_refraction = value;
                                }
                            }

                            // Stencil (decal) map
                            "decal" => {
                                // [filepath]
                                // Example:
                                // decal cube_decal.png

                                let mtl_relative_filepath =
                                    next_filepath(&mut line_tokens, mtl_file_path);

                                let texture_map_handle = texture_arena.insert(TextureMap::new(
                                    mtl_relative_filepath.as_str(),
                                    TextureMapStorageFormat::RGB24,
                                ));

                                if let Ok(entry) =
                                    material_arena.get_mut(&current_material.unwrap())
                                {
                                    let material = &mut entry.item;

                                    material.decal_map = Some(texture_map_handle);
                                }
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

    println!(
        "Parsed {} material{} from \"{}\".",
        parsed_materials_count,
        if parsed_materials_count > 1 { "s" } else { "" },
        mtl_file_path_display
    );
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
