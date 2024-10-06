use std::cell::RefCell;

use cairo::mem::linked_list::LinkedList;

pub struct Command<'a> {
    pub kind: &'a String,
    pub args: &'a [String],
}

#[derive(Default, Clone)]
pub struct CommandBuffer {
    pub pending_commands: RefCell<LinkedList<String>>,
    pub executed_commands: RefCell<LinkedList<String>>,
}
