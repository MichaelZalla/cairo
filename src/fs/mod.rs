use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

pub fn get_absolute_filepath(filepath: &str) -> String {
    if filepath.len() == 0
        || filepath.chars().nth(0).unwrap() == '/'
        || filepath.split(':').count() > 1
    {
        return filepath.to_string();
    }

    let root_directory: String = String::from(env!("CARGO_MANIFEST_DIR"));

    return format!("{}{}", root_directory, filepath).to_string();
}

pub fn read_lines<P>(filepath: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filepath)?;

    Ok(io::BufReader::new(file).lines())
}
