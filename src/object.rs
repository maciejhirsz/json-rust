use std::{mem, str, slice, fmt};
use std::borrow::Borrow;
use std::num::NonZeroU32;
use std::iter::FromIterator;
use std::cell::Cell;
use std::hash::{Hash, Hasher};

use beef::lean::Cow;

use crate::vec::Vec;
use crate::value::JsonValue;

#[inline]
fn hash_key<H: Hash>(hash: H) -> u64 {
    let mut hasher = fnv::FnvHasher::default();
    // let mut hasher = ahash::AHasher::default();

    hash.hash(&mut hasher);

    hasher.finish()
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

pub type Object<'json> = Map<Cow<'json, str>, JsonValue<'json>>;

/// A binary tree implementation of a string -> `JsonValue` map. You normally don't
/// have to interact with instances of `Object`, much more likely you will be
/// using the `JsonValue::Object` variant, which wraps around this struct.
#[derive(Debug, Clone)]
pub struct Map<K, V> {
    store: Vec<Node<K, V>>
}

enum FindResult<'find> {
    Hit(usize),
    Miss(Option<&'find Cell<Option<NonZeroU32>>>),
}

use FindResult::*;

impl<K, V> Map<K, V> {
    /// Create a new `Map`.
    #[inline]
    pub fn new() -> Self {
        Map {
            store: Vec::new()
        }
    }

    /// Create a `Map` with a given capacity
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Map {
            store: Vec::with_capacity(capacity)
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.store.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.store.is_empty()
    }

    pub fn clear(&mut self) {
        self.store.clear();
    }
}

impl<K, V> Map<K, V>
where
    K: Hash + Eq,
{
    /// Insert a new entry, or override an existing one.
    pub fn insert<Q>(&mut self, key: Q, value: V)
    where
        Q: Into<K>,
    {
        let key = key.into();
        let hash = hash_key(&key);

        match self.find(&key, hash) {
            Hit(idx) => {
                self.store[idx].value = value;
            },
            Miss(parent) => {
                if let Some(parent) = parent {
                    parent.set(NonZeroU32::new(self.store.len() as u32));
                }

                self.store.push(Node::new(key, value, hash));
            },
        }
    }

    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let hash = hash_key(key);

        match self.find(key, hash) {
            Hit(idx) => Some(&self.store[idx].value),
            Miss(_) => None,
        }
    }

    pub fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let hash = hash_key(key);

        match self.find(key, hash) {
            Hit(idx) => Some(&mut self.store[idx].value),
            Miss(_) => None,
        }
    }

    pub fn get_or_insert<Q, F>(&mut self, key: Q, fill: F) -> &mut V
    where
        Q: Into<K>,
        F: FnOnce() -> V,
    {
        let key = key.into();
        let hash = hash_key(&key);

        match self.find(&key, hash) {
            Hit(idx) => &mut self.store[idx].value,
            Miss(parent) => {
                let idx = self.store.len();

                if let Some(parent) = parent {
                    parent.set(NonZeroU32::new(self.store.len() as u32));
                }

                self.store.push(Node::new(key, fill(), hash));

                &mut self.store[idx].value
            },
        }
    }

    /// Attempts to remove the value behind `key`, if successful
    /// will return the `JsonValue` stored behind the `key`.
    pub fn remove<Q>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let hash = hash_key(key);

        let index = match self.find(key, hash) {
            Hit(idx) => idx,
            Miss(_) => return None,
        };

        // Removing a node would screw the tree badly, it's easier to just
        // recreate it. This is a very costly operation, but removing nodes
        // in JSON shouldn't happen very often if at all. Optimizing this
        // can wait for better times.
        let mut removed = None;
        let capacity = self.store.len();
        let old = mem::replace(&mut self.store, Vec::with_capacity(capacity));

        for (i, Node { key, value, hash, .. }) in old.into_iter().enumerate() {
            if i == index {
                // Rust doesn't like us moving things from `node`, even if
                // it is owned. Replace fixes that.
                removed = Some(value);
            } else {
                // Faster than .insert() since we can avoid hashing
                if let Miss(Some(parent)) = self.find(key.borrow(), hash) {
                    parent.set(NonZeroU32::new(self.store.len() as u32));
                }

                self.store.push(Node::new(key, value, hash));
            }
        }

        removed
    }

    #[inline]
    fn find<Q: ?Sized>(&self, key: &Q, hash: u64) -> FindResult
    where
        K: Borrow<Q>,
        Q: Eq,
    {
        if self.store.len() == 0 {
            return Miss(None);
        }

        let mut idx = 0;

        loop {
            let node = unsafe { self.store.get_unchecked(idx) };

            if hash < node.hash {
                match node.left.get() {
                    Some(i) => idx = i.get() as usize,
                    None => return Miss(Some(&node.left)),
                }
            } else if hash > node.hash {
                match node.right.get() {
                    Some(i) => idx = i.get() as usize,
                    None => return Miss(Some(&node.right)),
                }
            } else {
                return Hit(idx);
            }
        }
    }

    #[inline]
    pub fn iter(&self) -> Iter<K, V> {
        Iter {
            inner: self.store.iter()
        }
    }

    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<K, V> {
        IterMut {
            inner: self.store.iter_mut()
        }
    }
}

impl<'json, IK, IV, K, V> FromIterator<(IK, IV)> for Map<K, V>
where
    IK: Into<K>,
    IV: Into<V>,
    K: Hash + Eq,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item=(IK, IV)>,
    {
        let iter = iter.into_iter();
        let mut map = Map::with_capacity(iter.size_hint().0);

        for (key, value) in iter {
            map.insert(key, value.into());
        }

        map
    }
}

// Because keys can inserted in different order, the safe way to
// compare `Map`s is to iterate over one and check if the other
// has all the same keys.
impl<K, V> PartialEq for Map<K, V>
where
    K: Hash + Eq,
    V: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        if self.len() != other.len() {
            return false;
        }

        // Faster than .get() since we can avoid hashing
        for &Node { ref key, ref value, hash, .. } in self.store.iter() {
            if let Hit(idx) = other.find(key, hash) {
                if &other.store[idx].value == value {
                    continue;
                }
            }

            return false;
        }

        true
    }
}

pub struct Iter<'a, K, V> {
    inner: slice::Iter<'a, Node<K, V>>,
}

pub struct IterMut<'a, K, V> {
    inner: slice::IterMut<'a, Node<K, V>>,
}

impl<K, V> Iter<'_, K, V> {
    /// Create an empty iterator that always returns `None`
    pub fn empty() -> Self {
        Iter {
            inner: [].iter()
        }
    }
}

impl<'i, K, V> Iterator for Iter<'i, K, V> {
    type Item = (&'i K, &'i V);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|node| (&node.key, &node.value))
    }
}

impl<K, V> DoubleEndedIterator for Iter<'_, K, V> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back().map(|node| (&node.key, &node.value))
    }
}

impl<K, V> ExactSizeIterator for Iter<'_, K, V> {
    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl<K, V> IterMut<'_, K, V> {
    /// Create an empty iterator that always returns `None`
    pub fn empty() -> Self {
        IterMut {
            inner: [].iter_mut()
        }
    }
}

impl<'a, K, V> Iterator for IterMut<'a, K, V> {
    type Item = (&'a K, &'a mut V);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|node| (&node.key, &mut node.value))
    }
}

impl<K, V> DoubleEndedIterator for IterMut<'_, K, V> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back().map(|node| (&node.key, &mut node.value))
    }
}

impl<K, V> ExactSizeIterator for IterMut<'_, K, V> {
    fn len(&self) -> usize {
        self.inner.len()
    }
}
