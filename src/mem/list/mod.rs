#[derive(Default, Debug, Clone)]
pub struct List<T> {
    head: Link<T>,
}

pub struct IntoIter<T>(List<T>);

type Link<T> = Option<Box<Node<T>>>;

#[derive(Default, Debug, Clone)]
pub struct Node<T> {
    elem: T,
    next: Link<T>,
}

impl<T> Drop for List<T> {
    fn drop(&mut self) {
        let mut current_link = self.head.take();

        while let Some(mut boxed_node) = current_link {
            current_link = boxed_node.next.take();
        }
    }
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.pop()
    }
}

impl<T> List<T> {
    pub fn new() -> Self {
        Self { head: None }
    }

    pub fn into_iter(self) -> IntoIter<T> {
        IntoIter(self)
    }

    pub fn peek(&self) -> Option<&T> {
        self.head.as_ref().map(|boxed_node| &boxed_node.elem)
    }

    pub fn peek_mut(&mut self) -> Option<&mut T> {
        self.head.as_mut().map(|boxed_node| &mut boxed_node.elem)
    }

    pub fn push(&mut self, elem: T) {
        let node = Box::new(Node {
            elem,
            next: self.head.take(),
        });

        self.head = Some(node);
    }

    pub fn pop(&mut self) -> Option<T> {
        self.head.take().map(|boxed_node| {
            self.head = boxed_node.next;

            boxed_node.elem
        })
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

    #[test]
    fn peek() {
        let mut list = List::new();

        assert_eq!(list.peek(), None);
        assert_eq!(list.peek_mut(), None);

        for i in 0..3 {
            list.push(i);
        }

        assert_eq!(list.peek(), Some(&2));
        assert_eq!(list.peek_mut(), Some(&mut 2));

        if let Some(value) = list.peek_mut() {
            *value = 42
        }

        assert_eq!(list.peek(), Some(&42));
        assert_eq!(list.pop(), Some(42));
    }

    #[test]
    fn into_iter() {
        let mut list = List::new();

        list.push(1);
        list.push(2);
        list.push(3);

        let mut iter = list.into_iter();

        assert_eq!(iter.next(), Some(3));
        assert_eq!(iter.next(), Some(2));
        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next(), None);
    }
}
