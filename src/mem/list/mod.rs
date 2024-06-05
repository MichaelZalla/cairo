type Link = Option<Box<Node>>;

#[derive(Default, Debug, Clone)]
pub struct Node {
    elem: i32,
    next: Link,
}

#[derive(Default, Debug, Clone)]
pub struct List {
    head: Link,
}

impl Drop for List {
    fn drop(&mut self) {
        let mut current_link = self.head.take();

        while let Some(mut boxed_node) = current_link {
            current_link = boxed_node.next.take();
        }
    }
}

impl List {
    pub fn new() -> Self {
        Self { head: None }
    }

    pub fn push(&mut self, elem: i32) {
        let node = Box::new(Node {
            elem,
            next: self.head.take(),
        });

        self.head = Some(node);
    }

    pub fn pop(&mut self) -> Option<i32> {
        match self.head.take() {
            None => None,
            Some(boxed_node) => {
                self.head = boxed_node.next;

                Some(boxed_node.elem)
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::List;

    #[test]
    fn basics() {
        let mut list = List::new();

        // Check that an empty list behaves correctly.
        assert_eq!(list.pop(), None);

        // Push some values.
        list.push(1);
        list.push(2);
        list.push(3);

        // Check removal.
        assert_eq!(list.pop(), Some(3));
        assert_eq!(list.pop(), Some(2));

        // Push some more values.
        list.push(4);
        list.push(5);

        // Check removal.
        assert_eq!(list.pop(), Some(5));
        assert_eq!(list.pop(), Some(4));

        // Check list exhaustion.
        assert_eq!(list.pop(), Some(1));
        assert_eq!(list.pop(), None);
    }
}
