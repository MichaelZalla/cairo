pub mod message;

pub fn println_indent(depth: usize, msg: String) {
    let indent = 2 * (depth + 1);

    println!("{:indent$}{}", ">", msg);
}
