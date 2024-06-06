use std::rc::Rc;

#[derive(Default, Debug, Clone)]
pub struct List<T> {
    head: Link<T>,
}

pub struct Iter<'a, T> {
    next: Option<&'a Node<T>>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.next.map(|node| {
            self.next = node.next.as_deref();

            &node.elem
        })
    }
}

type Link<T> = Option<Rc<Node<T>>>;

#[derive(Default, Debug, Clone)]
pub struct Node<T> {
    elem: T,
    next: Link<T>,
}

impl<T> List<T> {
    pub fn new() -> Self {
        Self { head: None }
    }

    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            next: self.head.as_deref(),
        }
    }

    pub fn head(&self) -> Option<&T> {
        self.head.as_ref().map(|boxed_node| &boxed_node.elem)
    }

    pub fn prepend(&self, elem: T) -> List<T> {
        let new_node = Rc::new(Node {
            elem,
            next: self.head.clone(),
        });

        List {
            head: Some(new_node),
        }
    }

    pub fn tail(&self) -> List<T> {
        List {
            head: self.head.as_ref().and_then(|node| node.next.clone()),
        }
    }
}

#[cfg(test)]
mod test {
    use super::List;

    #[test]
    fn basics() {
        let list = List::new();
        assert_eq!(list.head(), None);

        // Build out a list of 3 nodes.
        let list = list.prepend(1).prepend(2).prepend(3);
        assert_eq!(list.head(), Some(&3));

        // Create a new list, with `head` pointing to the successor of the first
        // list's `head` (if any).
        let list = list.tail();
        assert_eq!(list.head(), Some(&2));

        // Create a new list, with `head` pointing to the successor of the
        // second list's `head` (if any).
        let list = list.tail();
        assert_eq!(list.head(), Some(&1));

        // Create a new list, with `head` pointing to the successor of the
        // third list's `head` (if any).
        let list = list.tail();
        assert_eq!(list.head(), None);

        // We should be able to continue calling `.tail()` without issues.
        let list = list.tail();
        assert_eq!(list.head(), None);
    }

    #[test]
    fn iter() {
        let list = List::new().prepend(1).prepend(2).prepend(3);

        let mut iter = list.iter();

        assert_eq!(iter.next(), Some(&3));
        assert_eq!(iter.next(), Some(&2));
        assert_eq!(iter.next(), Some(&1));
    }
}
