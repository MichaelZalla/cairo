use std::{cell::RefCell, rc::Rc};

use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Node<'a, T> {
    pub data: T,
    pub parent: Option<Rc<RefCell<Node<'a, T>>>>,
    pub children: Vec<Rc<RefCell<Node<'a, T>>>>,
}

impl<'a, T> Node<'a, T>
where
    T: Default + Clone + Serialize + Deserialize<'a>,
{
    pub fn new(data: T) -> Self {
        Self {
            data,
            ..Default::default()
        }
    }
}
