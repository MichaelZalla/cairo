use std::{fmt::Debug, hash::Hash, marker::PhantomData, ptr::NonNull};

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

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn clear(&mut self) {
        while let Some(_) = self.pop_front() {}
    }

    pub fn iter(&self) -> Iter<T> {
        return Iter {
            front: self.front,
            back: self.back,
            len: self.len,
            _phantom: PhantomData,
        };
    }

    pub fn iter_mut(&mut self) -> IterMut<T> {
        return IterMut {
            front: self.front,
            back: self.back,
            len: self.len,
            _phantom: PhantomData,
        };
    }

    pub fn into_iter(self) -> IntoIter<T> {
        IntoIter { list: self }
    }

    pub fn front(&self) -> Option<&T> {
        unsafe { self.front.map(|node| &(*node.as_ptr()).elem) }
    }

    pub fn front_mut(&mut self) -> Option<&mut T> {
        unsafe { self.front.map(|node| &mut (*node.as_ptr()).elem) }
    }

    pub fn back(&self) -> Option<&T> {
        unsafe { self.back.map(|node| &(*node.as_ptr()).elem) }
    }

    pub fn back_mut(&mut self) -> Option<&mut T> {
        unsafe { self.back.map(|node| &mut (*node.as_ptr()).elem) }
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
                // If there's no front, then we're the empty list, so we need to
                // set the back, too.

                self.back = Some(new_front);
            }

            self.front = Some(new_front);
            self.len += 1;
        }
    }

    pub fn push_back(&mut self, elem: T) {
        unsafe {
            let new_back = NonNull::new_unchecked(Box::into_raw(Box::new(Node {
                elem,
                front: None,
                back: None,
            })));

            if let Some(old_back) = self.back {
                // Put the new back in front of the old one.

                (*old_back.as_ptr()).back = Some(new_back);
                (*new_back.as_ptr()).front = Some(old_back);
            } else {
                // If there's no back, then we're the empty list, so we need to
                // set the front, too.

                self.front = Some(new_back);
            }

            self.back = Some(new_back);
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

    pub fn pop_back(&mut self) -> Option<T> {
        unsafe {
            self.back.map(|old_back| {
                // Bring the Box back to life so we can move out its value, and subsequently drop the Box here.
                let boxed_node = Box::from_raw(old_back.as_ptr());
                let result = boxed_node.elem;

                // Make the previous node into the new back.
                self.back = boxed_node.front;

                if let Some(new_back) = self.back {
                    // Cleanup the new back's reference to the old back.
                    (*new_back.as_ptr()).back = None;
                } else {
                    // If the back is now 'null', then the list must be empty!
                    self.front = None;
                }

                self.len -= 1;

                result
            })
        }
    }
}

// IntoIter<T>

pub struct IntoIter<T> {
    list: LinkedList<T>,
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.list.pop_front()
    }
}

impl<T> DoubleEndedIterator for IntoIter<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.list.pop_back()
    }
}

impl<T> ExactSizeIterator for IntoIter<T> {
    fn len(&self) -> usize {
        self.list.len
    }
}

// Iter<'a, T>

pub struct Iter<'a, T> {
    front: Link<T>,
    back: Link<T>,
    len: usize,
    _phantom: PhantomData<&'a T>,
}

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

impl<'a, T> IntoIterator for &'a LinkedList<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

// IterMut<'a, T>

pub struct IterMut<'a, T> {
    front: Link<T>,
    back: Link<T>,
    len: usize,
    _phantom: PhantomData<&'a mut T>,
}

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

impl<'a, T> DoubleEndedIterator for IterMut<'a, T> {
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

impl<'a, T> ExactSizeIterator for IterMut<'a, T> {
    fn len(&self) -> usize {
        self.len
    }
}

impl<'a, T> IntoIterator for &'a mut LinkedList<T> {
    type Item = &'a mut T;
    type IntoIter = IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

// Default

impl<T> Default for LinkedList<T> {
    fn default() -> Self {
        Self::new()
    }
}

// Clone

impl<T: Clone> Clone for LinkedList<T> {
    fn clone(&self) -> Self {
        let mut result = Self::new();

        for item in self {
            result.push_back(item.clone());
        }

        result
    }
}

// Extend

impl<T> Extend<T> for LinkedList<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for item in iter {
            self.push_back(item)
        }
    }
}

// FromIterator

impl<T> FromIterator<T> for LinkedList<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut result = Self::new();

        result.extend(iter);

        result
    }
}

// Debug

impl<T: Debug> Debug for LinkedList<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self).finish()
    }
}

// PartialEq

impl<T: PartialEq> PartialEq for LinkedList<T> {
    fn eq(&self, other: &Self) -> bool {
        self.len() == other.len() && self.iter().eq(other)
    }

    fn ne(&self, other: &Self) -> bool {
        self.len() != other.len() || self.iter().ne(other)
    }
}

// Eq

impl<T: Eq> Eq for LinkedList<T> {}

// PartialOrd

impl<T: PartialOrd> PartialOrd for LinkedList<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.iter().partial_cmp(other)
    }
}

// Ord

impl<T: Ord> Ord for LinkedList<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.iter().cmp(other)
    }
}

// Hash

impl<T: Hash> Hash for LinkedList<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.len.hash(state);

        for item in self {
            item.hash(state);
        }
    }
}

// Drop

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
