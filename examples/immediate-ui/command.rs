use std::cell::RefCell;

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

pub(crate) fn process_command(command: Command) -> Result<(), String> {
    if command.kind == "set_setting" {
        let (setting_key, new_value) = (&command.args[0], &command.args[1]);

        if setting_key == "clicked_count" {
            SETTINGS.with(|settings| {
                *settings.clicked_count.borrow_mut() = new_value.parse::<usize>().unwrap();
            });
        }
    }

    Ok(())
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
