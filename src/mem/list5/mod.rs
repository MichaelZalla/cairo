use std::ptr;

type Link<T> = Option<Box<Node<T>>>;

struct Node<T> {
    elem: T,
    next: Link<T>,
}

struct List<T> {
    head: Link<T>,
    tail: *mut Node<T>,
}

impl<T> List<T> {
    pub fn new() -> Self {
        Self {
            head: None,
            tail: ptr::null_mut(),
        }
    }

    pub fn push(&mut self, elem: T) {
        // Push a new node to the end of our queue.

        let mut new_tail = Box::new(Node { elem, next: None });

        let new_tail_raw_ptr: *mut _ = &mut *new_tail;

        if !self.tail.is_null() {
            unsafe {
                (*self.tail).next = Some(new_tail);
            }
        } else {
            self.head = Some(new_tail);
        }

        self.tail = new_tail_raw_ptr;
    }

    pub fn pop(&mut self) -> Option<T> {
        self.head.take().map(|old_head| {
            let head = *old_head;

            self.head = head.next;

            if self.head.is_none() {
                self.tail = ptr::null_mut();
            }

            head.elem
        })
    }
}

#[cfg(test)]
mod test {
    use super::List;

    // #[test]
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

    #[test]
    fn stack_aliasing() {
        let mut data = 10;

        unsafe {
            let ref1 = &mut data;
            let ptr2 = ref1 as *mut _;
            let ref3 = &mut *ptr2;
            let ptr4 = ref3 as *mut _;

            // Access the first raw pointer first. Doing this would pop `ref3`
            // and `ptr4` off the borrow stack, and the following lines of code
            // would be UB:
            //
            //    "ERROR: Undefined Behavior: attempting a read access using
            //    <441810> at alloc152071[0x4], but that tag does not exist in
            //    the borrow stack for this location"
            //
            // *ptr2 += 2;

            // Then access things in borrow-stack-order.
            *ptr4 += 4;
            *ref3 += 3;
            *ptr2 += 2;
            *ref1 += 1;
        }

        println!("{}", data);
    }

    #[test]
    fn array_aliasing() {
        let mut data = [0; 10];

        unsafe {
            let slice = &mut data[..];

            let (slice2_at_0, slice3_at_1) = slice.split_at_mut(1);

            let ref4_at_0 = &mut slice2_at_0[0];
            let ref5_at_1 = &mut slice3_at_1[1];

            let ptr6_at_0 = ref4_at_0 as *mut i32;
            let ptr7_at_1 = ref5_at_1 as *mut i32;

            *ptr7_at_1 += 3;
            *ptr6_at_0 += 3;
            *ref5_at_1 += 2;
            *ref4_at_0 += 1;
        }

        println!("{:?}", &data[..]);
    }
}
