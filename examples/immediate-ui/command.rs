use std::{any::TypeId, cell::RefCell, str::FromStr};

use cairo::mem::linked_list::LinkedList;

use crate::SETTINGS;

pub struct Command<'a> {
    pub kind: &'a String,
    pub args: &'a [String],
    pub is_undo: bool,
}

#[derive(Default, Debug, Clone)]
pub struct ExecutedCommand {
    pub kind: String,
    pub prev_value: Option<String>,
    pub args: Vec<String>,
}

#[derive(Default, Clone)]
pub struct CommandBuffer {
    pub pending_commands: RefCell<LinkedList<(String, bool)>>,
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

type ProcessCommandResult = Result<Option<String>, String>;

pub(crate) fn process_command(command: Command) -> ProcessCommandResult {
    match command.kind.as_str() {
        "set_setting" => {
            let (setting_key, value_str) = (&command.args[0], &command.args[1]);

            let mut prev_value_str: Option<String> = None;

            SETTINGS.with(|settings_rc| -> Result<(), String> {
                let mut current_settings = settings_rc.borrow_mut();

                match setting_key.as_str() {
                    "clicked_count" => {
                        prev_value_str.replace(current_settings.clicked_count.to_string());

                        current_settings.clicked_count = parse_or_map_err::<usize>(value_str)?;

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

            Ok(prev_value_str)
        }
        _ => Err(format!("Unknown command kind `{}`.", command.kind).to_string()),
    }
}

pub(crate) fn process_commands(
    pending_commands: &mut LinkedList<(String, bool)>,
    executed_commands: &mut LinkedList<ExecutedCommand>,
) -> Result<(), String> {
    while let Some((cmd, is_undo)) = pending_commands.pop_front() {
        let components: Vec<String> = cmd.split(' ').map(|s| s.to_string()).collect();

        if let Some((kind, args)) = components.split_first() {
            let command = Command {
                kind,
                args,
                is_undo,
            };

            let prev_value = process_command(command)?;

            let executed_command = ExecutedCommand {
                kind: kind.to_string(),
                args: args.iter().map(|s| s.to_string()).collect(),
                prev_value,
            };

            if !is_undo {
                executed_commands.push_back(executed_command);
            }
        } else {
            println!("Unrecognized command: '{}'", cmd);
        }
    }

    Ok(())
}
