use std::ptr;

pub struct List<T> {
    head: Link<T>,
    tail: *mut Node<T>,
}

pub type Link<T> = *mut Node<T>;

pub struct Node<T> {
    elem: T,
    next: Link<T>,
}

impl<T> List<T> {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            head: ptr::null_mut(),
            tail: ptr::null_mut(),
        }
    }

    pub fn push(&mut self, elem: T) {
        // Push a new node at the end of our queue.

        let new_tail = Box::into_raw(Box::new(Node {
            elem,
            next: ptr::null_mut(),
        }));

        if !self.tail.is_null() {
            unsafe {
                (*self.tail).next = new_tail;
            }
        } else {
            self.head = new_tail;
        }

        self.tail = new_tail;
    }

    pub fn pop(&mut self) -> Option<T> {
        unsafe {
            if !self.head.is_null() {
                let old_head = Box::from_raw(self.head);

                self.head = old_head.next;

                if self.head.is_null() {
                    self.tail = ptr::null_mut();
                }

                Some(old_head.elem)
            } else {
                None
            }
        }
    }
}

impl<T> Drop for List<T> {
    fn drop(&mut self) {
        while self.pop().is_some() {}
    }
}

#[cfg(test)]
mod test {
    use super::List;

    #[test]
    fn basics() {
        let mut list = List::new();

        assert_eq!(list.pop(), None);

        list.push(1);
        list.push(2);
        list.push(3);

        assert_eq!(list.pop(), Some(1));
        assert_eq!(list.pop(), Some(2));

        list.push(4);
        list.push(5);

        assert_eq!(list.pop(), Some(3));
        assert_eq!(list.pop(), Some(4));

        assert_eq!(list.pop(), Some(5));
        assert_eq!(list.pop(), None);

        list.push(6);
        list.push(7);

        assert_eq!(list.pop(), Some(6));
        assert_eq!(list.pop(), Some(7));
        assert_eq!(list.pop(), None);
    }
}
