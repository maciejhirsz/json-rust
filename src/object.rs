use std::{mem, str, slice, fmt};
use std::ops::{Index, IndexMut, Deref};
use std::num::NonZeroU32;
use std::iter::FromIterator;
use std::cell::Cell;
use beef::Cow;

use crate::codegen::{ DumpGenerator, Generator, PrettyGenerator };
use crate::value::JsonValue;

const NULL: JsonValue<'static> = JsonValue::Null;

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
// using an object as a store for entries by ids, where ids are sorted), this
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
#[inline]
fn hash_key(key: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in key {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

#[derive(Clone)]
struct Node<K, V> {
    // Key
    pub key: K,

    // Hash of the key
    pub hash: u64,

    // Value stored.
    pub value: V,

    // Store vector index pointing to the `Node` for which `hash` is smaller
    // than that of this `Node`.
    pub left: Cell<Option<NonZeroU32>>,

    // Same as above but for `Node`s with hash larger than this one. If the
    // hash is the same, but keys are different, the lookup will default
    // to the right branch as well.
    pub right: Cell<Option<NonZeroU32>>,
}

impl<K, V> fmt::Debug for Node<K, V>
where
    K: fmt::Debug,
    V: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&(&self.key, &self.value, self.left.get(), self.right.get()), f)
    }
}

impl<K, V> PartialEq for Node<K, V>
where
    K: PartialEq,
    V: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.hash == other.hash &&
        self.key == other.key &&
        self.value == other.value
    }
}

impl<K, V> Node<K, V> {
    #[inline]
    const fn new(key: K, value: V, hash: u64) -> Self {
        Node {
            key,
            hash,
            value,
            left: Cell::new(None),
            right: Cell::new(None),
        }
    }
}

// `Cell` isn't `Sync`, but all of our writes are contained and require
// `&mut` access, ergo this is safe.
unsafe impl<K: Sync, V: Sync> Sync for Node<K, V> {}

/// A binary tree implementation of a string -> `JsonValue` map. You normally don't
/// have to interact with instances of `Object`, much more likely you will be
/// using the `JsonValue::Object` variant, which wraps around this struct.
#[derive(Debug, Clone)]
pub struct Object<'json> {
    store: Vec<Node<Cow<'json, str>, JsonValue<'json>>>
}

enum FindResult<'find> {
    Hit(usize),
    Miss(Option<&'find Cell<Option<NonZeroU32>>>),
}

impl<'json> Object<'json> {
    /// Create a new, empty instance of `Object`. Empty `Object` performs no
    /// allocation until a value is inserted into it.
    #[inline]
    pub fn new() -> Self {
        Object {
            store: Vec::new()
        }
    }

    /// Create a new `Object` with memory preallocated for `capacity` number
    /// of entries.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Object {
            store: Vec::with_capacity(capacity)
        }
    }

    /// Insert a new entry, or override an existing one.
    pub fn insert<K>(&mut self, key: K, value: JsonValue<'json>)
    where
        K: Into<Cow<'json, str>>,
    {
        let key = key.into();
        let bytes = key.as_bytes();
        let hash = hash_key(bytes);

        match self.find(bytes, hash) {
            FindResult::Hit(idx) => {
                self.store[idx].value = value;
            },
            FindResult::Miss(parent) => {
                if let Some(parent) = parent {
                    parent.set(NonZeroU32::new(self.store.len() as u32));
                }

                self.store.push(Node::new(key, value, hash));
            },
        }
    }

    #[inline]
    fn find(&self, bytes: &[u8], hash: u64) -> FindResult {
        let mut idx = 0;

        while let Some(node) = self.store.get(idx) {
            if hash == node.hash && bytes == node.key.as_bytes() {
                return FindResult::Hit(idx);
            } else if hash < node.hash {
                match node.left.get() {
                    Some(i) => idx = i.get() as usize,
                    None => return FindResult::Miss(Some(&node.left)),
                }
            } else {
                match node.right.get() {
                    Some(i) => idx = i.get() as usize,
                    None => return FindResult::Miss(Some(&node.right)),
                }
            }
        }

        FindResult::Miss(None)
    }

    pub fn get(&self, key: &str) -> Option<&JsonValue<'json>> {
        let bytes = key.as_bytes();
        let hash = hash_key(bytes);

        match self.find(bytes, hash) {
            FindResult::Hit(idx) => Some(&self.store[idx].value),
            FindResult::Miss(_) => None,
        }
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut JsonValue<'json>> {
        let bytes = key.as_bytes();
        let hash = hash_key(bytes);

        match self.find(bytes, hash) {
            FindResult::Hit(idx) => Some(&mut self.store[idx].value),
            FindResult::Miss(_) => None,
        }
    }

    /// Attempts to remove the value behind `key`, if successful
    /// will return the `JsonValue` stored behind the `key`.
    pub fn remove(&mut self, key: &str) -> Option<JsonValue<'json>> {
        let bytes = key.as_bytes();
        let hash = hash_key(bytes);
        let index = match self.find(bytes, hash) {
            FindResult::Hit(idx) => idx,
            FindResult::Miss(_) => return None,
        };

        // Removing a node would screw the tree badly, it's easier to just
        // recreate it. This is a very costly operation, but removing nodes
        // in JSON shouldn't happen very often if at all. Optimizing this
        // can wait for better times.
        let mut removed = None;
        let capacity = self.store.len();
        let old_store = mem::replace(&mut self.store, Vec::with_capacity(capacity));

        for (i, node) in old_store.into_iter().enumerate() {
            if i == index {
                // Rust doesn't like us moving things from `node`, even if
                // it is owned. Replace fixes that.
                removed = Some(node.value);
            } else {
                self.insert(node.key, node.value);
            }
        }

        removed
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.store.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.store.is_empty()
    }

    /// Wipe the `Object` clear. The capacity will remain untouched.
    pub fn clear(&mut self) {
        self.store.clear();
    }

    #[inline]
    pub fn iter(&self) -> Iter {
        Iter {
            inner: self.store.iter()
        }
    }

    #[inline]
    pub fn iter_mut<'iter: 'json>(&'iter mut self) -> IterMut<'iter> {
        IterMut {
            inner: self.store.iter_mut()
        }
    }

    /// Prints out the value as JSON string.
    pub fn dump(&self) -> String {
        let mut gen = DumpGenerator::new();
        gen.write_object(self).expect("Can't fail");
        gen.consume()
    }

    /// Pretty prints out the value as JSON string. Takes an argument that's
    /// number of spaces to indent new blocks with.
    pub fn pretty(&self, spaces: u16) -> String {
        let mut gen = PrettyGenerator::new(spaces);
        gen.write_object(self).expect("Can't fail");
        gen.consume()
    }
}

impl<'json, K, V> FromIterator<(K, V)> for Object<'json>
where
    K: Into<Cow<'json, str>> + 'json,
    V: Into<JsonValue<'json>>,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item=(K, V)>,
    {
        let iter = iter.into_iter();
        let mut object = Object::with_capacity(iter.size_hint().0);

        for (key, value) in iter {
            object.insert(key, value.into());
        }

        object
    }
}

// Because keys can inserted in different order, the safe way to
// compare `Object`s is to iterate over one and check if the other
// has all the same keys.
impl<'json> PartialEq for Object<'json> {
    fn eq(&self, other: &Object<'json>) -> bool {
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
    inner: slice::Iter<'a, Node<Cow<'a, str>, JsonValue<'a>>>
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
    type Item = (&'a str, &'a JsonValue<'a>);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|node| (node.key.deref(), &node.value))
    }
}

impl<'a> DoubleEndedIterator for Iter<'a> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back().map(|node| (node.key.deref(), &node.value))
    }
}

impl<'a> ExactSizeIterator for Iter<'a> {
    fn len(&self) -> usize {
        self.inner.len()
    }
}

pub struct IterMut<'a> {
    inner: slice::IterMut<'a, Node<Cow<'a, str>, JsonValue<'a>>>,
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
    type Item = (&'a str, &'a mut JsonValue<'a>);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|node| (node.key.deref(), &mut node.value))
    }
}

impl<'a> DoubleEndedIterator for IterMut<'a> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back().map(|node| (node.key.deref(), &mut node.value))
    }
}

impl<'a> ExactSizeIterator for IterMut<'a> {
    fn len(&self) -> usize {
        self.inner.len()
    }
}

/// Implements indexing by `&str` to easily access object members:
///
/// ## Example
///
/// ```
/// # #[macro_use]
/// # extern crate json;
/// # use json::JsonValue;
/// #
/// # fn main() {
/// let value = object!{
///     foo: "bar"
/// };
///
/// if let JsonValue::Object(object) = value {
///   assert!(object["foo"] == "bar");
/// }
/// # }
/// ```
// TODO: doc
impl<'json> Index<&str> for Object<'json> {
    type Output = JsonValue<'json>;

    fn index(&self, index: &str) -> &JsonValue<'json> {
        match self.get(index) {
            Some(value) => value,
            _ => &NULL
        }
    }
}

impl<'json> Index<String> for Object<'json> {
    type Output = JsonValue<'json>;

    fn index(&self, index: String) -> &JsonValue<'json> {
        match self.get(&index) {
            Some(value) => value,
            _ => &NULL
        }
    }
}

impl<'json> Index<&String> for Object<'json> {
    type Output = JsonValue<'json>;

    fn index(&self, index: &String) -> &JsonValue<'json> {
        match self.get(index) {
            Some(value) => value,
            _ => &NULL
        }
    }
}

/// Implements mutable indexing by `&str` to easily modify object members:
///
/// ## Example
///
/// ```
/// # #[macro_use]
/// # extern crate json;
/// # use json::JsonValue;
/// #
/// # fn main() {
/// let value = object!{};
///
/// if let JsonValue::Object(mut object) = value {
///   object["foo"] = 42.into();
///
///   assert!(object["foo"] == 42);
/// }
/// # }
/// ```
impl<'json> IndexMut<&str> for Object<'json> {
    fn index_mut(&mut self, index: &str) -> &mut JsonValue<'json> {
        if self.get(index).is_none() {
            self.insert(Cow::owned(index.to_owned()), JsonValue::Null);
        }
        self.get_mut(index).unwrap()
    }
}

impl<'json> IndexMut<String> for Object<'json> {
    fn index_mut(&mut self, index: String) -> &mut JsonValue<'json> {
        if self.get(&index).is_none() {
            self.insert(Cow::owned(index.clone()), JsonValue::Null);
        }
        self.get_mut(&index).unwrap()
    }
}

impl<'json> IndexMut<&String> for Object<'json> {
    fn index_mut(&mut self, index: &String) -> &mut JsonValue<'json> {
        if self.get(index).is_none() {
            self.insert(Cow::owned(index.clone()), JsonValue::Null);
        }
        self.get_mut(index).unwrap()
    }
}

