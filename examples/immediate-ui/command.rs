use std::{any::TypeId, cell::RefCell, str::FromStr};

use cairo::{
    app::{
        resolution::{Resolution, RESOLUTIONS_16X9},
        window::{AppWindowingMode, APP_WINDOWING_MODES},
    },
    mem::linked_list::LinkedList,
};

use crate::SETTINGS;

pub struct Command<'a> {
    pub kind: &'a String,
    pub args: &'a [String],
    pub is_undo: bool,
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
                    "bloom" => {
                        prev_value_str.replace(current_settings.bloom.to_string());

                        current_settings.bloom = parse_or_map_err::<bool>(value_str)?;

                        Ok(())
                    }
                    _ => Err(format!("Unknown settings key `{}`.", setting_key).to_string()),
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
