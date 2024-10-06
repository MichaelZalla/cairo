use std::cell::RefCell;

#[derive(Default, Debug, Clone)]
pub(crate) struct Settings {
    pub clicked_count: RefCell<usize>,
}
