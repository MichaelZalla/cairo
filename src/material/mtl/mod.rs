use std::{path::Path, str::SplitWhitespace};

use crate::{fs::read_lines, image::TextureMap, vec::vec3::Vec3};

use super::Material;

pub fn load_mtl(filepath: &str) -> Vec<Material> {
    let path = Path::new(&filepath);
    let path_display = path.display();

    let lines = match read_lines(&path) {
        Err(why) => panic!("Failed to open file {}: {}", path_display, why),
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
                        match first {
                            // Comment
                            "#" => (),

                            // Material entry
                            "newmtl" => {
                                // Example:
                                // newmtl cube

                                let name = line_tokens.next().unwrap().to_string();

                                materials.push(Material::new(name));
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
                            "Ka" => {
                                // R G B
                                // Example:
                                // Ka 0.0000 0.0000 0.0000

                                materials.last_mut().unwrap().ambient_color =
                                    next_rgb(&mut line_tokens);
                            }

                            // Diffuse color
                            "Kd" => {
                                // R G B
                                // Example:
                                // Kd 0.5880 0.5880 0.5880

                                materials.last_mut().unwrap().diffuse_color =
                                    next_rgb(&mut line_tokens);
                            }

                            // Specular color
                            "Ks" => {
                                // R G B
                                // Example:
                                // Ks 0.0000 0.0000 0.0000

                                materials.last_mut().unwrap().specular_color =
                                    next_rgb(&mut line_tokens);
                            }

                            // Specular exponent
                            "Ns" => {
                                // [0, 1000] range
                                // Example:
                                // Ns 10.0000

                                let value = line_tokens.next().unwrap().parse::<f32>().unwrap();

                                materials.last_mut().unwrap().specular_exponent = value;
                            }

                            // Emissive color
                            "Ke" => {
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
                            "Tr" => {
                                // [0, 1] range
                                // Example:
                                // Tr 0.0000

                                let value = line_tokens.next().unwrap().parse::<f32>().unwrap();

                                materials.last_mut().unwrap().transparency = value;
                            }

                            // Transmission filter color
                            "Tf" => {
                                // R G B
                                // Example:
                                // Tf 1.0000 1.0000 1.0000

                                materials.last_mut().unwrap().transmission_filter_color =
                                    next_rgb(&mut line_tokens);
                            }

                            // Index of refraction
                            "Ni" => {
                                // [0.001, 10] range
                                // Example:
                                // Ni 1.5000

                                let value = line_tokens.next().unwrap().parse::<f32>().unwrap();

                                materials.last_mut().unwrap().index_of_refraction = value;
                            }

                            // Ambient texture map
                            "map_Ka" => {
                                // [filepath]
                                // Example:
                                // map_Ka cube.png

                                let filepath = line_tokens.next().unwrap().to_string();

                                materials.last_mut().unwrap().ambient_map =
                                    Some(TextureMap::new(filepath));
                            }

                            // Diffuse texture map (typically identical to map_Ka)
                            "map_Kd" => {
                                // [filepath]
                                // Example:
                                // map_Kd cube.png

                                let filepath = line_tokens.next().unwrap().to_string();

                                materials.last_mut().unwrap().diffuse_map =
                                    Some(TextureMap::new(filepath));
                            }

                            // Specular color map
                            "map_Ks" => (),

                            // Alpha map
                            "map_d" => (),

                            // Bump map
                            "map_bump" => (),

                            // Bump map (variant)
                            "bump" => (),

                            // Displacement map
                            "disp" => (),

                            // Stencil (decal) map
                            "decal" => (),

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
        path_display
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
