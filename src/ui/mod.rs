#[derive(Default, Debug, Copy, Clone)]
pub struct UILayout {}

#[derive(Default, Debug, Clone)]
pub struct UILayoutStack {
    stack: Vec<UILayout>,
}

impl UILayoutStack {
    pub fn push(&mut self, layout: UILayout) {
        self.stack.push(layout)
    }

    pub fn pop(&mut self) -> Option<UILayout> {
        self.stack.pop()
    }
}

pub struct UILayoutContext {
    pub layouts: UILayoutStack,
}

impl UILayoutContext {}
