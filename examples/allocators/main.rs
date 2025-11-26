use cairo::mem::arena::{Arena, stack::FixedStackArena};

fn main() -> Result<(), String> {
    println!("Hello, arena!");

    let stack = match FixedStackArena::new(1024, 1) {
        Ok(stack) => stack,
        Err(err) => panic!("{}", err.to_string()),
    };

    println!("Arena capacity is {}.", stack.capacity());

    Ok(())
}
