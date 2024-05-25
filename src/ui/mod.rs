#[derive(Default, Debug, Copy, Clone)]
pub struct UIWidget {}

#[derive(Default, Debug, Clone)]
pub struct UIWidgetStack {
    stack: Vec<UIWidget>,
}

impl UIWidgetStack {
    pub fn push(&mut self, widget: UIWidget) {
        self.stack.push(widget)
    }

    pub fn pop(&mut self) -> Option<UIWidget> {
        self.stack.pop()
    }
}

pub struct UIContext {
    pub stack: UIWidgetStack,
}

impl UIContext {}
