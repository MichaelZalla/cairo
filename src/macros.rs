#[macro_export]
#[cfg(feature = "debug_print_feature")]
macro_rules! debug_print {
    ($( $args:expr ),*) => { println!( $( $args ),* ); }
}

#[macro_export]
#[cfg(not(feature = "debug_print_feature"))]
macro_rules! debug_print {
    ($( $args:expr ),*) => {}
}
