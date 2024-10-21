use std::{any::TypeId, cell::RefCell, fmt::Display, str::FromStr};

use uuid::Uuid;

use cairo::{
    app::{
        resolution::{Resolution, RESOLUTIONS_16X9},
        window::{AppWindowingMode, APP_WINDOWING_MODES},
    },
    mem::linked_list::LinkedList,
    render::culling::{FACE_CULLING_REJECT, FACE_CULLING_WINDING_ORDER},
    resource::handle::Handle,
    scene::camera::{CameraProjectionKind, CAMERA_PROJECTION_KINDS},
    software_renderer::zbuffer::DEPTH_TEST_METHODS,
    vec::vec3::Vec3,
};

use crate::{SCENE_CONTEXT, SETTINGS};

pub struct Command<'a> {
    pub kind: &'a String,
    pub args: &'a [String],
    pub is_undo: bool,
}

impl<'a> Display for Command<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {}{}",
            self.kind,
            self.args.join(" "),
            if self.is_undo { " (undo)" } else { "" }
        )
    }
}

pub type PendingCommand = (String, bool);

#[derive(Default, Debug, Clone)]
pub struct ExecutedCommand {
    pub kind: String,
    pub prev_value: Option<String>,
    pub args: Vec<String>,
}

#[derive(Default, Clone)]
pub struct CommandBuffer {
    pub pending_commands: RefCell<LinkedList<PendingCommand>>,
    pub executed_commands: RefCell<LinkedList<ExecutedCommand>>,
}

fn parse_or_map_err<T: 'static + FromStr>(arg: &String) -> Result<T, String> {
    arg.parse::<T>().map_err(|_err| {
        format!(
            "Failed to parse a `{:?}` from argument string '{}'.",
            TypeId::of::<T>(),
            arg
        )
        .to_string()
    })
}

type ProcessCommandResult =
    Result<(Option<String>, Option<Resolution>, Option<AppWindowingMode>), String>;

fn process_command(command: Command) -> ProcessCommandResult {
    match command.kind.as_str() {
        "set" => {
            let (setting_key, value_str) = (&command.args[0], &command.args[1]);

            let mut prev_value_str: Option<String> = None;
            let mut new_resolution: Option<Resolution> = None;
            let mut new_windowing_mode: Option<AppWindowingMode> = None;

            SETTINGS.with(|settings_rc| -> Result<(), String> {
                let mut current_settings = settings_rc.borrow_mut();

                match setting_key.as_str() {
                    "windowing_mode" => {
                        let value = parse_or_map_err::<usize>(value_str)?;

                        let requested_mode = APP_WINDOWING_MODES[value];

                        if requested_mode != current_settings.windowing_mode {
                            prev_value_str
                                .replace((current_settings.windowing_mode as usize).to_string());

                            current_settings.windowing_mode = requested_mode;

                            new_windowing_mode.replace(current_settings.windowing_mode);
                        }

                        Ok(())
                    }
                    "resolution" => {
                        let value = parse_or_map_err::<usize>(value_str)?;

                        let requested_resolution = value;

                        if requested_resolution != current_settings.resolution {
                            prev_value_str.replace(current_settings.resolution.to_string());

                            current_settings.resolution = requested_resolution;

                            new_resolution.replace(RESOLUTIONS_16X9[requested_resolution]);
                        }

                        Ok(())
                    }
                    "brightness" => {
                        prev_value_str.replace(current_settings.brightness.to_string());

                        current_settings.brightness = parse_or_map_err::<f32>(value_str)?;

                        Ok(())
                    }
                    "gamma" => {
                        prev_value_str.replace(current_settings.gamma.to_string());

                        current_settings.gamma = parse_or_map_err::<f32>(value_str)?;

                        Ok(())
                    }
                    "vsync" => {
                        prev_value_str.replace(current_settings.vsync.to_string());

                        current_settings.vsync = parse_or_map_err::<bool>(value_str)?;

                        Ok(())
                    }
                    "hdr" => {
                        prev_value_str.replace(current_settings.hdr.to_string());

                        current_settings.hdr = parse_or_map_err::<bool>(value_str)?;

                        Ok(())
                    }
                    "render.fragment_shader" => {
                        prev_value_str.replace(current_settings.fragment_shader.to_string());

                        current_settings.fragment_shader = parse_or_map_err::<usize>(value_str)?;

                        Ok(())
                    }
                    "render_options.do_rasterization" => {
                        prev_value_str
                            .replace(current_settings.render_options.do_rasterization.to_string());

                        current_settings.render_options.do_rasterization =
                            parse_or_map_err::<bool>(value_str)?;

                        Ok(())
                    }
                    "render_options.rasterizer_options.face_culling_strategy.winding_order" => {
                        prev_value_str.replace(
                            current_settings
                                .render_options
                                .rasterizer_options
                                .face_culling_strategy
                                .winding_order
                                .to_string(),
                        );

                        let new_index = parse_or_map_err::<usize>(value_str)?;

                        current_settings
                            .render_options
                            .rasterizer_options
                            .face_culling_strategy
                            .winding_order = FACE_CULLING_WINDING_ORDER[new_index];

                        Ok(())
                    }
                    "render_options.rasterizer_options.face_culling_strategy.reject" => {
                        prev_value_str.replace(
                            current_settings
                                .render_options
                                .rasterizer_options
                                .face_culling_strategy
                                .reject
                                .to_string(),
                        );

                        let new_index = parse_or_map_err::<usize>(value_str)?;

                        current_settings
                            .render_options
                            .rasterizer_options
                            .face_culling_strategy
                            .reject = FACE_CULLING_REJECT[new_index];

                        Ok(())
                    }
                    "render_options.do_lighting" => {
                        prev_value_str
                            .replace(current_settings.render_options.do_lighting.to_string());

                        current_settings.render_options.do_lighting =
                            parse_or_map_err::<bool>(value_str)?;

                        Ok(())
                    }
                    "render_options.do_deferred_lighting" => {
                        prev_value_str.replace(
                            current_settings
                                .render_options
                                .do_deferred_lighting
                                .to_string(),
                        );

                        current_settings.render_options.do_deferred_lighting =
                            parse_or_map_err::<bool>(value_str)?;

                        Ok(())
                    }
                    "render_options.do_bloom" => {
                        prev_value_str
                            .replace(current_settings.render_options.do_bloom.to_string());

                        current_settings.render_options.do_bloom =
                            parse_or_map_err::<bool>(value_str)?;

                        Ok(())
                    }
                    "render_options.draw_wireframe" => {
                        prev_value_str
                            .replace(current_settings.render_options.draw_wireframe.to_string());

                        current_settings.render_options.draw_wireframe =
                            parse_or_map_err::<bool>(value_str)?;

                        Ok(())
                    }
                    "render_options.wireframe_color" => {
                        prev_value_str
                            .replace(current_settings.render_options.wireframe_color.to_string());

                        let wireframe_color = parse_or_map_err::<Vec3>(value_str)?;

                        current_settings.render_options.wireframe_color = wireframe_color;

                        Ok(())
                    }
                    "render_options.draw_normals" => {
                        prev_value_str
                            .replace(current_settings.render_options.draw_normals.to_string());

                        current_settings.render_options.draw_normals =
                            parse_or_map_err::<bool>(value_str)?;

                        Ok(())
                    }
                    "render_options.draw_normals_scale" => {
                        prev_value_str.replace(
                            current_settings
                                .render_options
                                .draw_normals_scale
                                .to_string(),
                        );

                        current_settings.render_options.draw_normals_scale =
                            parse_or_map_err::<f32>(value_str)?;

                        Ok(())
                    }
                    "render.tone_mapping" => {
                        prev_value_str.replace(current_settings.tone_mapping.to_string());

                        current_settings.tone_mapping = parse_or_map_err::<usize>(value_str)?;

                        Ok(())
                    }
                    "depth_test_method" => {
                        prev_value_str.replace(current_settings.depth_test_method.to_string());

                        let new_index = parse_or_map_err::<usize>(value_str)?;

                        current_settings.depth_test_method = DEPTH_TEST_METHODS[new_index];

                        Ok(())
                    }
                    "shader_options.texture_filtering" => {
                        prev_value_str
                            .replace(current_settings.shader_options.bilinear_active.to_string());

                        current_settings.shader_options.bilinear_active = false;
                        current_settings.shader_options.trilinear_active = false;

                        let selected_index = parse_or_map_err::<usize>(value_str)?;

                        match selected_index {
                            0 => {}
                            1 => {
                                current_settings.shader_options.bilinear_active = true;
                            }
                            2 => {
                                current_settings.shader_options.trilinear_active = true;
                            }
                            _ => (),
                        }

                        Ok(())
                    }
                    "shader_options.albedo_color_maps" => {
                        prev_value_str.replace(
                            current_settings
                                .shader_options
                                .albedo_mapping_active
                                .to_string(),
                        );

                        current_settings.shader_options.albedo_mapping_active =
                            parse_or_map_err::<bool>(value_str)?;

                        Ok(())
                    }
                    "shader_options.ambient_occlusion_maps" => {
                        prev_value_str.replace(
                            current_settings
                                .shader_options
                                .ambient_occlusion_mapping_active
                                .to_string(),
                        );

                        current_settings
                            .shader_options
                            .ambient_occlusion_mapping_active =
                            parse_or_map_err::<bool>(value_str)?;

                        Ok(())
                    }
                    "shader_options.roughness_maps" => {
                        prev_value_str.replace(
                            current_settings
                                .shader_options
                                .roughness_mapping_active
                                .to_string(),
                        );

                        current_settings.shader_options.roughness_mapping_active =
                            parse_or_map_err::<bool>(value_str)?;

                        Ok(())
                    }
                    "shader_options.metallic_maps" => {
                        prev_value_str.replace(
                            current_settings
                                .shader_options
                                .metallic_mapping_active
                                .to_string(),
                        );

                        current_settings.shader_options.metallic_mapping_active =
                            parse_or_map_err::<bool>(value_str)?;

                        Ok(())
                    }
                    "shader_options.normal_maps" => {
                        prev_value_str.replace(
                            current_settings
                                .shader_options
                                .normal_mapping_active
                                .to_string(),
                        );

                        current_settings.shader_options.normal_mapping_active =
                            parse_or_map_err::<bool>(value_str)?;

                        Ok(())
                    }
                    "shader_options.displacement_maps" => {
                        prev_value_str.replace(
                            current_settings
                                .shader_options
                                .displacement_mapping_active
                                .to_string(),
                        );

                        current_settings.shader_options.displacement_mapping_active =
                            parse_or_map_err::<bool>(value_str)?;

                        Ok(())
                    }
                    "shader_options.specular_maps" => {
                        prev_value_str.replace(
                            current_settings
                                .shader_options
                                .specular_exponent_mapping_active
                                .to_string(),
                        );

                        current_settings
                            .shader_options
                            .specular_exponent_mapping_active =
                            parse_or_map_err::<bool>(value_str)?;

                        Ok(())
                    }
                    "shader_options.emissive_maps" => {
                        prev_value_str.replace(
                            current_settings
                                .shader_options
                                .emissive_color_mapping_active
                                .to_string(),
                        );

                        current_settings
                            .shader_options
                            .emissive_color_mapping_active = parse_or_map_err::<bool>(value_str)?;

                        Ok(())
                    }
                    "postprocessing.effects.outline" => {
                        prev_value_str.replace(current_settings.effects.outline.to_string());

                        current_settings.effects.outline = parse_or_map_err::<bool>(value_str)?;

                        Ok(())
                    }
                    "postprocessing.effects.invert" => {
                        prev_value_str.replace(current_settings.effects.invert.to_string());

                        current_settings.effects.invert = parse_or_map_err::<bool>(value_str)?;

                        Ok(())
                    }
                    "postprocessing.effects.grayscale" => {
                        prev_value_str.replace(current_settings.effects.grayscale.to_string());

                        current_settings.effects.grayscale = parse_or_map_err::<bool>(value_str)?;

                        Ok(())
                    }
                    "postprocessing.effects.sharpen_kernel" => {
                        prev_value_str.replace(current_settings.effects.sharpen_kernel.to_string());

                        current_settings.effects.sharpen_kernel =
                            parse_or_map_err::<bool>(value_str)?;

                        Ok(())
                    }
                    "postprocessing.effects.blur_kernel" => {
                        prev_value_str.replace(current_settings.effects.blur_kernel.to_string());

                        current_settings.effects.blur_kernel = parse_or_map_err::<bool>(value_str)?;

                        Ok(())
                    }
                    "postprocessing.effects.edge_detection_kernel" => {
                        prev_value_str
                            .replace(current_settings.effects.edge_detection_kernel.to_string());

                        current_settings.effects.edge_detection_kernel =
                            parse_or_map_err::<bool>(value_str)?;

                        Ok(())
                    }
                    "camera" => {
                        debug_assert!(command.args.len() >= 5);

                        let uuid = parse_or_map_err::<Uuid>(&command.args[1])?;
                        let index = parse_or_map_err::<usize>(&command.args[2])?;

                        SCENE_CONTEXT.with(|ctx| -> Result<(), String> {
                            let resources = ctx.resources.borrow();

                            let mut camera_arena = resources.camera.borrow_mut();

                            let camera_handle = Handle { index, uuid };

                            match camera_arena.get_mut(&camera_handle) {
                                Ok(entry) => {
                                    let camera = &mut entry.item;

                                    let attribute = &command.args[3];

                                    match attribute.as_str() {
                                        "kind" => {
                                            let new_kind_index =
                                                parse_or_map_err::<usize>(&command.args[4])?;

                                            let new_kind = CAMERA_PROJECTION_KINDS[new_kind_index];

                                            match new_kind {
                                                CameraProjectionKind::Perspective => {
                                                    // TODO
                                                    todo!()
                                                }
                                                CameraProjectionKind::Orthographic => {
                                                    // TODO
                                                    todo!()
                                                }
                                            }
                                        }
                                        "perspective.field_of_view" => {
                                            let new_fov =
                                                parse_or_map_err::<f32>(&command.args[4])?;

                                            camera.set_field_of_view(Some(new_fov));
                                        }
                                        "projection_z_near" => {
                                            let new_z_near =
                                                parse_or_map_err::<f32>(&command.args[4])?;

                                            camera.set_projection_z_near(
                                                new_z_near.min(camera.get_projection_z_far()),
                                            );
                                        }
                                        "projection_z_far" => {
                                            let new_z_far =
                                                parse_or_map_err::<f32>(&command.args[4])?;

                                            camera.set_projection_z_far(
                                                new_z_far.max(camera.get_projection_z_near()),
                                            );
                                        }
                                        _ => {
                                            println!(
                                                "Unrecognized camera attribute '{}'!",
                                                attribute
                                            );
                                        }
                                    }

                                    Ok(())
                                }
                                Err(_) => Err(format!(
                                    "Camera not found for Handle {}!",
                                    camera_handle.uuid
                                )
                                .to_string()),
                            }
                        })
                    }
                    _ => {
                        println!("Unknown settings key `{}`.", setting_key);

                        Ok(())
                    }
                }
            })?;

            Ok((prev_value_str, new_resolution, new_windowing_mode))
        }
        _ => Err(format!("Unknown command kind `{}`.", command.kind).to_string()),
    }
}

type ProcessCommandsResult = Result<(Option<Resolution>, Option<AppWindowingMode>), String>;

pub(crate) fn process_commands(
    pending_commands: &mut LinkedList<PendingCommand>,
    executed_commands: &mut LinkedList<ExecutedCommand>,
) -> ProcessCommandsResult {
    let mut result: (Option<Resolution>, Option<AppWindowingMode>) = (None, None);

    while let Some((cmd, is_undo)) = pending_commands.pop_front() {
        let components: Vec<String> = cmd.split(' ').map(|s| s.to_string()).collect();

        if let Some((kind, args)) = components.split_first() {
            let command = Command {
                kind,
                args,
                is_undo,
            };

            let process_command_result = process_command(command)?;

            let prev_value = process_command_result.0;

            let executed_command = ExecutedCommand {
                kind: kind.to_string(),
                args: args.iter().map(|s| s.to_string()).collect(),
                prev_value,
            };

            match process_command_result {
                (_, Some(new_resolution), None) => {
                    result.0.replace(new_resolution);
                }
                (_, None, Some(new_windowing_mode)) => {
                    result.1.replace(new_windowing_mode);
                }
                (_, Some(new_resolution), Some(new_windowing_mode)) => {
                    result.0.replace(new_resolution);
                    result.1.replace(new_windowing_mode);
                }
                _ => (),
            }

            if !is_undo {
                executed_commands.push_back(executed_command);
            }
        } else {
            println!("Unrecognized command: '{}'", cmd);
        }
    }

    Ok(result)
}
