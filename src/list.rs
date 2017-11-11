use cell::CopyCell;
use std::fmt::{self, Debug};
use arena::Arena;

#[derive(Debug, PartialEq, Clone, Copy)]
struct ListItem<'arena, T: 'arena + Copy> {
    value: CopyCell<&'arena T>,
    next: CopyCell<Option<&'arena ListItem<'arena, T>>>,
}

pub struct ListBuilder<'arena, T: 'arena + Copy> {
    arena: &'arena Arena,
    first: &'arena ListItem<'arena, T>,
    last: &'arena ListItem<'arena, T>,
}

impl<'arena, T: 'arena + Copy> ListBuilder<'arena, T> {
    #[inline]
    pub fn new(arena: &'arena Arena, first: &'arena T) -> Self {
        let first = arena.alloc(ListItem {
            value: CopyCell::new(first),
            next: CopyCell::new(None)
        });

        ListBuilder {
            arena,
            first,
            last: first
        }
    }

    #[inline]
    pub fn push(&mut self, item: &'arena T) {
        let next = self.arena.alloc(ListItem {
            value: CopyCell::new(item),
            next: CopyCell::new(None)
        });

        self.last.next.set(Some(next));
        self.last = next;
    }

    #[inline]
    pub fn into_list(self) -> List<'arena, T> {
        List {
            root: CopyCell::new(Some(self.first))
        }
    }
}

pub struct EmptyListBuilder<'arena, T: 'arena + Copy> {
    arena: &'arena Arena,
    first: Option<&'arena ListItem<'arena, T>>,
    last: Option<&'arena ListItem<'arena, T>>,
}

impl<'arena, T: 'arena + Copy> EmptyListBuilder<'arena, T> {
    #[inline]
    pub fn new(arena: &'arena Arena) -> Self {
        EmptyListBuilder {
            arena,
            first: None,
            last: None,
        }
    }

    #[inline]
    pub fn push(&mut self, item: &'arena T) {
        match self.last {
            None => {
                self.first = Some(self.arena.alloc(ListItem {
                    value: CopyCell::new(item),
                    next: CopyCell::new(None)
                }));
                self.last = self.first;
            },
            Some(ref mut last) => {
                let next = self.arena.alloc(ListItem {
                    value: CopyCell::new(item),
                    next: CopyCell::new(None)
                });

                last.next.set(Some(next));
                *last = next;
            }
        }
    }

    #[inline]
    pub fn into_list(self) -> List<'arena, T> {
        List {
            root: CopyCell::new(self.first)
        }
    }
}

#[derive(Clone, Copy)]
pub struct List<'arena, T: 'arena + Copy> {
    root: CopyCell<Option<&'arena ListItem<'arena, T>>>,
}

impl<'arena, T: 'arena + PartialEq + Copy> PartialEq for List<'arena, T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.iter().eq(other.iter())
    }
}

impl<'arena, T: 'arena + Debug + Copy> Debug for List<'arena, T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

impl<'arena, T: 'arena + Copy> List<'arena, T> {
    #[inline]
    pub fn empty() -> Self {
        List {
            root: CopyCell::new(None)
        }
    }

    #[inline]
    pub fn clear(&self) {
        self.root.set(None);
    }

    #[inline]
    pub fn iter(&self) -> ListIter<'arena, T> {
        ListIter {
            next: self.root.get()
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.root.get().is_none()
    }

    pub fn from_iter<I>(arena: &'arena Arena, source: I) -> List<'arena, T> where
        I: IntoIterator<Item = T>
    {
        let mut iter = source.into_iter();

        let mut builder = match iter.next() {
            Some(item) => ListBuilder::new(arena, arena.alloc(item)),
            None       => return List::empty(),
        };

        for item in iter {
            builder.push(arena.alloc(item));
        }

        builder.into_list()
    }
}


impl<'arena, T: 'arena + Copy> IntoIterator for List<'arena, T> {
    type Item = &'arena T;
    type IntoIter = ListIter<'arena, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, 'arena, T: 'arena + Copy> IntoIterator for &'a List<'arena, T> {
    type Item = &'arena T;
    type IntoIter = ListIter<'arena, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub struct ListIter<'arena, T: 'arena + Copy> {
    next: Option<&'arena ListItem<'arena, T>>
}

impl<'arena, T: 'arena + Copy> Iterator for ListIter<'arena, T> {
    type Item = &'arena T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let next = self.next;

        next.map(|list_item| {
            let value = list_item.value.get();
            self.next = list_item.next.get();
            value
        })
    }
}
