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
            let next = if let Some(current) = self.current {
                (*current.as_ptr()).back
            } else {
                self.list.front
            };

            next.map(|node| &mut (*node.as_ptr()).elem)
        }
    }

    pub fn peek_prev(&mut self) -> Option<&mut T> {
        unsafe {
            let prev = if let Some(current) = self.current {
                (*current.as_ptr()).front
            } else {
                self.list.back
            };

            prev.map(|node| &mut (*node.as_ptr()).elem)
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

    pub fn remove_next(&mut self) -> Option<T> {
        if !self.list.is_empty() {
            unsafe {
                let next = self
                    .current
                    .and_then(|current| (*current.as_ptr()).back)
                    .map(|node| node.as_ptr());

                if let Some(next) = next {
                    // We have at least 1 node following `current` that we
                    // should remove.

                    // Check if we should patch the node that follows `next`.
                    if let Some(after_next) = (*next).back {
                        (*after_next.as_ptr()).front = self.current;
                    } else {
                        // We just removed the last node in the list.
                        self.list.back = self.current;
                    }

                    (*self.current.unwrap().as_ptr()).back = (*next).back;

                    let boxed_node = Box::from_raw(next);
                    let elem = boxed_node.elem;

                    Some(elem)
                } else {
                    // Either `current` points to the ghost node, or `current`
                    // points to the last node in our list. In either case, we
                    // should remove the first node from the list.
                    let old_front = self.list.front.take().unwrap();

                    let new_front = (*old_front.as_ptr()).back;

                    self.list.front = new_front;

                    if let Some(front) = new_front {
                        (*front.as_ptr()).front = None;
                    }

                    self.list.len -= 1;

                    // No change to cursor index.

                    let boxed_node = Box::from_raw(old_front.as_ptr());
                    let elem = boxed_node.elem;

                    Some(elem)
                }
            }
        } else {
            // Our list is empty. Nothing to remove.

            None
        }
    }

    pub fn remove_prev(&mut self) -> Option<T> {
        if !self.list.is_empty() {
            unsafe {
                let prev = self
                    .current
                    .and_then(|current| (*current.as_ptr()).front)
                    .map(|node| node.as_ptr());

                let node = if let Some(prev) = prev {
                    // We have at least 1 node before `current` that we
                    // should remove.

                    // Check if we should patch the node that comes before `prev`.
                    if let Some(before_prev) = (*prev).front {
                        (*before_prev.as_ptr()).back = self.current;
                    } else {
                        // We just removed the first node in the list.
                        self.list.front = self.current;
                    }

                    (*self.current.unwrap().as_ptr()).front = (*prev).front;

                    *self.index.as_mut().unwrap() -= 1;

                    prev
                } else {
                    // Either `current` points to the ghost node, or `current`
                    // points to the first node in our list. In either case, we
                    // should remove the last node from the list.
                    let old_back = self.list.back.take().unwrap();

                    let new_back = (*old_back.as_ptr()).front;

                    self.list.back = new_back;

                    if let Some(back) = new_back {
                        (*back.as_ptr()).back = None;
                    }

                    old_back.as_ptr()
                };

                self.list.len -= 1;

                let boxed_node = Box::from_raw(node);
                let elem = boxed_node.elem;

                Some(elem)
            }
        } else {
            // Our list is empty. Nothing to remove.

            None
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

    pub fn splice_before(&mut self, mut input: LinkedList<T>) {
        unsafe {
            if input.is_empty() {
                // 1. Nothing to add, so our list is unchanged.
            } else if let Some(current) = self.current {
                // 2. Insert all nodes from `input` before `current`.
                let input_front = input.front.take().unwrap();
                let input_back = input.back.take().unwrap();

                if let Some(prev) = (*current.as_ptr()).front {
                    // We're appending somewhere in the middle of our list.

                    (*prev.as_ptr()).back = Some(input_front);
                    (*input_front.as_ptr()).front = Some(prev);
                    (*input_back.as_ptr()).back = Some(current);
                    (*current.as_ptr()).front = Some(input_back);
                } else {
                    // We're appending to the front of our list.

                    self.list.front = Some(input_front);

                    (*current.as_ptr()).front = Some(input_back);
                    (*input_back.as_ptr()).back = Some(current);
                }

                *self.index.as_mut().unwrap() += input.len;
            } else if let Some(back) = self.list.back {
                // 3. Insert all nodes from `input` at the end of our list (append).
                let input_front = input.front.take().unwrap();
                let input_back = input.back.take().unwrap();

                (*input_front.as_ptr()).front = Some(back);
                (*back.as_ptr()).back = Some(input_front);

                self.list.back = Some(input_back);
            } else {
                // 4. Our list is empty, so replace our list with `input`.
                std::mem::swap(self.list, &mut input);
            }

            self.list.len += input.len;

            input.len = 0;
        }
    }

    pub fn splice_after(&mut self, mut input: LinkedList<T>) {
        unsafe {
            if input.is_empty() {
                // 1. Nothing to add, so our list is unchanged.
            } else if let Some(current) = self.current {
                // 2. Insert all nodes from `input` after `current`.
                let input_front = input.front.take().unwrap();
                let input_back = input.back.take().unwrap();

                if let Some(next) = (*current.as_ptr()).back {
                    // We're appending somewhere in the middle of our list.

                    (*current.as_ptr()).back = Some(input_front);
                    (*input_front.as_ptr()).front = Some(current);
                    (*next.as_ptr()).front = Some(input_back);
                    (*input_back.as_ptr()).back = Some(next);
                } else {
                    // We're appending to the back of our list.

                    self.list.back = Some(input_back);

                    (*current.as_ptr()).back = Some(input_front);
                    (*input_front.as_ptr()).front = Some(current);
                }

                // No change to cursor index.
            } else if let Some(front) = self.list.front {
                // 3. Insert all nodes from `input` at the beginning of our list (prepend).
                let input_front = input.front.take().unwrap();
                let input_back = input.back.take().unwrap();

                (*input_front.as_ptr()).back = Some(front);
                (*front.as_ptr()).front = Some(input_back);

                self.list.front = Some(input_front);
            } else {
                // 4. Our list is empty, so replace our list with `input`.
                std::mem::swap(self.list, &mut input);
            }

            self.list.len += input.len;

            input.len = 0;
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

        check_links(&ll);

        assert_eq!(
            &before.into_iter().collect::<Vec<_>>(),
            &[200, 201, 202, 203, 1, 100, 101, 102, 103, 8, 2, 3, 4, 5, 6]
        );

        assert_eq!(&ll.into_iter().collect::<Vec<_>>(), &Vec::<u32>::new());
    }

    #[test]
    fn remove_empty() {
        let mut ll: LinkedList<u32> = LinkedList::new();

        {
            let mut cursor = ll.cursor_mut();

            assert_eq!(cursor.remove_next(), None);
            assert_eq!(cursor.index(), None);
            assert_eq!(cursor.remove_prev(), None);
            assert_eq!(cursor.index(), None);
        }

        check_links(&ll);

        assert_eq!(&ll.into_iter().collect::<Vec<_>>(), &Vec::<u32>::new());
    }

    #[test]
    fn remove_next_cursor_at_front() {
        let mut ll: LinkedList<u32> = LinkedList::new();
        ll.extend([1, 2, 3, 4, 5, 6]);

        {
            let mut cursor = ll.cursor_mut();

            assert_eq!(cursor.remove_next(), Some(1));
            assert_eq!(cursor.index(), None);
        }

        check_links(&ll);

        assert_eq!(&ll.into_iter().collect::<Vec<_>>(), &[2, 3, 4, 5, 6]);
    }

    #[test]
    fn remove_prev_cursor_at_front() {
        let mut ll: LinkedList<u32> = LinkedList::new();
        ll.extend([1, 2, 3, 4, 5, 6]);

        {
            let mut cursor = ll.cursor_mut();

            assert_eq!(cursor.remove_prev(), Some(6));
            assert_eq!(cursor.index(), None);
        }

        check_links(&ll);

        assert_eq!(&ll.into_iter().collect::<Vec<_>>(), &[1, 2, 3, 4, 5]);
    }

    #[test]
    fn remove_next_cursor_at_end() {
        let mut ll: LinkedList<u32> = LinkedList::new();
        ll.extend([1, 2, 3, 4, 5, 6]);

        let mut cursor = ll.cursor_mut();

        cursor.move_next();
        cursor.move_next();
        cursor.move_next();
        cursor.move_next();
        cursor.move_next();
        cursor.move_next();

        assert_eq!(cursor.index(), Some(5));
        assert_eq!(cursor.remove_next(), Some(1));
        assert_eq!(cursor.index(), Some(5));

        check_links(&ll);

        assert_eq!(&ll.into_iter().collect::<Vec<_>>(), &[2, 3, 4, 5, 6]);
    }

    #[test]
    fn remove_prev_cursor_at_end() {
        let mut ll: LinkedList<u32> = LinkedList::new();
        ll.extend([1, 2, 3, 4, 5, 6]);

        let mut cursor = ll.cursor_mut();

        cursor.move_next();
        cursor.move_next();
        cursor.move_next();
        cursor.move_next();
        cursor.move_next();
        cursor.move_next();

        assert_eq!(cursor.index(), Some(5));
        assert_eq!(cursor.remove_prev(), Some(5));
        assert_eq!(cursor.index(), Some(4));

        check_links(&ll);

        assert_eq!(&ll.into_iter().collect::<Vec<_>>(), &[1, 2, 3, 4, 6]);
    }

    #[test]
    fn remove_next_at_middle() {
        let mut ll: LinkedList<u32> = LinkedList::new();
        ll.extend([1, 2, 3, 4, 5, 6]);

        let mut cursor = ll.cursor_mut();

        cursor.move_next();
        assert_eq!(cursor.remove_next(), Some(2));
        assert_eq!(cursor.index(), Some(0));

        cursor.move_next();
        assert_eq!(cursor.remove_next(), Some(4));
        assert_eq!(cursor.index(), Some(1));

        cursor.move_next();
        assert_eq!(cursor.remove_next(), Some(6));
        assert_eq!(cursor.index(), Some(2));

        check_links(&ll);

        assert_eq!(&ll.into_iter().collect::<Vec<_>>(), &[1, 3, 5]);
    }

    #[test]
    fn remove_prev_at_middle() {
        let mut ll: LinkedList<u32> = LinkedList::new();
        ll.extend([1, 2, 3, 4, 5, 6]);

        let mut cursor = ll.cursor_mut();

        cursor.move_next();
        assert_eq!(cursor.remove_prev(), Some(6));
        assert_eq!(cursor.index(), Some(0));

        cursor.move_next();
        assert_eq!(cursor.remove_prev(), Some(1));
        assert_eq!(cursor.index(), Some(0));

        cursor.move_next();
        assert_eq!(cursor.remove_prev(), Some(2));
        assert_eq!(cursor.index(), Some(0));

        check_links(&ll);

        assert_eq!(&ll.into_iter().collect::<Vec<_>>(), &[3, 4, 5]);
    }

    #[test]
    fn drain() {
        let mut ll: LinkedList<u32> = LinkedList::new();
        ll.extend([1, 2, 3, 4, 5, 6]);

        for i in 0..ll.len() {
            {
                let mut cursor = ll.cursor_mut();

                assert_eq!(cursor.remove_next(), Some((i + 1) as u32));
                assert_eq!(cursor.index(), None);
            }

            check_links(&ll);
        }
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

        assert_eq!(cursor.index(), Some(6));

        let after = cursor.split_after();

        assert_eq!(cursor.index(), Some(6));

        check_links(&ll);

        assert_eq!(
            after.into_iter().collect::<Vec<_>>(),
            &[102, 103, 8, 2, 3, 4, 5, 6]
        );

        assert_eq!(
            ll.iter().cloned().collect::<Vec<_>>(),
            &[200, 201, 202, 203, 1, 100, 101]
        );
    }

    #[test]
    fn splice_before() {
        let mut ll: LinkedList<u32> = LinkedList::new();
        ll.extend([1, 2, 3, 4, 5, 6]);

        let mut cursor = ll.cursor_mut();

        cursor.move_next();

        assert_eq!(cursor.index(), Some(0));

        cursor.splice_before(Some(7).into_iter().collect());

        assert_eq!(cursor.index(), Some(1));

        check_links(&ll);

        assert_eq!(
            ll.iter().cloned().collect::<Vec<_>>(),
            &[7, 1, 2, 3, 4, 5, 6]
        );

        let mut cursor = ll.cursor_mut();

        cursor.move_next();
        cursor.move_prev();

        assert_eq!(cursor.index(), None);

        cursor.splice_before(Some(9).into_iter().collect());

        assert_eq!(cursor.index(), None);

        check_links(&ll);

        assert_eq!(
            ll.iter().cloned().collect::<Vec<_>>(),
            &[7, 1, 2, 3, 4, 5, 6, 9]
        );

        let mut cursor = ll.cursor_mut();

        cursor.move_next();

        let mut input: LinkedList<u32> = LinkedList::new();
        input.extend([100, 101, 102, 103]);

        assert_eq!(cursor.index(), Some(0));

        cursor.splice_before(input);

        assert_eq!(cursor.index(), Some(4));

        check_links(&ll);

        assert_eq!(
            ll.iter().cloned().collect::<Vec<_>>(),
            &[100, 101, 102, 103, 7, 1, 2, 3, 4, 5, 6, 9]
        );
    }

    #[test]
    fn splice_after() {
        let mut ll: LinkedList<u32> = LinkedList::new();
        ll.extend([1, 2, 3, 4, 5, 6]);

        let mut cursor = ll.cursor_mut();

        cursor.move_next();

        assert_eq!(cursor.index(), Some(0));

        cursor.splice_after(Some(8).into_iter().collect());

        assert_eq!(cursor.index(), Some(0));

        check_links(&ll);

        assert_eq!(
            ll.iter().cloned().collect::<Vec<_>>(),
            &[1, 8, 2, 3, 4, 5, 6]
        );

        let mut cursor = ll.cursor_mut();

        cursor.move_next();
        cursor.move_prev();

        assert_eq!(cursor.index(), None);

        cursor.splice_after(Some(10).into_iter().collect());

        assert_eq!(cursor.index(), None);

        check_links(&ll);

        assert_eq!(
            ll.iter().cloned().collect::<Vec<_>>(),
            &[10, 1, 8, 2, 3, 4, 5, 6]
        );

        let mut cursor = ll.cursor_mut();

        cursor.move_next();

        let mut input: LinkedList<u32> = LinkedList::new();
        input.extend([100, 101, 102, 103]);

        assert_eq!(cursor.index(), Some(0));

        cursor.splice_after(input);

        assert_eq!(cursor.index(), Some(0));

        check_links(&ll);

        assert_eq!(
            ll.iter().cloned().collect::<Vec<_>>(),
            &[10, 100, 101, 102, 103, 1, 8, 2, 3, 4, 5, 6]
        );
    }

    fn check_links<T: Eq + std::fmt::Debug>(list: &LinkedList<T>) {
        let from_front: Vec<_> = list.iter().collect();
        let from_back: Vec<_> = list.iter().rev().collect();
        let re_reved: Vec<_> = from_back.into_iter().rev().collect();

        assert_eq!(from_front, re_reved);
    }
}
