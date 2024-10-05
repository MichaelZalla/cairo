use std::marker::PhantomData;
use std::ptr::NonNull;

pub struct LinkedList<T> {
    front: Link<T>,
    back: Link<T>,
    len: usize,
    _phantom: PhantomData<T>,
}

type Link<T> = Option<NonNull<Node<T>>>;

struct Node<T> {
    elem: T,
    front: Link<T>,
    back: Link<T>,
}

impl<T> LinkedList<T> {
    pub fn new() -> Self {
        Self {
            front: None,
            back: None,
            len: 0,
            _phantom: PhantomData,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn front(&self) -> Option<&T> {
        unsafe { self.front.map(|node| &(*node.as_ptr()).elem) }
    }

    pub fn front_mut(&mut self) -> Option<&mut T> {
        unsafe { self.front.map(|node| &mut (*node.as_ptr()).elem) }
    }

    pub fn push_front(&mut self, elem: T) {
        unsafe {
            let new_front = NonNull::new_unchecked(Box::into_raw(Box::new(Node {
                elem,
                front: None,
                back: None,
            })));

            if let Some(old_front) = self.front {
                // Put the new front before the old one.

                (*old_front.as_ptr()).front = Some(new_front);
                (*new_front.as_ptr()).back = Some(old_front);
            } else {
                // If there's no front, then we're the empty list and we need
                // to set the back too; also, some integrity checks...

                self.back = Some(new_front);
            }

            self.front = Some(new_front);
            self.len += 1;
        }
    }

    pub fn pop_front(&mut self) -> Option<T> {
        unsafe {
            self.front.map(|old_front| {
                // Bring the Box back to life so we can move out its value, and
                // subsequently drop the Box here.
                let boxed_node = Box::from_raw(old_front.as_ptr());
                let result = boxed_node.elem;

                // Make the next node into the new front.
                self.front = boxed_node.back;

                if let Some(new_front) = self.front {
                    // Cleanup the new front's reference to the old front.
                    (*new_front.as_ptr()).front = None;
                } else {
                    // If the front is now 'null', then the list must be empty!
                    self.back = None;
                }

                self.len -= 1;

                result

                // Box dropped (freed) here.
            })
        }
    }
}

impl<T> Drop for LinkedList<T> {
    fn drop(&mut self) {
        // Pop until we stop...
        while let Some(_) = self.pop_front() {}
    }
}

#[cfg(test)]
mod test {
    use super::LinkedList;

    #[test]
    fn test_basic_front() {
        let mut list = LinkedList::new();

        // Try to break an empty list.

        assert_eq!(list.len(), 0);
        assert_eq!(list.pop_front(), None);
        assert_eq!(list.len(), 0);

        // Try to break a single-item list.

        list.push_front(10);

        assert_eq!(list.len(), 1);
        assert_eq!(list.pop_front(), Some(10));
        assert_eq!(list.len(), 0);
        assert_eq!(list.pop_front(), None);
        assert_eq!(list.len(), 0);

        // Mess around

        list.push_front(10);

        assert_eq!(list.len(), 1);

        list.push_front(20);

        assert_eq!(list.len(), 2);

        list.push_front(30);

        assert_eq!(list.len(), 3);

        let popped = list.pop_front();

        assert_eq!(popped, Some(30));

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
