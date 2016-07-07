use std::collections::btree_map;
use std::slice;
use std::iter::{Iterator, DoubleEndedIterator};
use JsonValue;

pub enum Members<'a> {
    Some(slice::Iter<'a, JsonValue>),
    None
}

pub enum MembersMut<'a> {
    Some(slice::IterMut<'a, JsonValue>),
    None
}

pub enum Entries<'a> {
    Some(btree_map::Iter<'a, String, JsonValue>),
    None
}

pub enum EntriesMut<'a> {
    Some(btree_map::IterMut<'a, String, JsonValue>),
    None
}

impl<'a> Iterator for Members<'a> {
    type Item = &'a JsonValue;

    fn next(&mut self) -> Option<&'a JsonValue> {
        match *self {
            Members::Some(ref mut iter) => iter.next(),
            Members::None               => None,
        }
    }
}

impl<'a> DoubleEndedIterator for Members<'a> {
    fn next_back(&mut self) -> Option<&'a JsonValue> {
        match *self {
            Members::Some(ref mut iter) => iter.next_back(),
            Members::None               => None,
        }
    }
}

impl<'a> Iterator for MembersMut<'a> {
    type Item = &'a mut JsonValue;

    fn next(&mut self) -> Option<&'a mut JsonValue> {
        match *self {
            MembersMut::Some(ref mut iter) => iter.next(),
            MembersMut::None               => None,
        }
    }
}

impl<'a> DoubleEndedIterator for MembersMut<'a> {
    fn next_back(&mut self) -> Option<&'a mut JsonValue> {
        match *self {
            MembersMut::Some(ref mut iter) => iter.next_back(),
            MembersMut::None               => None,
        }
    }
}

impl<'a> Iterator for Entries<'a> {
    type Item = (&'a String, &'a JsonValue);

    fn next(&mut self) -> Option<(&'a String, &'a JsonValue)> {
        match *self {
            Entries::Some(ref mut iter) => iter.next(),
            Entries::None               => None
        }
    }
}

impl<'a> DoubleEndedIterator for Entries<'a> {
    fn next_back(&mut self) -> Option<(&'a String, &'a JsonValue)> {
        match *self {
            Entries::Some(ref mut iter) => iter.next_back(),
            Entries::None               => None
        }
    }
}

impl<'a> Iterator for EntriesMut<'a> {
    type Item = (&'a String, &'a mut JsonValue);

    fn next(&mut self) -> Option<(&'a String, &'a mut JsonValue)> {
        match *self {
            EntriesMut::Some(ref mut iter) => iter.next(),
            EntriesMut::None               => None
        }
    }
}

impl<'a> DoubleEndedIterator for EntriesMut<'a> {
    fn next_back(&mut self) -> Option<(&'a String, &'a mut JsonValue)> {
        match *self {
            EntriesMut::Some(ref mut iter) => iter.next_back(),
            EntriesMut::None               => None
        }
    }
}
