use std::{ ptr, mem, str, slice, fmt };

use value::JsonValue;

const KEY_BUF_LEN: usize = 30;

struct Node {
    pub key_buf: [u8; KEY_BUF_LEN],
    pub key_len: usize,
    pub key_ptr: *mut u8,
    pub key_hash: u64,
    pub value: JsonValue,
    pub left: usize,
    pub right: usize,
}

impl fmt::Debug for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&(self.key_str(), self.left, self.right), f)
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Node) -> bool {
        self.key() == other.key() && self.value == other.value
    }
}

unsafe impl Sync for Node { }

// Simple FNV 1a implementation
//
// While the `Object` is implemented as a binary tree, not a hash table, the
// order in which the tree is balanced makes absolutely no difference as long
// as there is a good chance there is a left or right side to the equation.
// Comparing a hashed `u64` is faster than comparing `&str` or even `&[u8]`,
// for larger objects this yields non-trivial performance benefits.
//
// Additionally this "randomizes" the keys a bit. Should the keys in an object
// be inserted in alphabetical order (an example of such a use case would be
// using an object as a store for entires by ids, where ids are sorted), this
// will prevent the tree for being constructed in a way that only right branch
// of a node is always used, effectively producing linear lookup times. Bad!
// Using this solution fixes that problem.
//
// Example:
//
// ```
// println!("{}", hash_key(b"10000056"));
// println!("{}", hash_key(b"10000057"));
// println!("{}", hash_key(b"10000058"));
// println!("{}", hash_key(b"10000059"));
// ```
//
// Produces:
//
// ```
// 15043794053238616431
// 15043792953726988220
// 15043800650308385697
// 15043799550796757486
// ```
#[inline(always)]
fn hash_key(key: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in key {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

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
                key_buf: mem::uninitialized(),
                key_len: 0,
                key_ptr: mem::uninitialized(),
                key_hash: mem::uninitialized(),
                value: value,
                left: 0,
                right: 0,
            }
        }
    }

    #[inline(always)]
    fn attach_key(&mut self, key: &[u8], hash: u64) {
        self.key_len = key.len();
        self.key_hash = hash;
        if key.len() <= KEY_BUF_LEN {
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
        if self.key_len <= KEY_BUF_LEN {
            self.key_ptr = self.key_buf.as_mut_ptr();
        }
    }
}

impl Drop for Node {
    fn drop(&mut self) {
        unsafe {
            if self.key_len > KEY_BUF_LEN {
                let heap = Vec::from_raw_parts(self.key_ptr, self.key_len, self.key_len);
                drop(heap);
            }
        }
    }
}

impl Clone for Node {
    fn clone(&self) -> Self {
        unsafe {
            if self.key_len > KEY_BUF_LEN {
                let heap = Vec::from_raw_parts(self.key_ptr, self.key_len, self.key_len);
                let mut cloned = heap.clone();
                mem::forget(heap);

                Node {
                    key_buf: mem::uninitialized(),
                    key_len: self.key_len,
                    key_ptr: cloned.as_mut_ptr(),
                    key_hash: self.key_hash,
                    value: self.value.clone(),
                    left: self.left,
                    right: self.right,
                }
            } else {
                Node {
                    key_buf: self.key_buf,
                    key_len: self.key_len,
                    key_ptr: mem::uninitialized(),
                    key_hash: self.key_hash,
                    value: self.value.clone(),
                    left: self.left,
                    right: self.right,
                }
            }
        }
    }
}

#[derive(Debug, PartialEq)]
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
    fn add_node(&mut self, key: &[u8], value: JsonValue, hash: u64) -> usize {
        let index = self.store.len();

        if index < self.store.capacity() {
            self.store.push(Node::new(value));
            self.store[index].attach_key(key, hash);
        } else {
            self.store.push(Node::new(value));
            self.store[index].attach_key(key, hash);

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
        let hash = hash_key(key);

        if self.store.len() == 0 {
            self.store.push(Node::new(value));
            self.store[0].attach_key(key, hash);
            return;
        }

        let mut node = self.node_at_index_mut(0);
        let mut parent = 0;

        loop {
            if hash == node.key_hash && key == node.key() {
                node.value = value;
                return;
            } else if hash < node.key_hash {
                if node.left != 0 {
                    parent = node.left;
                    node = self.node_at_index_mut(node.left);
                    continue;
                }
                self.store[parent].left = self.add_node(key, value, hash);
                return;
            } else {
                if node.right != 0 {
                    parent = node.right;
                    node = self.node_at_index_mut(node.right);
                    continue;
                }
                self.store[parent].right = self.add_node(key, value, hash);
                return;
            }
        }
    }

    pub fn get(&self, key: &str) -> Option<&JsonValue> {
        if self.store.len() == 0 {
            return None;
        }

        let key = key.as_bytes();
        let hash = hash_key(key);

        let mut node = self.node_at_index(0);

        loop {
            if hash == node.key_hash && key == node.key() {
                return Some(&node.value);
            } else if hash < node.key_hash {
                if node.left == 0 {
                    return None;
                }
                node = self.node_at_index(node.left);
            } else {
                if node.right == 0 {
                    return None;
                }
                node = self.node_at_index(node.right);
            }
        }
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut JsonValue> {
        if self.store.len() == 0 {
            return None;
        }

        let key = key.as_bytes();
        let hash = hash_key(key);

        let mut node = self.node_at_index_mut(0);

        loop {
            if hash == node.key_hash && key == node.key() {
                return Some(&mut node.value);
            } else if hash < node.key_hash {
                if node.left == 0 {
                    return None;
                }
                node = self.node_at_index_mut(node.left);
            } else {
                if node.right == 0 {
                    return None;
                }
                node = self.node_at_index_mut(node.right);
            }
        }
    }

    pub fn remove(&mut self, key: &str) -> Option<JsonValue> {
        if self.store.len() == 0 {
            return None;
        }

        let key = key.as_bytes();
        let hash = hash_key(key);

        let mut node = self.node_at_index_mut(0);
        let mut index = 0;

        // Try to find the node
        loop {
            if hash == node.key_hash && key == node.key() {
                break;
            } else if hash < node.key_hash {
                if node.left == 0 {
                    return None;
                }
                index = node.left;
                node = self.node_at_index_mut(node.left);
            } else {
                if node.right == 0 {
                    return None;
                }
                index = node.right;
                node = self.node_at_index_mut(node.right);
            }
        }

        let mut new_object = Object::with_capacity(self.store.len() - 1);
        let mut removed: JsonValue = unsafe { mem::uninitialized() };

        for (i, node) in self.store.iter_mut().enumerate() {
            if i == index {
                removed = mem::replace(&mut node.value, JsonValue::Null);
            } else {
                new_object.insert(
                    node.key_str(),
                    mem::replace(&mut node.value, JsonValue::Null)
                );
            }
        }

        mem::swap(self, &mut new_object);

        Some(removed)
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

impl Clone for Object {
    fn clone(&self) -> Self {
        let mut store = self.store.clone();

        for node in store.iter_mut() {
            node.fix_key_ptr();
        }

        Object {
            store: store
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
