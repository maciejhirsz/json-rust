use std::{ ptr, mem, str, slice, fmt };

use value::JsonValue;

const KEY_BUF_LEN: usize = 30;

struct Node {
    // Internal buffer to store keys that fit within `KEY_BUF_LEN`,
    // otherwise this field will contain garbage.
    pub key_buf: [u8; KEY_BUF_LEN],

    // Length of the key in bytes.
    pub key_len: usize,

    // Cached raw pointer to the key, so that we can cheaply construct
    // a `&str` slice from the `Node` without checking if the key is
    // allocated separately on the heap, or in the `key_buf`.
    pub key_ptr: *mut u8,

    // A hash of the key, explanation below.
    pub key_hash: u64,

    // Value stored.
    pub value: JsonValue,

    // Store vector index pointing to the `Node` for which `key_hash` is smaller
    // than that of this `Node`.
    // Will default to 0 as root node can't be referrenced anywhere else.
    pub left: usize,

    // Same as above but for `Node`s with hash larger than this one. If the
    // hash is the same, but keys are different, the lookup will default
    // to the right branch as well.
    pub right: usize,
}

impl fmt::Debug for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&(self.key_str(), &self.value, self.left, self.right), f)
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Node) -> bool {
        self.key_hash == other.key_hash &&
        self.key()    == other.key()    &&
        self.value    == other.value
    }
}

// Because `Node` contains a raw pointer, `Sync` marker is missing. This
// in turn disables `Sync` for `Object`, and eventually `JsonValue`. Without
// the `Sync` marker it's impossible to create a static `JsonValue`, which
// would break all the API that returns `&'static JsonValue::Null`.
//
// Since `Node` is not exposed anywhere in the API on it's own, and we manage
// heap of long keys manually, we just need to tell the compiler we know what
// we are doing here.
unsafe impl Sync for Node { }

// FNV-1a implementation
//
// While the `Object` is implemented as a binary tree, not a hash table, the
// order in which the tree is balanced makes absolutely no difference as long
// as there is a deterministic left / right ordering with good spread.
// Comparing a hashed `u64` is faster than comparing `&str` or even `&[u8]`,
// for larger objects this yields non-trivial performance benefits.
//
// Additionally this "randomizes" the keys a bit. Should the keys in an object
// be inserted in alphabetical order (an example of such a use case would be
// using an object as a store for entires by ids, where ids are sorted), this
// will prevent the tree from being constructed in a way where the same branch
// of each node is always used, effectively producing linear lookup times. Bad!
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
// 15043794053238616431  <-- 2nd
// 15043792953726988220  <-- 1st
// 15043800650308385697  <-- 4th
// 15043799550796757486  <-- 3rd
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

    // While `new` crates a fresh `Node` instance, it cannot do much about
    // the `key_*` fields. In case of short keys that can be stored on the
    // `Node`, only once the `Node` is somewhere on the heap, a persisting
    // pointer to the key can be obtained.
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

    // Since `Node`s are stored on a `Vec<Node>`, they will suffer from
    // reallocation, changing `key_ptr` addresses for buffered keys. This
    // needs to be called on each `Node` after each reallocation.
    #[inline(always)]
    fn fix_key_ptr(&mut self) {
        if self.key_len <= KEY_BUF_LEN {
            self.key_ptr = self.key_buf.as_mut_ptr();
        }
    }
}

// Because long keys _can_ be stored separately from the `Node` on heap,
// it's essential to clean up the heap allocation when the `Node` is dropped.
impl Drop for Node {
    fn drop(&mut self) {
        unsafe {
            if self.key_len > KEY_BUF_LEN {
                // Construct a `Vec` out of the `key_ptr`. Since the key is
                // always allocated from a slice, the capacity is equal to length.
                let heap = Vec::from_raw_parts(
                    self.key_ptr,
                    self.key_len,
                    self.key_len
                );

                // Now that we have an owned `Vec<u8>`, drop it.
                drop(heap);
            }
        }
    }
}

// Just like with `Drop`, `Clone` needs a custom implementation that accounts
// for the fact that key _can_ be separately heap allcated.
impl Clone for Node {
    fn clone(&self) -> Self {
        unsafe {
            if self.key_len > KEY_BUF_LEN {
                let mut heap = self.key().to_vec();
                let ptr = heap.as_mut_ptr();
                mem::forget(heap);

                Node {
                    key_buf: mem::uninitialized(),
                    key_len: self.key_len,
                    key_ptr: ptr,
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

/// A binary tree implementation of a string -> `JsonValue` map. You normally don't
/// have to interact with instances of `Object`, much more likely you will be
/// using the `JsonValue::Object` variant, which wraps around this struct.
#[derive(Debug)]
pub struct Object {
    store: Vec<Node>
}

impl Object {
    /// Create a new, empty instance of `Object`. Empty `Object` performs no
    /// allocation until a value is inserted into it.
    #[inline(always)]
    pub fn new() -> Self {
        Object {
            store: Vec::new()
        }
    }

    /// Create a new `Object` with memory preallocated for `capacity` number
    /// of entries.
    #[inline(always)]
    pub fn with_capacity(capacity: usize) -> Self {
        Object {
            store: Vec::with_capacity(capacity)
        }
    }

    #[inline(always)]
    fn node_at_index<'a>(&self, index: usize) -> &'a Node {
        unsafe {
            &*self.store.as_ptr().offset(index as isize)
        }
    }

    #[inline(always)]
    fn node_at_index_mut<'a>(&mut self, index: usize) -> &'a mut Node {
        unsafe {
            &mut *self.store.as_mut_ptr().offset(index as isize)
        }
    }

    #[inline(always)]
    fn add_node(&mut self, key: &[u8], value: JsonValue, hash: u64) -> usize {
        let index = self.store.len();

        if index < self.store.capacity() {
            // Because we've just checked the capacity, we can avoid
            // using `push`, and instead do unsafe magic to memcpy
            // the new node at the correct index without additional
            // capacity or bound checks.
            unsafe {
                let node = Node::new(value);
                self.store.set_len(index + 1);
                ptr::copy_nonoverlapping(
                    &node as *const Node,
                    self.store.as_mut_ptr().offset(index as isize),
                    1,
                );
                // Since the Node has been copied, we need to forget about
                // the owned value, else we may run into use after free.
                mem::forget(node);
            }
            self.node_at_index_mut(index).attach_key(key, hash);
        } else {
            self.store.push(Node::new(value));
            self.node_at_index_mut(index).attach_key(key, hash);

            for i in 0 .. index {
                self.node_at_index_mut(i).fix_key_ptr();
            }
        }

        index
    }

    /// Insert a new entry, or override an existing one. Note that `key` has
    /// to be a `&str` slice and not an owned `String`. The internals of
    /// `Object` will handle the heap allocation of the key if needed for
    /// better performance.
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

    /// Attempts to remove the value behind `key`, if successful
    /// will return the `JsonValue` stored behind the `key`.
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

        // Removing a node would screw the tree badly, it's easier to just
        // recreate it. This is a very costly operation, but removing nodes
        // in JSON shouldn't happen very often if at all. Optimizing this
        // can wait for better times.
        let mut new_object = Object::with_capacity(self.store.len() - 1);
        let mut removed: JsonValue = unsafe { mem::uninitialized() };

        for (i, node) in self.store.iter_mut().enumerate() {
            if i == index {
                // Rust doesn't like us moving things from `node`, even if
                // it is owned. Replace fixes that.
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

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.store.len()
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.store.is_empty()
    }

    /// Wipe the `Object` clear. The capacity will remain untouched.
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

// Custom implementation of `Clone`, as new heap allocation means
// we have to fix key pointers everywhere!
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

// Because keys can inserted in different order, the safe way to
// compare `Object`s is to iterate over one and check if the other
// has all the same keys.
impl PartialEq for Object {
    fn eq(&self, other: &Object) -> bool {
        if self.len() != other.len() {
            return false;
        }

        for (key, value) in self.iter() {
            match other.get(key) {
                Some(ref other_val) => if *other_val != value { return false; },
                None                => return false
            }
        }

        true
    }
}

pub struct Iter<'a> {
    inner: slice::Iter<'a, Node>
}

impl<'a> Iter<'a> {
    /// Create an empty iterator that always returns `None`
    pub fn empty() -> Self {
        Iter {
            inner: [].iter()
        }
    }
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

impl<'a> IterMut<'a> {
    /// Create an empty iterator that always returns `None`
    pub fn empty() -> Self {
        IterMut {
            inner: [].iter_mut()
        }
    }
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
