use std::fmt::Debug;
use std::ptr::{self, NonNull};

pub struct Node<T> {
    pub value: T,
    next: Option<Box<Node<T>>>,
    prev: Option<NonNull<Node<T>>>,
}

pub struct OwningList<T>(Option<Box<Node<T>>>);

impl<T> Default for OwningList<T> {
    fn default() -> Self {
        Self(None)
    }
}

impl<T> Debug for OwningList<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut list = f.debug_list();
        list.entries(self.iter());
        list.finish()
    }
}

impl<T> OwningList<T> {
    pub fn prepend(&mut self, value: T) -> NonNull<Node<T>> {
        let list_tail = self.0.take();
        let mut head = Box::new(Node {
            value,
            next: list_tail,
            prev: None,
        });
        let raw = &mut *head as *mut Node<T>;
        let head_ptr = unsafe { NonNull::new_unchecked(raw) };
        if let Some(list_tail) = &mut head.next {
            list_tail.prev = Some(head_ptr)
        }
        self.0.replace(head);
        head_ptr
    }

    pub fn move_to_head(&mut self, ptr: NonNull<Node<T>>) {
        // check if it's already at the head of the list
        if let Some(existing) = &self.0 {
            if ptr::eq(ptr.as_ptr(), existing.as_ref()) {
                return;
            }
        }
        let mut head = self.remove_ptr(ptr).unwrap();
        head.as_mut().next = self.0.take();
        if let Some(list_tail) = &mut head.next {
            list_tail.prev = Some(ptr)
        }
        self.0.replace(head);
    }

    // returns the pointed-to node
    pub fn remove_ptr(&mut self, mut ptr: NonNull<Node<T>>) -> Option<Box<Node<T>>> {
        let to_remove = unsafe { ptr.as_mut() };
        self.remove_to_owned(to_remove)
    }

    fn remove_to_owned(&mut self, item: &mut Node<T>) -> Option<Box<Node<T>>> {
        if let Some(mut prev_ptr) = item.prev.take() {
            // not the head
            let mut prev = unsafe { prev_ptr.as_mut() };
            // careful, this contains "item". Don't use it, just return it
            let node = prev.next.take();
            if let Some(mut old_next) = item.next.take() {
                old_next.prev = Some(prev_ptr);
                prev.next = Some(old_next);
            }
            node
        } else {
            // is the head
            if let Some(mut next) = item.next.take() {
                next.prev = None;
                self.0.replace(next)
            } else {
                self.0.take()
            }
        }
    }

    pub fn iter(&self) -> ListIter<'_, T> {
        ListIter::new(&self.0)
    }
}

impl<T> IntoIterator for OwningList<T> {
    type Item = T;

    type IntoIter = ListIntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        ListIntoIter::new(self.0)
    }
}

pub struct ListIter<'list, T> {
    next: &'list Option<Box<Node<T>>>,
}

impl<'list, T> ListIter<'list, T> {
    pub fn new(head: &'list Option<Box<Node<T>>>) -> Self {
        Self { next: head }
    }
}

impl<'list, T> Iterator for ListIter<'list, T> {
    type Item = &'list T;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(next) = self.next {
            let val = &next.value;
            self.next = &next.next;
            Some(val)
        } else {
            None
        }
    }
}

pub struct ListIntoIter<T> {
    next: Option<Box<Node<T>>>,
}

impl<T> ListIntoIter<T> {
    pub fn new(head: Option<Box<Node<T>>>) -> Self {
        Self { next: head }
    }
}

impl<T> Iterator for ListIntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(next) = self.next.take() {
            let val = next.value;
            self.next = next.next;
            Some(val)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn to_vec<T: Clone>(list: &OwningList<T>) -> Vec<T> {
        list.iter().cloned().collect()
    }

    #[test]
    fn it_works() {
        let mut list = OwningList::<usize>::default();
        let one_ptr = list.prepend(1);
        assert_eq!(to_vec(&list), vec![1]);
        let two_ptr = list.prepend(2);
        assert_eq!(to_vec(&list), vec![2, 1]);
        let three_ptr = list.prepend(3);
        assert_eq!(to_vec(&list), vec![3, 2, 1]);
        list.remove_ptr(two_ptr);
        assert_eq!(to_vec(&list), vec![3, 1]);
        list.remove_ptr(three_ptr);
        assert_eq!(to_vec(&list), vec![1]);
        list.remove_ptr(one_ptr);
        assert_eq!(to_vec(&list), vec![]);
    }

    #[test]
    fn move_to_head() {
        let mut list = OwningList::<usize>::default();
        let one_ptr = list.prepend(1);
        let two_ptr = list.prepend(2);
        assert_eq!(to_vec(&list), vec![2, 1]);
        list.remove_ptr(one_ptr);
        let _one_ptr = list.prepend(1);
        list.move_to_head(two_ptr);
    }
}
