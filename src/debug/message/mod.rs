#[derive(Default, Debug, Clone)]
pub struct DebugMessageBuffer {
    current_line: usize,
    lines: Vec<String>,
}

impl DebugMessageBuffer {
    pub fn write(&mut self, line: String) {
        self.lines.push(line)
    }

    pub fn drain(&mut self) {
        self.lines.clear();

        self.current_line = 0;
    }
}

impl Iterator for DebugMessageBuffer {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        if self.lines.is_empty() {
            return None;
        }

        if self.current_line < self.lines.len() {
            let line = &self.lines[self.current_line];

            self.current_line += 1;

            return Some(line.clone());
        }

        None
    }
}
