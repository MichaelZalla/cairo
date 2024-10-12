use std::{any::TypeId, cell::RefCell, str::FromStr};

use cairo::mem::linked_list::LinkedList;

use crate::SETTINGS;

pub struct Command<'a> {
    pub kind: &'a String,
    pub args: &'a [String],
}

#[derive(Default, Clone)]
pub struct CommandBuffer {
    pub pending_commands: RefCell<LinkedList<String>>,
    pub executed_commands: RefCell<LinkedList<String>>,
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

pub(crate) fn process_command(command: Command) -> Result<(), String> {
    match command.kind.as_str() {
        "set_setting" => {
            let (setting_key, value_str) = (&command.args[0], &command.args[1]);

            SETTINGS.with(|settings| -> Result<(), String> {
                let mut current_settings = settings.borrow_mut();

                match setting_key.as_str() {
                    "clicked_count" => {
                        current_settings.clicked_count = parse_or_map_err::<usize>(value_str)?;

                        Ok(())
                    }
                    _ => Err(format!("Unknown settings key `{}`.", setting_key).to_string()),
                }
            })
        }
        _ => Err(format!("Unknown command kind `{}`.", command.kind).to_string()),
    }
}

pub(crate) fn process_commands(
    pending_commands: &mut LinkedList<String>,
    executed_commands: &mut LinkedList<String>,
) -> Result<(), String> {
    while let Some(cmd) = pending_commands.pop_front() {
        let components: Vec<String> = cmd.split(' ').map(|s| s.to_string()).collect();

        if let Some((kind, args)) = components.split_first() {
            process_command(Command { kind, args })?;
        }

        executed_commands.push_back(cmd);
    }

    Ok(())
}
