use std::{marker::PhantomData, ptr::NonNull};

pub struct LinkedList<T> {
    front: Link<T>,
    back: Link<T>,
    len: usize,
    _boo: PhantomData<T>,
}

pub struct Iter<'a, T> {
    front: Link<T>,
    back: Link<T>,
    len: usize,
    _boo: PhantomData<&'a T>,
}

pub struct IntoIter<T> {
    list: LinkedList<T>,
}

type Link<T> = Option<NonNull<Node<T>>>;

struct Node<T> {
    elem: T,
    front: Link<T>,
    back: Link<T>,
}

impl<T> LinkedList<T> {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            front: None,
            back: None,
            len: 0,
            _boo: PhantomData,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn clear(&mut self) {
        while self.pop_front().is_some() {}
    }

    pub fn iter(&self) -> Iter<T> {
        Iter {
            front: self.front,
            back: self.back,
            len: self.len,
            _boo: PhantomData,
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn into_iter(self) -> IntoIter<T> {
        IntoIter { list: self }
    }

    pub fn front(&self) -> Option<&T> {
        unsafe { self.front.map(|front| &(*front.as_ptr()).elem) }
    }

    pub fn push_front(&mut self, elem: T) {
        unsafe {
            // Box::new() could panic, but it would occur here before we
            // (temporarily) violate our list's invariants.
            let new_node = NonNull::new_unchecked(Box::into_raw(Box::new(Node {
                elem,
                front: None,
                back: None,
            })));

            if let Some(old_front) = self.front {
                // List is not empty.

                (*old_front.as_ptr()).front = Some(new_node);

                (*new_node.as_ptr()).back = Some(old_front);
            } else {
                // List is empty.

                // Asserts could panic here, where our invariants don't all hold!
                // debug_assert!(self.front.is_none());
                // debug_assert!(self.back.is_none());
                // debug_assert!(self.len == 0);

                self.back = Some(new_node);
            }

            self.front = Some(new_node);

            self.len += 1;
        }
    }

    pub fn pop_front(&mut self) -> Option<T> {
        unsafe {
            self.front.map(|old_front| {
                // List is not empty.

                let boxed_node = Box::from_raw(old_front.as_ptr());

                let result = boxed_node.elem;

                self.front = boxed_node.back;

                if let Some(front) = self.front {
                    // Update new front's front-pointer.

                    (*front.as_ptr()).front = None;
                } else {
                    // List has no nodes remaining.

                    // Asserts could panic here, where our invariants don't all hold!
                    // debug_assert!(self.len == 1);

                    self.back = None;
                }

                self.len -= 1;

                result
            })
        }
    }
}

impl<T> Drop for LinkedList<T> {
    fn drop(&mut self) {
        while self.pop_front().is_some() {}
    }
}

// Traits for LinkedList<T>.

impl<T> IntoIterator for LinkedList<T> {
    type Item = T;

    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.into_iter()
    }
}

// Traits for &'a LinkedList<T>.

impl<'a, T> IntoIterator for &'a LinkedList<T> {
    type Item = &'a T;

    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

// Traits for IntoIter<T>.

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.list.pop_front()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.list.len, Some(self.list.len))
    }
}

impl<T> DoubleEndedIterator for IntoIter<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

impl<T> ExactSizeIterator for IntoIter<T> {
    fn len(&self) -> usize {
        self.list.len
    }
}

// Traits for Iter<'a, T>.

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.len > 0 {
            self.front.map(|node| unsafe {
                self.len -= 1;

                self.front = (*node.as_ptr()).back;

                &(*node.as_ptr()).elem
            })
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<'a, T> DoubleEndedIterator for Iter<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.len > 0 {
            self.back.map(|node| unsafe {
                self.len -= 1;

                self.back = (*node.as_ptr()).front;

                &(*node.as_ptr()).elem
            })
        } else {
            None
        }
    }
}

impl<'a, T> ExactSizeIterator for Iter<'a, T> {
    fn len(&self) -> usize {
        self.len
    }
}

#[cfg(test)]
mod test {
    use super::LinkedList;

    #[test]
    fn basics_front() {
        let mut list = LinkedList::new();

        assert_eq!(list.len(), 0);
        assert_eq!(list.pop_front(), None);
        assert_eq!(list.len(), 0);

        list.push_front(10);
        assert_eq!(list.len(), 1);

        assert_eq!(list.pop_front(), Some(10));
        assert_eq!(list.len(), 0);
        assert_eq!(list.pop_front(), None);
        assert_eq!(list.len(), 0);

        list.push_front(10);
        assert_eq!(list.len(), 1);

        list.push_front(20);
        assert_eq!(list.len(), 2);

        list.push_front(30);
        assert_eq!(list.len(), 3);
        assert_eq!(list.pop_front(), Some(30));
        assert_eq!(list.len(), 2);

        list.push_front(40);
        assert_eq!(list.len(), 3);

        assert_eq!(list.pop_front(), Some(40));
        assert_eq!(list.len(), 2);

        assert_eq!(list.pop_front(), Some(20));
        assert_eq!(list.len(), 1);

        assert_eq!(list.pop_front(), Some(10));
        assert_eq!(list.len(), 0);
        assert_eq!(list.pop_front(), None);
        assert_eq!(list.len(), 0);
        assert_eq!(list.pop_front(), None);
        assert_eq!(list.len(), 0);
    }
}
