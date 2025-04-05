use std::{fmt::Debug, hash::Hash, marker::PhantomData, ptr::NonNull};

use cursor::CursorMut;

pub mod cursor;

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

pub struct IterMut<'a, T> {
    front: Link<T>,
    back: Link<T>,
    len: usize,
    _boo: PhantomData<&'a mut T>,
}

pub struct IntoIter<T> {
    list: LinkedList<T>,
}

pub(in crate::mem::linked_list) type Link<T> = Option<NonNull<Node<T>>>;

pub(in crate::mem::linked_list) struct Node<T> {
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

    pub fn iter_mut(&self) -> IterMut<T> {
        IterMut {
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

    pub fn cursor_mut(&mut self) -> CursorMut<T> {
        // When index is None, cursor points to the list's 'ghost' node.

        CursorMut::new(self)
    }

    pub fn front(&self) -> Option<&T> {
        unsafe { self.front.map(|node| &(*node.as_ptr()).elem) }
    }

    pub fn front_mut(&self) -> Option<&mut T> {
        unsafe { self.front.map(|node| &mut (*node.as_ptr()).elem) }
    }

    pub fn back(&self) -> Option<&T> {
        unsafe { self.back.map(|node| &(*node.as_ptr()).elem) }
    }

    pub fn back_mut(&self) -> Option<&mut T> {
        unsafe { self.back.map(|node| &mut (*node.as_ptr()).elem) }
    }

    pub fn retain<F>(&mut self, mut filter: F)
    where
        F: FnMut(&mut T) -> bool,
    {
        let mut cursor = self.cursor_mut();

        while let Some(next) = cursor.peek_next() {
            if filter(next) {
                cursor.move_next();
            } else {
                cursor.remove_next();
            }
        }
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

    pub fn push_back(&mut self, elem: T) {
        unsafe {
            // Box::new() could panic, but it would occur here before we
            // (temporarily) violate our list's invariants.
            let new_node = NonNull::new_unchecked(Box::into_raw(Box::new(Node {
                elem,
                front: None,
                back: None,
            })));

            if let Some(old_back) = self.back {
                // List is not empty.

                (*old_back.as_ptr()).back = Some(new_node);

                (*new_node.as_ptr()).front = Some(old_back);
            } else {
                // List is empty.

                // Asserts could panic here, where our invariants don't all hold!
                // debug_assert!(self.front.is_none());
                // debug_assert!(self.back.is_none());
                // debug_assert!(self.len == 0);

                self.front = Some(new_node);
            }

            self.back = Some(new_node);

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

    pub fn pop_back(&mut self) -> Option<T> {
        unsafe {
            self.back.map(|old_back| {
                // List is not empty.

                let boxed_node = Box::from_raw(old_back.as_ptr());

                let result = boxed_node.elem;

                self.back = boxed_node.front;

                if let Some(back) = self.back {
                    // Update new front's front-pointer.

                    (*back.as_ptr()).back = None;
                } else {
                    // List has no nodes remaining.

                    // Asserts could panic here, where our invariants don't all hold!
                    // debug_assert!(self.len == 1);

                    self.front = None;
                }

                self.len -= 1;

                result
            })
        }
    }
}

impl<T> Default for LinkedList<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Debug> Debug for LinkedList<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self).finish()
    }
}

impl<T: Clone> Clone for LinkedList<T> {
    fn clone(&self) -> Self {
        let mut new_list = Self::new();

        for elem in self {
            new_list.push_back(elem.clone());
        }

        new_list
    }
}

impl<T> Extend<T> for LinkedList<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for elem in iter {
            self.push_back(elem);
        }
    }
}

impl<T> FromIterator<T> for LinkedList<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut list = Self::new();

        list.extend(iter);

        list
    }
}

impl<T: PartialEq> PartialEq for LinkedList<T> {
    fn eq(&self, other: &Self) -> bool {
        self.len() == other.len() && self.iter().eq(other)
    }

    // fn ne(&self, other: &Self) -> bool {
    //     self.len() != other.len() || self.iter().ne(other)
    // }
}

// See: https://doc.rust-lang.org/std/cmp/trait.Eq.html
impl<T: Eq> Eq for LinkedList<T> {}

impl<T: PartialOrd> PartialOrd for LinkedList<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.iter().partial_cmp(other)
    }
}

impl<T: Ord> Ord for LinkedList<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.iter().cmp(other)
    }
}

impl<T: Hash> Hash for LinkedList<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.len.hash(state);

        for elem in self {
            elem.hash(state);
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

unsafe impl<T: Send> Send for LinkedList<T> {}
unsafe impl<T: Sync> Sync for LinkedList<T> {}

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

unsafe impl<T: Send> Send for Iter<'_, T> {}
unsafe impl<T: Sync> Sync for Iter<'_, T> {}

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

impl<T> DoubleEndedIterator for Iter<'_, T> {
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

impl<T> ExactSizeIterator for Iter<'_, T> {
    fn len(&self) -> usize {
        self.len
    }
}

// Traits for IterMut<'a, T>.

unsafe impl<T: Send> Send for IterMut<'_, T> {}
unsafe impl<T: Sync> Sync for IterMut<'_, T> {}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.len > 0 {
            self.front.map(|node| unsafe {
                self.len -= 1;

                self.front = (*node.as_ptr()).back;

                &mut (*node.as_ptr()).elem
            })
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<T> DoubleEndedIterator for IterMut<'_, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.len > 0 {
            self.back.map(|node| unsafe {
                self.len -= 1;

                self.back = (*node.as_ptr()).front;

                &mut (*node.as_ptr()).elem
            })
        } else {
            None
        }
    }
}

impl<T> ExactSizeIterator for IterMut<'_, T> {
    fn len(&self) -> usize {
        self.len
    }
}

/// ```compile_fail
/// use cairo::mem::linked_list::IterMut;
///
/// fn iter_mut_covariant<'i, 'a, T>(x: IterMut<'i, &'static T>) -> IterMut<'i, &'a T> { x }
/// ```
fn iter_mut_invariant() {}

#[cfg(test)]
mod test {
    use super::{IntoIter, Iter, IterMut, LinkedList};

    fn generate_test() -> LinkedList<i32> {
        list_from(&[0, 1, 2, 3, 4, 5, 6])
    }

    fn list_from<T: Clone>(values: &[T]) -> LinkedList<T> {
        values.iter().map(|value| (*value).clone()).collect()
    }

    #[test]
    fn assert_send_and_sync_traits() {
        fn is_send<T: Send>() {}
        fn is_sync<T: Sync>() {}

        is_send::<LinkedList<i32>>();
        is_sync::<LinkedList<i32>>();

        is_send::<IntoIter<i32>>();
        is_sync::<IntoIter<i32>>();

        is_send::<Iter<i32>>();
        is_sync::<Iter<i32>>();

        is_send::<IterMut<i32>>();
        is_sync::<IterMut<i32>>();
    }

    #[test]
    fn assert_covariance() {
        fn linked_list_covariant<'a, T>(x: LinkedList<&'static T>) -> LinkedList<&'a T> {
            x
        }

        fn iter_covariant<'i, 'a, T>(x: Iter<'i, &'static T>) -> Iter<'i, &'a T> {
            x
        }

        fn into_iter_covariant<'a, T>(x: IntoIter<&'static T>) -> IntoIter<&'a T> {
            x
        }
    }

    #[test]
    fn basics_front() {
        let mut a = LinkedList::new();

        assert_eq!(a.len(), 0);
        assert_eq!(a.pop_front(), None);
        assert_eq!(a.len(), 0);

        a.push_front(10);
        assert_eq!(a.len(), 1);

        assert_eq!(a.pop_front(), Some(10));
        assert_eq!(a.len(), 0);
        assert_eq!(a.pop_front(), None);
        assert_eq!(a.len(), 0);

        a.push_front(10);
        assert_eq!(a.len(), 1);

        a.push_front(20);
        assert_eq!(a.len(), 2);

        a.push_front(30);
        assert_eq!(a.len(), 3);
        assert_eq!(a.pop_front(), Some(30));
        assert_eq!(a.len(), 2);

        a.push_front(40);
        assert_eq!(a.len(), 3);

        assert_eq!(a.pop_front(), Some(40));
        assert_eq!(a.len(), 2);

        assert_eq!(a.pop_front(), Some(20));
        assert_eq!(a.len(), 1);

        assert_eq!(a.pop_front(), Some(10));
        assert_eq!(a.len(), 0);
        assert_eq!(a.pop_front(), None);
        assert_eq!(a.len(), 0);
        assert_eq!(a.pop_front(), None);
        assert_eq!(a.len(), 0);
    }

    #[test]
    fn test_basic() {
        let mut a = LinkedList::new();

        assert_eq!(a.pop_front(), None);
        assert_eq!(a.pop_back(), None);
        assert_eq!(a.pop_front(), None);

        a.push_front(1);
        assert_eq!(a.pop_front(), Some(1));

        a.push_back(2);
        a.push_back(3);
        assert_eq!(a.len(), 2);
        assert_eq!(a.pop_front(), Some(2));
        assert_eq!(a.pop_front(), Some(3));
        assert_eq!(a.len(), 0);
        assert_eq!(a.pop_front(), None);

        a.push_back(1);
        a.push_back(3);
        a.push_back(5);
        a.push_back(7);
        assert_eq!(a.pop_front(), Some(1));

        let mut n = LinkedList::new();
        n.push_front(2);
        n.push_front(3);

        {
            assert_eq!(n.front().unwrap(), &3);

            let x = n.front_mut().unwrap();
            assert_eq!(*x, 3);

            *x = 0;
        }

        {
            assert_eq!(n.back().unwrap(), &2);

            let y = n.back_mut().unwrap();
            assert_eq!(*y, 2);

            *y = 1;
        }

        assert_eq!(n.pop_front(), Some(0));
        assert_eq!(n.pop_front(), Some(1));
    }

    #[test]
    fn test_retain() {
        // Keep all.

        let mut ll = LinkedList::new();
        ll.extend([1, 2, 3, 4, 5, 6]);

        ll.retain(|_elem| true);

        assert_eq!(ll.into_iter().collect::<Vec<_>>(), &[1, 2, 3, 4, 5, 6]);

        // Drop all.

        let mut ll = LinkedList::new();
        ll.extend([1, 2, 3, 4, 5, 6]);

        ll.retain(|_elem| false);

        assert_eq!(ll.into_iter().collect::<Vec<_>>(), Vec::<u32>::new());

        // Keep evens.

        let mut ll = LinkedList::new();
        ll.extend([1, 2, 3, 4, 5, 6]);

        ll.retain(|elem| *elem % 2 == 0);

        assert_eq!(ll.into_iter().collect::<Vec<_>>(), &[2, 4, 6]);
    }

    #[test]
    fn test_iterator() {
        let a = generate_test();

        for (i, elem) in a.iter().enumerate() {
            assert_eq!(i as i32, *elem);
        }

        let mut b = LinkedList::new();
        assert_eq!(b.iter().next(), None);

        b.push_front(4);

        let mut it = b.iter();
        assert_eq!(it.size_hint(), (1, Some(1)));
        assert_eq!(it.next().unwrap(), &4);
        assert_eq!(it.size_hint(), (0, Some(0)));
        assert_eq!(it.next(), None);
    }

    #[test]
    fn test_iterator_double_end() {
        let mut a = LinkedList::new();
        assert_eq!(a.iter().next(), None);

        a.push_front(4);
        a.push_front(5);
        a.push_front(6);

        let mut it = a.iter();
        assert_eq!(it.size_hint(), (3, Some(3)));
        assert_eq!(it.next().unwrap(), &6);
        assert_eq!(it.size_hint(), (2, Some(2)));
        assert_eq!(it.next_back().unwrap(), &4);
        assert_eq!(it.size_hint(), (1, Some(1)));
        assert_eq!(it.next_back().unwrap(), &5);
        assert_eq!(it.next_back(), None);
        assert_eq!(it.next(), None);
    }

    #[test]
    fn test_rev_iter() {
        let a = generate_test();

        for (i, elt) in a.iter().rev().enumerate() {
            assert_eq!(6 - i as i32, *elt);
        }

        let mut b = LinkedList::new();
        assert_eq!(b.iter().next_back(), None);

        b.push_front(4);

        let mut it = b.iter().rev();
        assert_eq!(it.size_hint(), (1, Some(1)));
        assert_eq!(it.next().unwrap(), &4);
        assert_eq!(it.size_hint(), (0, Some(0)));
        assert_eq!(it.next(), None);
    }

    #[test]
    fn test_mut_iter() {
        let a = generate_test();
        let mut len = a.len();

        for (i, elt) in a.iter_mut().enumerate() {
            assert_eq!(i as i32, *elt);
            len -= 1;
        }
        assert_eq!(len, 0);

        let mut b = LinkedList::new();
        assert!(b.iter_mut().next().is_none());

        b.push_front(4);
        b.push_back(5);

        let mut it = b.iter_mut();
        assert_eq!(it.size_hint(), (2, Some(2)));
        assert!(it.next().is_some());
        assert!(it.next().is_some());
        assert_eq!(it.size_hint(), (0, Some(0)));
        assert!(it.next().is_none());
    }

    #[test]
    fn test_iterator_mut_double_end() {
        let mut a = LinkedList::new();
        assert!(a.iter_mut().next_back().is_none());

        a.push_front(4);
        a.push_front(5);
        a.push_front(6);

        let mut it = a.iter_mut();

        assert_eq!(it.size_hint(), (3, Some(3)));
        assert_eq!(*it.next().unwrap(), 6);
        assert_eq!(it.size_hint(), (2, Some(2)));
        assert_eq!(*it.next_back().unwrap(), 4);
        assert_eq!(it.size_hint(), (1, Some(1)));
        assert_eq!(*it.next_back().unwrap(), 5);
        assert!(it.next_back().is_none());
        assert!(it.next().is_none());
    }

    #[test]
    fn test_eq() {
        let mut a: LinkedList<u8> = list_from(&[]);
        let mut b = list_from(&[]);

        assert!(a == b);

        a.push_front(1);
        assert!(a != b);

        b.push_back(1);
        assert!(a == b);

        let a = list_from(&[2, 3, 4]);
        let b = list_from(&[1, 2, 3]);
        assert!(a != b);
    }

    #[test]
    fn test_ord() {
        let a = list_from(&[]);
        let b = list_from(&[1, 2, 3]);

        assert!(a < b);
        assert!(b > a);
        assert!(a <= a);
        assert!(a >= a);
    }

    #[test]
    #[allow(clippy::neg_cmp_op_on_partial_ord)]
    fn test_ord_nan() {
        let nan = f64::NAN;
        let a = list_from(&[nan]);
        let b = list_from(&[nan]);

        assert!(!(a < b));
        assert!(!(a > b));
        assert!(!(a <= b));
        assert!(!(a >= b));

        let a = list_from(&[nan]);
        let one = list_from(&[1.0f64]);
        assert!(!(a < one));
        assert!(!(a > one));
        assert!(!(a <= one));
        assert!(!(a >= one));

        let a = list_from(&[1.0f64, 2.0, nan]);
        let b = list_from(&[1.0f64, 2.0, 3.0]);
        assert!(!(a < b));
        assert!(!(a > b));
        assert!(!(a <= b));
        assert!(!(a >= b));

        let a = list_from(&[1.0f64, 2.0, 4.0, 2.0]);
        let b = list_from(&[1.0f64, 2.0, 3.0, 2.0]);
        assert!(!(a < b));
        assert!(a > one);
        assert!(!(a <= one));
        assert!(a >= one);
    }

    #[test]
    fn test_debug() {
        let a: LinkedList<i32> = (0..10).collect();
        assert_eq!(format!("{:?}", a), "[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]");

        let a: LinkedList<&str> = ["just", "one", "test", "more"].iter().copied().collect();
        assert_eq!(format!("{:?}", a), r#"["just", "one", "test", "more"]"#);
    }

    #[test]
    fn test_hashmap() {
        // Check that HashMap works with this as a key

        let a: LinkedList<i32> = (0..10).collect();
        let b: LinkedList<i32> = (1..11).collect();

        let mut map = std::collections::HashMap::new();

        assert_eq!(map.insert(a.clone(), "list1"), None);
        assert_eq!(map.insert(b.clone(), "list2"), None);

        assert_eq!(map.len(), 2);

        assert_eq!(map.get(&a), Some(&"list1"));
        assert_eq!(map.get(&b), Some(&"list2"));

        assert_eq!(map.remove(&a), Some("list1"));
        assert_eq!(map.remove(&b), Some("list2"));

        assert!(map.is_empty());
    }
}
