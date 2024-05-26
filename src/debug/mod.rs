use crate::debug_print;

pub mod message;

pub fn println_indent(depth: usize, msg: String) {
    let indent = 2 * (depth + 1);

    debug_print!("{:indent$}{}", ">", msg);
}
