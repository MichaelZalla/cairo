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

    pub fn current(&mut self) -> Option<&mut T> {
        self.current
            .map(|node| unsafe { &mut (*node.as_ptr()).elem })
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
        assert_eq!(cursor.index(), Some(0));

        cursor.move_prev();
        assert_eq!(cursor.current(), None);
        assert_eq!(cursor.index(), None);

        cursor.move_next();
        cursor.move_next();
        assert_eq!(cursor.current(), Some(&mut 2));
        assert_eq!(cursor.index(), Some(1));

        let mut cursor = ll.cursor_mut();

        cursor.move_prev();
        assert_eq!(cursor.current(), Some(&mut 6));
        assert_eq!(cursor.index(), Some(5));

        cursor.move_next();
        assert_eq!(cursor.current(), None);
        assert_eq!(cursor.index(), None);

        cursor.move_prev();
        cursor.move_prev();
        assert_eq!(cursor.current(), Some(&mut 5));
        assert_eq!(cursor.index(), Some(4));
    }
}
