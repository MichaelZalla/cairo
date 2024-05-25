#[derive(Default, Debug, Copy, Clone)]
pub enum UISize {
    #[default]
    Null,
    Pixels(u32),
    TextContent,
    PercentOfParent(f32),
    ChildrenSum,
}

#[derive(Default, Debug, Copy, Clone)]
pub struct UISizeWithStrictness {
    size: UISize,
    strictness: f32,
}

#[derive(Default, Debug, Copy, Clone)]
pub enum UI2DAxis {
    #[default]
    X,
    Y,
}

#[derive(Default, Debug, Clone)]
pub struct UIWidget {
    semantic_sizes: [UISizeWithStrictness; 2],
}

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
