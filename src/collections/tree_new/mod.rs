use std::ptr;

pub struct List<T> {
    head: Link<T>,
    tail: *mut Node<T>,
}

type Link<T> = *mut Node<T>;

struct Node<T> {
    elem: T,
    next: Link<T>,
}

impl<T> List<T> {
    pub fn new() -> Self {
        Self {
            head: ptr::null_mut(),
            tail: ptr::null_mut(),
        }
    }

    pub fn push(&mut self, elem: T) {
        let new_tail = Node {
            elem,
            next: ptr::null_mut(),
        };

        let new_tail_ptr = Box::into_raw(Box::new(new_tail));

        if self.tail.is_null() {
            self.head = new_tail_ptr;
        } else {
            unsafe {
                (*self.tail).next = new_tail_ptr;
            }
        }

        self.tail = new_tail_ptr;
    }

    pub fn pop(&mut self) -> Option<T> {
        unsafe {
            if self.head.is_null() {
                None
            } else {
                let old_head = Box::from_raw(self.head);

                self.head = old_head.next;

                if self.head.is_null() {
                    self.tail = ptr::null_mut();
                }

                Some(old_head.elem)
            }
        }
    }
}

impl<T> Drop for List<T> {
    fn drop(&mut self) {
        while let Some(_) = self.pop() {}
    }
}

#[cfg(test)]
mod test {
    use super::List;

    #[test]
    fn basics() {
        let mut list = List::new();

        // Check empty list.
        assert_eq!(list.pop(), None);

        // Populate a list.
        list.push(1);
        list.push(2);
        list.push(3);

        // Check normal removal.
        assert_eq!(list.pop(), Some(1));
        assert_eq!(list.pop(), Some(2));

        // Push some more nodes.
        list.push(4);
        list.push(5);

        // Check normal removal.
        assert_eq!(list.pop(), Some(3));
        assert_eq!(list.pop(), Some(4));

        // Check exhaustion.
        assert_eq!(list.pop(), Some(5));
        assert_eq!(list.pop(), None);

        // Check exhaustion case left us with a correct `head` and `tail`.
        list.push(6);
        list.push(7);

        // Check normal removal.
        assert_eq!(list.pop(), Some(6));
        assert_eq!(list.pop(), Some(7));
        assert_eq!(list.pop(), None);
    }
}
