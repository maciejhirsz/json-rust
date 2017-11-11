use std::mem::size_of;
use std::cell::Cell;

const ARENA_BLOCK: usize = 64 * 1024;

pub struct Arena {
    store: Cell<Vec<Vec<u8>>>,
    ptr: Cell<*mut u8>,
    offset: Cell<usize>
}

impl Arena {
    pub fn new() -> Self {
        let mut store = vec![Vec::with_capacity(ARENA_BLOCK)];
        let ptr = store[0].as_mut_ptr();

        Arena {
            store: Cell::new(store),
            ptr: Cell::new(ptr),
            offset: Cell::new(0)
        }
    }

    #[inline]
    pub fn alloc<'a, T: Sized + Copy>(&'a self, val: T) -> &'a T {
        let mut offset = self.offset.get();
        let cap = offset + size_of::<T>();

        if cap > ARENA_BLOCK {
            self.grow();

            offset = 0;
            self.offset.set(size_of::<T>());
        } else {
            self.offset.set(cap);
        }

        unsafe {
            let ptr = self.ptr.get().offset(offset as isize) as *mut T;
            *ptr = val;
            &*ptr
        }
    }

    pub fn alloc_str<'a>(&'a self, val: &str) -> &'a str {
        let offset = self.offset.get();
        let alignment = size_of::<usize>() - (val.len() % size_of::<usize>());
        let cap = offset + val.len() + alignment;

        if cap > ARENA_BLOCK {
            return self.alloc_string(val.into());
        }

        self.offset.set(cap);

        unsafe {
            use std::ptr::copy_nonoverlapping;
            use std::str::from_utf8_unchecked;
            use std::slice::from_raw_parts;

            let ptr = self.ptr.get().offset(offset as isize);
            copy_nonoverlapping(val.as_ptr(), ptr, val.len());

            from_utf8_unchecked(from_raw_parts(ptr, val.len()))
        }
    }

    pub fn alloc_string<'a>(&'a self, val: String) -> &'a str {
        let ptr = val.as_ptr();
        let len = val.len();

        let mut temp = self.store.replace(Vec::new());
        temp.push(val.into_bytes());
        self.store.replace(temp);

        unsafe {
            use std::str::from_utf8_unchecked;
            use std::slice::from_raw_parts;

            from_utf8_unchecked(from_raw_parts(ptr, len))
        }
    }

    fn grow(&self) {
        let mut temp = self.store.replace(Vec::new());
        let mut block = Vec::with_capacity(ARENA_BLOCK);
        self.ptr.set(block.as_mut_ptr());
        temp.push(block);
        self.store.replace(temp);
    }
}
