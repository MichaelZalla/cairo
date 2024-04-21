use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

pub fn read_lines(filepath: &Path) -> io::Result<io::Lines<io::BufReader<File>>> {
    let path_display = filepath.display();

    match File::open(filepath) {
        Ok(lines) => Ok(io::BufReader::new(lines).lines()),
        Err(err) => panic!("Failed to open file {}: {}", path_display, err),
    }
}
