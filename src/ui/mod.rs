use std::{cell::RefCell, rc::Rc};

use serde::{Deserialize, Serialize};

use crate::vec::vec2::Vec2;

#[derive(Default, Debug, Copy, Clone, Serialize, Deserialize)]
pub enum UISize {
    #[default]
    Null,
    Pixels(u32),
    TextContent,
    PercentOfParent(f32),
    ChildrenSum,
}

#[derive(Default, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct UISizeWithStrictness {
    pub size: UISize,
    pub strictness: f32,
}

#[derive(Default, Debug, Copy, Clone)]
pub enum UI2DAxis {
    #[default]
    X,
    Y,
}

const UI_2D_AXIS_COUNT: usize = 2;

// An immediate-mode data structure, doubling as a cache entry for persistent
// UIWidgets across frames; computed fields from the previous frame as used to
// interpret user inputs, while computed fields from the current frame are used
// for widget rendering.
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct UIWidget {
    pub id: String,

    // Auto-layout inputs
    pub semantic_sizes: [UISizeWithStrictness; UI_2D_AXIS_COUNT],

    // Auto-layout outputs
    #[serde(skip)]
    computed_relative_position: [f32; UI_2D_AXIS_COUNT], // Position relative to parent, in pixels.

    #[serde(skip)]
    computed_size: [f32; UI_2D_AXIS_COUNT], // Size in pixels.

    #[serde(skip)]
    rect: [Vec2; 2], // On-screen rectangle coordinates, in pixels.
}

impl UIWidget {
    pub fn new(id: String, semantic_sizes: [UISizeWithStrictness; UI_2D_AXIS_COUNT]) -> Self {
        Self {
            id,
            semantic_sizes,
            ..Default::default()
        }
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct UIWidgetNode<'a> {
    pub widget: UIWidget,
    parent: Option<Rc<RefCell<UIWidgetNode<'a>>>>,
    children: Vec<Rc<RefCell<UIWidgetNode<'a>>>>,
}

impl<'a> UIWidgetNode<'a> {
    pub fn new(widget: UIWidget) -> Self {
        Self {
            widget,
            ..Default::default()
        }
    }
}

pub struct UIWidgetTree<'a> {
    current: Option<Rc<RefCell<UIWidgetNode<'a>>>>,
    root: Rc<RefCell<UIWidgetNode<'a>>>,
}

impl<'a> UIWidgetTree<'a> {
    pub fn new(mut root: UIWidgetNode<'a>) -> Self {
        root.parent = None;
        root.children = vec![];

        let root_rc = Rc::new(RefCell::new(root));

        Self {
            root: root_rc.clone(),
            current: Some(root_rc),
        }
    }
}

impl<'a> UIWidgetTree<'a> {
    pub fn push(&mut self, widget: UIWidget) {
        let new_child_node_rc: Rc<RefCell<UIWidgetNode<'a>>>;

        if let Some(current_node_rc) = &self.current {
            let mut current_node = (*current_node_rc).borrow_mut();

            let new_child_node = UIWidgetNode {
                widget,
                parent: self.current.clone(),
                children: vec![],
            };

            new_child_node_rc = Rc::new(RefCell::new(new_child_node));

            current_node.children.push(new_child_node_rc.clone());
        } else {
            panic!("Called UIWidgetTree::push() on a tree with no `current` value!");
        }

        self.current = Some(new_child_node_rc.clone());
    }

    pub fn pop(&mut self) -> Option<Rc<RefCell<UIWidgetNode<'a>>>> {
        let (child_node_rc, parent_node_rc) = {
            match &self.current {
                Some(current_node_rc) => {
                    let current_node = current_node_rc.borrow();

                    match &current_node.parent {
                        Some(parent_node_rc) => {
                            (Some(current_node_rc.clone()), Some(parent_node_rc.clone()))
                        }
                        None => (Some(current_node_rc.clone()), None),
                    }
                }
                None => (None, None),
            }
        };

        let removed_child_node_rc = match (&child_node_rc, &parent_node_rc) {
            (Some(child_node_rc), Some(parent_node_rc)) => {
                let child_node = (*child_node_rc).borrow_mut();
                let mut parent_node = (*parent_node_rc).borrow_mut();

                // Remove child from parent.

                if let Some(child_index) = get_child_index(&parent_node, &child_node) {
                    let removed_child_node = parent_node.children.swap_remove(child_index);

                    removed_child_node.borrow_mut().parent = None;
                    removed_child_node.borrow_mut().children = vec![];

                    removed_child_node
                } else {
                    panic!("Failed to find child index!");
                }
            }
            (Some(_child_node_rc), None) => {
                panic!("Called UIWidgetTree::pop() on the root of the tree!");
            }
            _ => {
                panic!("Called UIWidgetTree::pop() on an empty tree!");
            }
        };

        // Set `self.current` to parent.

        self.current = parent_node_rc.clone();

        // Return child.

        Some(removed_child_node_rc)
    }
}

fn get_child_index(parent: &UIWidgetNode, child: &UIWidgetNode) -> Option<usize> {
    let mut child_index: isize = -1;

    for (i, other_child) in parent.children.iter().enumerate() {
        let lhs = (*other_child).borrow();
        let rhs = child;

        if lhs.widget.id == rhs.widget.id {
            child_index = i as isize;
        }
    }

    if child_index > -1 {
        Some(child_index as usize)
    } else {
        None
    }
}

pub struct UIContext<'a> {
    pub stack: UIWidgetTree<'a>,
}

impl<'a> UIContext<'a> {}
