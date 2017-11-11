use std::fmt;

use json::JsonValue;
use cell::CopyCell;
use arena::Arena;

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

#[derive(Clone, Copy)]
struct Node<'arena> {
    pub key: &'arena str,
    pub hash: u64,
    pub value: CopyCell<&'arena JsonValue<'arena>>,
    pub left: CopyCell<Option<&'arena Node<'arena>>>,
    pub right: CopyCell<Option<&'arena Node<'arena>>>,
}

impl<'arena> fmt::Debug for Node<'arena> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&(self.key, &self.value), f)
    }
}

impl<'arena> PartialEq for Node<'arena> {
    fn eq(&self, other: &Node<'arena>) -> bool {
        self.hash  == other.hash &&
        self.key   == other.key  &&
        self.value == other.value
    }
}

impl<'arena> Node<'arena> {
    #[inline]
    pub fn new(key: &'arena str, hash: u64, value: &'arena JsonValue<'arena>) -> Self {
        Node {
            key,
            hash,
            value: CopyCell::new(value),
            left: CopyCell::new(None),
            right: CopyCell::new(None),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Object<'arena> {
    root: CopyCell<Option<&'arena Node<'arena>>>,
}

impl<'arena> Object<'arena> {
    /// Create a new, empty instance of `Object`.
    #[inline]
    pub fn new() -> Self {
        Object {
            root: CopyCell::new(None),
        }
    }

    #[inline]
    fn find_slot(&self, key: &str, hash: u64) -> &CopyCell<Option<&'arena Node<'arena>>> {
        let mut node = &self.root;

        loop {
            match node.get() {
                None         => return node,
                Some(parent) => {
                    if hash == parent.hash && key == parent.key {
                        return node;
                    } else if hash < parent.hash {
                        node = &parent.left;
                    } else {
                        node = &parent.right;
                    }
                }
            }
        }
    }

    #[inline]
    pub fn insert(&self, arena: &'arena Arena, key: &str, value: JsonValue<'arena>) {
        let key = arena.alloc_str(key);
        let value = arena.alloc(value);

        self.insert_allocated(arena, key, value);
    }

    #[inline]
    pub fn insert_allocated(&self, arena: &'arena Arena, key: &'arena str, value: &'arena JsonValue<'arena>) {
        let hash = hash_key(key.as_bytes());
        let node = self.find_slot(key, hash);

        match node.get() {
            Some(node) => node.value.set(value),
            None => {
                let new = arena.alloc(Node::new(key, hash, value));
                node.set(Some(new));
            }
        }
    }

    pub fn get(&self, key: &str) -> Option<&'arena JsonValue<'arena>> {
        let hash = hash_key(key.as_bytes());
        let node = self.find_slot(key, hash);

        node.get().map(|node| node.value.get())
    }


    // /// Attempts to remove the value behind `key`, if successful
    // /// will return the `JsonValue` stored behind the `key`.
    // pub fn remove(&mut self, key: &str) -> Option<JsonValue> {
    //     if self.store.len() == 0 {
    //         return None;
    //     }

    //     let key = key.as_bytes();
    //     let hash = hash_key(key);
    //     let mut index = 0;

    //     {
    //         let mut node = unsafe { self.store.get_unchecked(0) };

    //         // Try to find the node
    //         loop {
    //             if hash == node.key.hash && key == node.key.as_bytes() {
    //                 break;
    //             } else if hash < node.key.hash {
    //                 if node.left == 0 {
    //                     return None;
    //                 }
    //                 index = node.left;
    //                 node = unsafe { self.store.get_unchecked(node.left) };
    //             } else {
    //                 if node.right == 0 {
    //                     return None;
    //                 }
    //                 index = node.right;
    //                 node = unsafe { self.store.get_unchecked(node.right) };
    //             }
    //         }
    //     }

    //     // Removing a node would screw the tree badly, it's easier to just
    //     // recreate it. This is a very costly operation, but removing nodes
    //     // in JSON shouldn't happen very often if at all. Optimizing this
    //     // can wait for better times.
    //     let mut new_object = Object::with_capacity(self.store.len() - 1);
    //     let mut removed = None;

    //     for (i, node) in self.store.iter_mut().enumerate() {
    //         if i == index {
    //             // Rust doesn't like us moving things from `node`, even if
    //             // it is owned. Replace fixes that.
    //             removed = Some(mem::replace(&mut node.value, JsonValue::Null));
    //         } else {
    //             let value = mem::replace(&mut node.value, JsonValue::Null);

    //             new_object.insert(node.key.as_str(), value);
    //         }
    //     }

    //     mem::swap(self, &mut new_object);

    //     removed
    // }

    // #[inline(always)]
    // pub fn len(&self) -> usize {
    //     self.store.len()
    // }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.root.get().is_none()
    }

    /// Wipe the `Object` clear. The capacity will remain untouched.
    pub fn clear(&self) {
        self.root.set(None);
    }

    // #[inline(always)]
    // pub fn iter(&self) -> Iter {
    //     Iter {
    //         inner: self.store.iter()
    //     }
    // }

    // #[inline(always)]
    // pub fn iter_mut(&mut self) -> IterMut {
    //     IterMut {
    //         inner: self.store.iter_mut()
    //     }
    // }

    // /// Prints out the value as JSON string.
    // pub fn dump(&self) -> String {
    //     let mut gen = DumpGenerator::new();
    //     gen.write_object(self).expect("Can't fail");
    //     gen.consume()
    // }

    // /// Pretty prints out the value as JSON string. Takes an argument that's
    // /// number of spaces to indent new blocks with.
    // pub fn pretty(&self, spaces: u16) -> String {
    //     let mut gen = PrettyGenerator::new(spaces);
    //     gen.write_object(self).expect("Can't fail");
    //     gen.consume()
    // }
}
