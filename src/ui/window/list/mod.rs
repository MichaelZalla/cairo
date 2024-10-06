use crate::mem::linked_list::LinkedList;

use super::Window;

#[derive(Default, Debug, Clone)]
pub struct WindowList<'a>(pub LinkedList<Window<'a>>);
