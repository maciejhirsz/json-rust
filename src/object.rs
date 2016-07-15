use std::{ ptr, mem, str, slice, fmt };

use value::JsonValue;

// FIXME: Manual Clone!
#[derive(PartialEq, Clone)]
struct Node {
    pub vacant: bool,
    pub key_buf: [u8; 16],
    pub key_len: usize,
    pub key_ptr: *mut u8,
    pub value: JsonValue,
    pub left: usize,
    pub right: usize,
}

impl fmt::Debug for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&(self.key_str(), self.left, self.right), f)
    }
}

unsafe impl Sync for Node { }

impl Node {
    #[inline(always)]
    fn key<'a>(&self) -> &'a [u8] {
        unsafe {
            slice::from_raw_parts(self.key_ptr, self.key_len)
        }
    }

    #[inline(always)]
    fn key_str<'a>(&self) -> &'a str {
        unsafe {
            str::from_utf8_unchecked(self.key())
        }
    }

    #[inline(always)]
    fn new(value: JsonValue) -> Node {
        unsafe {
            Node {
                vacant: false,
                key_buf: mem::uninitialized(),
                key_len: 0,
                key_ptr: mem::uninitialized(),
                value: value,
                left: 0,
                right: 0,
            }
        }
    }

    #[inline(always)]
    fn attach_key(&mut self, key: &[u8]) {
        self.key_len = key.len();
        if key.len() <= 16 {
            unsafe {
                ptr::copy_nonoverlapping(
                    key.as_ptr(),
                    self.key_buf.as_mut_ptr(),
                    key.len()
                );
            }
            self.key_ptr = self.key_buf.as_mut_ptr();
        } else {
            let mut heap: Vec<u8> = key.to_vec();
            self.key_ptr = heap.as_mut_ptr();
            mem::forget(heap);
        }
    }

    #[inline(always)]
    fn fix_key_ptr(&mut self) {
        if self.key_len <= 16 {
            self.key_ptr = self.key_buf.as_mut_ptr();
        }
    }
}

impl Drop for Node {
    fn drop(&mut self) {
        unsafe {
            if self.key_len > 16 {
                let heap = Vec::from_raw_parts(self.key_ptr, self.key_len, self.key_len);
                drop(heap);
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Object {
    store: Vec<Node>
}


impl Object {
    pub fn new() -> Self {
        Object {
            store: Vec::new()
        }
    }

    fn node_at_index<'a>(&self, index: usize) -> &'a Node {
        let store_ptr = self.store.as_ptr();
        unsafe {
            &*store_ptr.offset(index as isize)
        }
    }

    fn node_at_index_mut<'a>(&mut self, index: usize) -> &'a mut Node {
        let store_ptr = self.store.as_mut_ptr();
        unsafe {
            &mut *store_ptr.offset(index as isize)
        }
    }

    #[inline(always)]
    fn add_node(&mut self, key: &[u8], value: JsonValue) -> usize {
        let index = self.store.len();

        if index < self.store.capacity() {
            self.store.push(Node::new(value));
            self.store[index].attach_key(key);
        } else {
            self.store.push(Node::new(value));
            self.store[index].attach_key(key);

            // FIXME: don't fix the last element again
            for node in self.store.iter_mut() {
                node.fix_key_ptr();
            }
        }

        index
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Object {
            store: Vec::with_capacity(capacity)
        }
    }

    pub fn insert(&mut self, key: &str, value: JsonValue) {
        let key = key.as_bytes();

        if self.store.len() == 0 {
            self.store.push(Node::new(value));
            self.store[0].attach_key(key);
            return;
        }

        let mut node = self.node_at_index_mut(0);
        let mut parent = 0;

        loop {
            if key == node.key() {
                node.value = value;
                return;
            } else if key < node.key() {
                if node.left != 0 {
                    parent = node.left;
                    node = self.node_at_index_mut(node.left);
                    continue;
                }
                self.store[parent].left = self.add_node(key, value);
                return;
            } else {
                if node.right != 0 {
                    parent = node.right;
                    node = self.node_at_index_mut(node.right);
                    continue;
                }
                self.store[parent].right = self.add_node(key, value);
                return;
            }
        }
    }

    pub fn get(&self, key: &str) -> Option<&JsonValue> {
        if self.store.len() == 0 {
            return None;
        }

        let key = key.as_bytes();

        let mut node = self.node_at_index(0);

        loop {
            if key == node.key() {
                return if node.vacant {
                    None
                } else {
                    Some(&node.value)
                };
            } else if key < &node.key() {
                if node.left == 0 {
                    return None;
                }
                node = self.node_at_index(node.left);
            } else if key > &node.key() {
                if node.right == 0 {
                    return None;
                }
                node = self.node_at_index(node.right);
            } else {
                return None;
            }
        }
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut JsonValue> {
        if self.store.len() == 0 {
            return None;
        }

        let key = key.as_bytes();

        let mut node = self.node_at_index_mut(0);

        loop {
            if key == node.key() {
                return if node.vacant {
                    None
                } else {
                    Some(&mut node.value)
                };
            } else if key < &node.key() {
                if node.left == 0 {
                    return None;
                }
                node = self.node_at_index_mut(node.left);
            } else if key > &node.key() {
                if node.right == 0 {
                    return None;
                }
                node = self.node_at_index_mut(node.right);
            } else {
                return None;
            }
        }
    }

    pub fn len(&self) -> usize {
        self.store.len()
    }

    pub fn is_empty(&self) -> bool {
        self.store.is_empty()
    }

    pub fn clear(&mut self) {
        self.store.clear();
    }

    #[inline(always)]
    pub fn iter(&self) -> Iter {
        Iter {
            inner: self.store.iter()
        }
    }

    #[inline(always)]
    pub fn iter_mut(&mut self) -> IterMut {
        IterMut {
            inner: self.store.iter_mut()
        }
    }
}

pub struct Iter<'a> {
    inner: slice::Iter<'a, Node>
}

impl<'a> Iterator for Iter<'a> {
    type Item = (&'a str, &'a JsonValue);

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|node| (node.key_str(), &node.value))
    }
}

impl<'a> DoubleEndedIterator for Iter<'a> {
    #[inline(always)]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back().map(|node| (node.key_str(), &node.value))
    }
}

pub struct IterMut<'a> {
    inner: slice::IterMut<'a, Node>
}

impl<'a> Iterator for IterMut<'a> {
    type Item = (&'a str, &'a mut JsonValue);

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|node| (node.key_str(), &mut node.value))
    }
}

impl<'a> DoubleEndedIterator for IterMut<'a> {
    #[inline(always)]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back().map(|node| (node.key_str(), &mut node.value))
    }
}
