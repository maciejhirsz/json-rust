use std::{ ptr, mem, str, slice };
use std::iter::{ Iterator, DoubleEndedIterator };

use value::JsonValue;

// FIXME: Manual Clone!
#[derive(Debug, PartialEq, Clone)]
struct Node {
    pub vacant: bool,
    pub key_buf: [u8; 16],
    pub key_len: usize,
    pub key_ptr: *mut u8,
    pub value: JsonValue,
    pub left: *mut Node,
    pub right: *mut Node,
    pub prev: *mut Node,
    pub next: *mut Node,
}

unsafe impl Sync for Node { }

impl Node {
    fn key(&self) -> &[u8] {
        unsafe {
            slice::from_raw_parts(self.key_ptr, self.key_len)
        }
    }

    fn key_str(&self) -> &str {
        unsafe {
            str::from_utf8_unchecked(self.key())
        }
    }

    fn new(key: &[u8], value: JsonValue) -> Box<Node> {
        unsafe {
            let len = key.len();

            let mut node = Box::new(Node {
                vacant: false,
                key_buf: mem::uninitialized(),
                key_len: key.len(),
                key_ptr: mem::uninitialized(),
                value: value,
                left: ptr::null_mut(),
                right: ptr::null_mut(),
                prev: ptr::null_mut(),
                next: ptr::null_mut(),
            });

            if len <= 16 {
                ptr::copy_nonoverlapping(key.as_ptr(), node.key_buf.as_mut_ptr(), len);
                node.key_ptr = node.key_buf.as_mut_ptr();
            } else {
                let mut heap: Vec<u8> = key.to_vec();
                node.key_ptr = heap.as_mut_ptr();
                mem::forget(heap);
            }

            node
        }
    }

    fn with_prev(key: &[u8], value: JsonValue, prev: *mut Node) -> Box<Node> {
        let mut node = Node::new(key, value);

        node.prev = prev;

        node
    }
}

impl Drop for Node {
    fn drop(&mut self) {
        unsafe {
            if !self.left.is_null() {
                drop(Box::from_raw(self.left));
            }
            if !self.right.is_null() {
                drop(Box::from_raw(self.right));
            }
            if self.key_len > 16 {
                let heap = Vec::from_raw_parts(self.key_ptr, self.key_len, self.key_len);
                drop(heap);
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Object {
    root: *mut Node,
    last: *mut Node,
}

unsafe impl Sync for Object { }

impl Drop for Object {
    fn drop(&mut self) {
        unsafe {
            if !self.root.is_null() {
                drop(Box::from_raw(self.root));
            }
        }
    }
}

impl Object {
    pub fn new() -> Self {
        Object {
            root: ptr::null_mut(),
            last: ptr::null_mut(),
        }
    }

    pub fn insert(&mut self, key: &str, value: JsonValue) {
        let key = key.as_bytes();

        if self.root.is_null() {
            self.root = Box::into_raw(Node::new(key, value));
            self.last = self.root;
            return;
        }

        unsafe {
            let mut node = &mut *self.root;

            loop {
                if key == node.key() {
                    (*node).value = value;
                    return;
                } else if key < node.key() {
                    if !node.left.is_null() {
                        node = &mut *node.left;
                        continue;
                    }
                    node.left = Box::into_raw(Node::with_prev(key, value, self.last));
                    (*self.last).next = node.left;
                    self.last = node.left;
                    return;
                } else {
                    if !node.right.is_null() {
                        node = &mut *node.right;
                        continue;
                    }
                    node.right = Box::into_raw(Node::with_prev(key, value, self.last));
                    (*self.last).next = node.right;
                    self.last = node.right;
                    return;
                }
            }
        }
    }

    pub fn get(&self, key: &str) -> Option<&JsonValue> {
        if self.root.is_null() {
            return None;
        }

        let key = key.as_bytes();

        unsafe {
            let mut node = &*self.root;

            loop {
                if key == node.key() {
                    return if node.vacant {
                        None
                    } else {
                        Some(&node.value)
                    };
                } else if key < &node.key() {
                    if node.left.is_null() {
                        return None;
                    }
                    node = &*node.left;
                } else if key > &node.key() {
                    if node.right.is_null() {
                        return None;
                    }
                    node = &*node.right;
                } else {
                    return None;
                }
            }
        }
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut JsonValue> {
        if self.root.is_null() {
            return None;
        }

        let key = key.as_bytes();

        unsafe {
            let mut node = &mut *self.root;

            loop {
                if key == node.key() {
                    return if node.vacant {
                        None
                    } else {
                        Some(&mut node.value)
                    };
                } else if key < &node.key() {
                    if node.left.is_null() {
                        return None;
                    }
                    node = &mut *node.left;
                } else if key > &node.key() {
                    if node.right.is_null() {
                        return None;
                    }
                    node = &mut *node.right;
                } else {
                    return None;
                }
            }
        }
    }

    pub fn iter<'a>(&self) -> Iter<'a> {
        if self.root.is_null() {
            Iter {
                front: None,
                back: None,
                end: true,
            }
        } else {
            unsafe {
                Iter {
                    front: Some(&*self.root),
                    back: Some(&*self.last),
                    end: false,
                }
            }
        }
    }


    pub fn iter_mut<'a>(&self) -> IterMut<'a> {
        if self.root.is_null() {
            IterMut {
                front: None,
                back: None,
                end: true,
            }
        } else {
            unsafe {
                IterMut {
                    front: Some(&mut *self.root),
                    back: Some(&mut *self.last),
                    end: false,
                }
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.root.is_null()
    }
}

pub struct Iter<'a> {
    front: Option<&'a Node>,
    back: Option<&'a Node>,
    end: bool,
}

impl<'a> Iterator for Iter<'a> {
    type Item = (&'a str, &'a JsonValue);

    fn next(&mut self) -> Option<Self::Item> {
        if self.end {
            return None;
        }

        let node = self.front.expect("Should have a node");

        unsafe {
            let ref value = node.value;
            let key = node.key_str();

            if self.front == self.back {
                self.end = true;
            } else {
                self.front = Some(&*node.next);
            }

            Some((key, value))
        }
    }
}

impl<'a> DoubleEndedIterator for Iter<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.end {
            return None;
        }

        let node = self.back.expect("Should have a node");

        unsafe {
            let ref value = node.value;
            let key = node.key_str();

            if self.front == self.back {
                self.end = true;
            } else {
                self.back = Some(&*node.prev);
            }

            Some((key, value))
        }
    }
}

pub struct IterMut<'a> {
    front: Option<&'a mut Node>,
    back: Option<&'a mut Node>,
    end: bool,
}

impl<'a> Iterator for IterMut<'a> {
    type Item = (&'a str, &'a mut JsonValue);

    fn next(&mut self) -> Option<Self::Item> {
        if self.end {
            return None;
        }

        if self.front == self.back {
            self.end = true;
        }

        let node = self.front.take().expect("Should have a node");

        unsafe {
            let ref mut value = node.value;

            // Construct the key here to avoid the borrow checked
            let key_ptr = node.key_ptr;
            let key_len = node.key_len;
            let key = str::from_utf8_unchecked(
                slice::from_raw_parts(key_ptr, key_len)
            );

            if !self.end {
                self.front = Some(&mut *node.next);
            }

            Some((key, value))
        }
    }
}

impl<'a> DoubleEndedIterator for IterMut<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.end {
            return None;
        }

        if self.front == self.back {
            self.end = true;
        }

        let node = self.back.take().expect("Should have a node");

        unsafe {
            let ref mut value = node.value;

            // Construct the key here to avoid the borrow checked
            let key_ptr = node.key_ptr;
            let key_len = node.key_len;
            let key = str::from_utf8_unchecked(
                slice::from_raw_parts(key_ptr, key_len)
            );

            if !self.end {
                self.back = Some(&mut *node.prev);
            }

            Some((key, value))
        }
    }
}
