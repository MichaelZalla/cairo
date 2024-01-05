use std::{path::Path, str::SplitWhitespace};

use crate::{fs::read_lines, image::TextureMap, mesh::MaterialSource, vec::vec3::Vec3};

use super::Material;

pub fn load_mtl(filepath: &str) -> Vec<Material> {
    let mtl_file_path = Path::new(&filepath);
    let mtl_file_path_display = mtl_file_path.display();

    let lines = match read_lines(&mtl_file_path) {
        Err(why) => panic!("Failed to open file {}: {}", mtl_file_path_display, why),
        Ok(lines) => lines,
    };

    let mut materials: Vec<Material> = vec![];

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

                                let source = MaterialSource {
                                    filepath: mtl_file_path_display.to_string(),
                                };

                                let name = line_tokens.next().unwrap().to_string();

                                let mut mat = Material::new(name);

                                mat.material_source = Some(source);

                                materials.push(mat);
                            }

                            // Illumination model
                            "illum" => {
                                // [0, 10] range
                                // Example:
                                // illum 2
                                let value = line_tokens.next().unwrap().parse::<u8>().unwrap();

                                materials.last_mut().unwrap().illumination_model = value;
                            }

                            // Ambient color
                            "ka" => {
                                // R G B
                                // Example:
                                // Ka 0.0000 0.0000 0.0000

                                materials.last_mut().unwrap().ambient_color =
                                    next_rgb(&mut line_tokens);
                            }

                            // Diffuse color
                            "kd" => {
                                // R G B
                                // Example:
                                // Kd 0.5880 0.5880 0.5880

                                materials.last_mut().unwrap().diffuse_color =
                                    next_rgb(&mut line_tokens);
                            }

                            // Specular color
                            "ks" => {
                                // R G B
                                // Example:
                                // Ks 0.0000 0.0000 0.0000

                                materials.last_mut().unwrap().specular_color =
                                    next_rgb(&mut line_tokens);
                            }

                            // Specular exponent
                            "ns" => {
                                // [0, 1000] range
                                // Example:
                                // Ns 10.0000

                                let value = line_tokens.next().unwrap().parse::<f32>().unwrap();

                                materials.last_mut().unwrap().specular_exponent = value as i32;
                            }

                            // Emissive color
                            "ke" => {
                                // R G B
                                // Example:
                                // Ke 0.0000 0.0000 0.0000

                                materials.last_mut().unwrap().emissive_color =
                                    next_rgb(&mut line_tokens);
                            }

                            // Dissolve (opaqueness)
                            "d" => {
                                // [0, 1] range
                                // Example:
                                // d 1.0000

                                let value = line_tokens.next().unwrap().parse::<f32>().unwrap();

                                materials.last_mut().unwrap().dissolve = value;
                            }

                            // Transparency
                            "tr" => {
                                // [0, 1] range
                                // Example:
                                // Tr 0.0000

                                let value = line_tokens.next().unwrap().parse::<f32>().unwrap();

                                materials.last_mut().unwrap().transparency = value;
                            }

                            // Transmission filter color
                            "tf" => {
                                // R G B
                                // Example:
                                // Tf 1.0000 1.0000 1.0000

                                materials.last_mut().unwrap().transmission_filter_color =
                                    next_rgb(&mut line_tokens);
                            }

                            // Index of refraction
                            "ni" => {
                                // [0.001, 10] range
                                // Example:
                                // Ni 1.5000

                                let value = line_tokens.next().unwrap().parse::<f32>().unwrap();

                                materials.last_mut().unwrap().index_of_refraction = value;
                            }

                            // Ambient texture map
                            "map_ka" => {
                                // [filepath]
                                // Example:
                                // map_Ka cube.png

                                let filepath = line_tokens.next().unwrap().to_string();

                                let mtl_relative_filepath = mtl_file_path
                                    .parent()
                                    .unwrap()
                                    .join(filepath)
                                    .into_os_string()
                                    .into_string()
                                    .unwrap();

                                materials.last_mut().unwrap().ambient_map =
                                    Some(TextureMap::new(&mtl_relative_filepath.as_str()));
                            }

                            // Diffuse texture map (typically identical to map_Ka)
                            "map_kd" => {
                                // [filepath]
                                // Example:
                                // map_Kd cube.png

                                let filepath = line_tokens.next().unwrap().to_string();

                                let mtl_relative_filepath = mtl_file_path
                                    .parent()
                                    .unwrap()
                                    .join(filepath)
                                    .into_os_string()
                                    .into_string()
                                    .unwrap();

                                materials.last_mut().unwrap().diffuse_map =
                                    Some(TextureMap::new(&mtl_relative_filepath.as_str()));
                            }

                            // Specular color map
                            "map_ks" => {
                                println!("@TODO Implementation for \"{}\".", "map_Ks");
                            }

                            // Alpha map
                            "map_d" => {
                                println!("@TODO Implementation for \"{}\".", "map_d");
                            }

                            // Bump map
                            "map_bump" | "bump" => {
                                println!("@TODO Implementation for \"{}\".", "map_Disp");
                            }

                            // Displacement map
                            "map_disp" | "disp" => {
                                println!("@TODO Implementation for \"{}\".", "map_Disp");
                            }

                            // Stencil (decal) map
                            "decal" => {
                                println!("@TODO Implementation for \"{}\".", "decal");
                            }

                            // Unrecognized prefix
                            other => {
                                println!("{}", other)
                            }
                        }
                    }
                }
            }
        }
    }

    let count = materials.len();

    println!(
        "Parsed {} material{} from \"{}\".",
        count,
        if count > 1 { "s" } else { "" },
        mtl_file_path_display
    );

    println!();
    for mat in &materials {
        println!("{}", mat);
    }

    return materials;
}

fn next_rgb<'a>(line_tokens: &mut SplitWhitespace<'a>) -> Vec3 {
    let r = line_tokens.next().unwrap().parse::<f32>().unwrap();
    let g = line_tokens.next().unwrap().parse::<f32>().unwrap();
    let b = line_tokens.next().unwrap().parse::<f32>().unwrap();

    return Vec3 { x: r, y: g, z: b };
}
