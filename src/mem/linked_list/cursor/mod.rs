use std::{marker::PhantomData, mem};

use super::{Link, LinkedList};

pub struct CursorMut<'a, T> {
    current: Link<T>,
    pub list: &'a mut LinkedList<T>,
    pub index: Option<usize>,
}

impl<'a, T> CursorMut<'a, T> {
    pub fn new(list: &'a mut LinkedList<T>) -> Self {
        Self {
            index: None,
            current: None,
            list,
        }
    }

    pub fn index(&self) -> Option<usize> {
        self.index
    }

    pub fn current(&mut self) -> Option<&'a mut T> {
        self.current
            .map(|mut node| unsafe { &mut node.as_mut().elem })
    }

    pub fn peek_next(&mut self) -> Option<&mut T> {
        unsafe {
            match self.current {
                Some(node) => (*node.as_ptr()).back.map(|node| &mut (*node.as_ptr()).elem),
                None => self.list.front.map(|node| &mut (*node.as_ptr()).elem),
            }
        }
    }

    pub fn peek_prev(&mut self) -> Option<&mut T> {
        unsafe {
            match self.current {
                Some(node) => (*node.as_ptr())
                    .front
                    .map(|node| &mut (*node.as_ptr()).elem),
                None => self.list.back.map(|node| &mut (*node.as_ptr()).elem),
            }
        }
    }

    pub fn move_next(&mut self) {
        if let Some(current) = self.current {
            unsafe {
                self.current = (*current.as_ptr()).back;

                if self.current.is_some() {
                    // If there was a next node, advance our cursor index to
                    // reflect that.

                    *self.index.as_mut().unwrap() += 1;
                } else {
                    // If our cursor's `current` was the last node in the list,
                    // update our cursor index to indicate that we now point to
                    // the ghost node.

                    self.index = None;
                }
            }
        } else if !self.list.is_empty() {
            // We're pointing at the ghost node, and our list has nodes.

            self.current = self.list.front;

            self.index = Some(0);
        } else {
            // We're pointing at the ghost node, and our list is empty.

            // Do nothing.
        }
    }

    pub fn move_prev(&mut self) {
        if let Some(current) = self.current {
            unsafe {
                self.current = (*current.as_ptr()).front;

                if self.current.is_some() {
                    // If there was a previous node, retreat our cursor index to
                    // reflect that.

                    *self.index.as_mut().unwrap() -= 1;
                } else {
                    // If our cursor's `current` was the first node in the list,
                    // update our cursor index to indicate that we point to the
                    // ghost node.

                    self.index = None;
                }
            }
        } else if !self.list.is_empty() {
            // We're pointing at the ghost node, and list has at least 1 node.

            self.current = self.list.back;

            self.index = Some(self.list.len - 1);
        } else {
            // We're pointing at the ghost node, and the list is empty.

            // Do nothing.
        }
    }

    pub fn split_before(&mut self) -> LinkedList<T> {
        if let Some(current) = self.current {
            // Bifurcate our list.

            unsafe {
                let original_front = self.list.front;
                let original_back = self.list.back;
                let original_len = self.list.len;

                let prev = (*current.as_ptr()).front;

                let after_front = Some(current);
                let after_back = original_back;
                let after_len = original_len - self.index.unwrap();

                let before_front = original_front;
                let before_back = prev;
                let before_len = original_len - after_len;

                // panic!();

                if let Some(prev) = prev {
                    (*prev.as_ptr()).back = None;
                    (*current.as_ptr()).front = None;
                }

                self.list.front = after_front;
                self.list.back = after_back;
                self.list.len = after_len;

                self.index = Some(0);

                LinkedList {
                    front: before_front,
                    back: before_back,
                    len: before_len,
                    _boo: PhantomData,
                }
            }
        } else {
            // Our cursor is pointing at `self.list`'s ghost node.
            // Our result should consume the entire entire existing list.

            mem::replace(self.list, LinkedList::new())
        }
    }

    pub fn split_after(&mut self) -> LinkedList<T> {
        if let Some(current) = self.current {
            // Bifurcate our list.

            unsafe {
                let original_front = self.list.front;
                let original_back = self.list.back;
                let original_len = self.list.len;

                let next = (*current.as_ptr()).back;

                let before_front = original_front;
                let before_back = Some(current);
                let before_len = self.index.unwrap() + 1;

                let after_front = next;
                let after_back = original_back;
                let after_len = original_len - before_len;

                if let Some(next) = next {
                    (*next.as_ptr()).front = None;
                    (*current.as_ptr()).back = None;
                }

                self.list.front = before_front;
                self.list.back = before_back;
                self.list.len = before_len;

                LinkedList {
                    front: after_front,
                    back: after_back,
                    len: after_len,
                    _boo: PhantomData,
                }
            }
        } else {
            // Our cursor is pointing at `self.list`'s ghost node.
            // Our result should consume the entire entire existing list.

            mem::replace(self.list, LinkedList::new())
        }
    }
}

#[cfg(test)]
mod test {
    use super::LinkedList;

    #[test]
    fn test_cursor_move_peek() {
        let mut ll: LinkedList<u32> = LinkedList::new();

        ll.extend([1, 2, 3, 4, 5, 6]);

        let mut cursor = ll.cursor_mut();

        cursor.move_next();
        assert_eq!(cursor.current(), Some(&mut 1));
        assert_eq!(cursor.peek_next(), Some(&mut 2));
        assert_eq!(cursor.peek_prev(), None);
        assert_eq!(cursor.index(), Some(0));

        cursor.move_prev();
        assert_eq!(cursor.current(), None);
        assert_eq!(cursor.peek_next(), Some(&mut 1));
        assert_eq!(cursor.peek_prev(), Some(&mut 6));
        assert_eq!(cursor.index(), None);

        cursor.move_next();
        cursor.move_next();
        assert_eq!(cursor.current(), Some(&mut 2));
        assert_eq!(cursor.peek_next(), Some(&mut 3));
        assert_eq!(cursor.peek_prev(), Some(&mut 1));
        assert_eq!(cursor.index(), Some(1));

        let mut cursor = ll.cursor_mut();

        cursor.move_prev();
        assert_eq!(cursor.current(), Some(&mut 6));
        assert_eq!(cursor.peek_next(), None);
        assert_eq!(cursor.peek_prev(), Some(&mut 5));
        assert_eq!(cursor.index(), Some(5));

        cursor.move_next();
        assert_eq!(cursor.current(), None);
        assert_eq!(cursor.peek_next(), Some(&mut 1));
        assert_eq!(cursor.peek_prev(), Some(&mut 6));
        assert_eq!(cursor.index(), None);

        cursor.move_prev();
        cursor.move_prev();
        assert_eq!(cursor.current(), Some(&mut 5));
        assert_eq!(cursor.peek_next(), Some(&mut 6));
        assert_eq!(cursor.peek_prev(), Some(&mut 4));
        assert_eq!(cursor.index(), Some(4));
    }

    #[test]
    fn split_before() {
        let mut ll: LinkedList<u32> = LinkedList::new();

        ll.extend([200, 201, 202, 203, 1, 100, 101, 102, 103, 8, 2, 3, 4, 5, 6]);

        let mut cursor = ll.cursor_mut();

        cursor.move_next();
        cursor.move_prev();

        let before = cursor.split_before();

        assert_eq!(
            &before.into_iter().collect::<Vec<_>>(),
            &[200, 201, 202, 203, 1, 100, 101, 102, 103, 8, 2, 3, 4, 5, 6]
        );

        assert_eq!(&ll.into_iter().collect::<Vec<_>>(), &Vec::<u32>::new());
    }

    #[test]
    fn split_after() {
        let mut ll: LinkedList<u32> = LinkedList::new();

        ll.extend([200, 201, 202, 203, 1, 100, 101, 102, 103, 8, 2, 3, 4, 5, 6]);

        let mut cursor = ll.cursor_mut();

        cursor.move_next();
        cursor.move_next();
        cursor.move_next();
        cursor.move_next();
        cursor.move_next();
        cursor.move_next();
        cursor.move_next();

        let after = cursor.split_after();

        assert_eq!(
            after.into_iter().collect::<Vec<_>>(),
            &[102, 103, 8, 2, 3, 4, 5, 6]
        );

        assert_eq!(
            ll.iter().cloned().collect::<Vec<_>>(),
            &[200, 201, 202, 203, 1, 100, 101]
        );
    }
}
