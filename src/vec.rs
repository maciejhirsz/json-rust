use std::vec::{Vec as SVec, IntoIter};
use std::fmt;
use core::iter::FromIterator;
use core::mem::{forget, replace, ManuallyDrop};
use core::ptr::{NonNull, slice_from_raw_parts_mut, slice_from_raw_parts};

pub struct Vec<T> {
    ptr: NonNull<T>,
    len: u32,
    cap: u32,
}

impl<T> Vec<T> {
    pub fn new() -> Self {
        Self::from_svec_unchecked(SVec::new())
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self::from_svec_unchecked(SVec::with_capacity(capacity))
    }

    pub fn push(&mut self, val: T) {
        if self.len == self.cap {
            let new_cap = match self.cap {
                0 => 1,
                n => n * 2,
            };
            // Create a new bigger buffer
            let mut svec = ManuallyDrop::new(SVec::with_capacity(new_cap as usize));

            unsafe {
                let old = self.ptr.as_ptr();

                // Copy contents
                std::ptr::copy_nonoverlapping(old as *const T, svec.as_mut_ptr(), self.len as usize);

                // Drop old buffer, len 0 (we don't want to drop content)
                std::mem::drop(SVec::from_raw_parts(old, 0, self.cap as usize));
            }

            self.ptr = unsafe { NonNull::new_unchecked(svec.as_mut_ptr()) };
            self.cap = new_cap;
        }
        unsafe { self.ptr.as_ptr().add(self.len as usize).write(val) }
        self.len += 1;
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }

        self.len -= 1;
        Some(unsafe {
            self.ptr.as_ptr().add(self.len as usize).read()
        })
    }

    pub fn clear(&mut self) {
        self.with(move |v| v.clear())
    }

    pub fn len(&self) -> usize {
        self.len as usize
    }

    pub fn capacity(&self) -> usize {
        self.cap as usize
    }

    pub fn remove(&mut self, index: usize) -> T {
        self.with(move |v| v.remove(index))
    }

    pub fn as_ptr(&self) -> *const T {
        self.ptr.as_ptr()
    }

    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.ptr.as_ptr()
    }

    fn with<'a, R: 'a, F: FnOnce(&mut SVec<T>) -> R>(&mut self, f: F) -> R {
        let mut svec = ManuallyDrop::new(unsafe {
            SVec::from_raw_parts(
                self.ptr.as_ptr(),
                self.len as usize,
                self.cap as usize,
            )
        });
        self.len = 0;
        self.cap = 0;

        let r = f(&mut svec);

        self.ptr = unsafe { NonNull::new_unchecked(svec.as_mut_ptr()) };
        self.len = svec.len() as u32;
        self.cap = svec.capacity() as u32;

        r
    }

    fn into_inner(self) -> SVec<T> {
        let Vec { ptr, len, cap } = self;

        ManuallyDrop::new(self);

        unsafe {
            SVec::from_raw_parts(
                ptr.as_ptr(),
                len as usize,
                cap as usize,
            )
        }
    }

    fn from_svec_unchecked(svec: SVec<T>) -> Self {
        let mut svec = ManuallyDrop::new(svec);

        let (ptr, len, cap) = (svec.as_mut_ptr(), svec.len(), svec.capacity());

        Vec {
            ptr: unsafe { NonNull::new_unchecked(ptr) },
            len: len as u32,
            cap: cap as u32,
        }
    }

    fn inner_ref<'a>(&'a self) -> &'a [T] {
        unsafe {
            &*slice_from_raw_parts(self.as_ptr(), self.len())
        }
    }

    fn inner_mut<'a>(&'a mut self) -> &'a mut [T] {
        unsafe {
            &mut*slice_from_raw_parts_mut(self.as_mut_ptr(), self.len())
        }
    }
}

impl<T> std::ops::Drop for Vec<T> {
    fn drop(&mut self) {
        unsafe {
            SVec::from_raw_parts(
                self.ptr.as_ptr(),
                self.len as usize,
                self.cap as usize,
            );
        }
    }
}

impl<T: Clone> Clone for Vec<T> {
    fn clone(&self) -> Vec<T> {
        Vec::from_svec_unchecked((&**self).to_vec())
    }
}

impl<T> std::ops::Deref for Vec<T> {
    type Target = [T];

    fn deref(&self) -> &[T] {
        self.inner_ref()
    }
}

impl<T> std::ops::DerefMut for Vec<T> {
    fn deref_mut(&mut self) -> &mut [T] {
        self.inner_mut()
    }
}

impl<T: fmt::Debug> fmt::Debug for Vec<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.inner_ref().fmt(f)
    }
}

impl<T> IntoIterator for Vec<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> IntoIter<T> {
        self.into_inner().into_iter()
    }
}

impl<T> FromIterator<T> for Vec<T> {
    fn from_iter<I>(iter: I) -> Vec<T>
    where
        I: IntoIterator<Item = T>,
    {
        Self::from_svec_unchecked(SVec::from_iter(iter))
    }
}

impl<T: PartialEq> PartialEq for Vec<T> {
    fn eq(&self, other: &Vec<T>) -> bool {
        self.inner_ref() == other.inner_ref()
    }
}

unsafe impl<T: Sync> Sync for Vec<T> {}
unsafe impl<T: Send> Send for Vec<T> {}


const MASK_LO: usize = core::u32::MAX as usize;
const MASK_HI: usize = !(core::u32::MAX as usize);


#[inline]
unsafe fn pack<T>(ptr: *mut T, len: usize, capacity: usize) -> *mut [T] {
    if (capacity & MASK_HI) != 0 {
        panic!("beef::Cow::owned: Capacity out of bounds");
    }

    pack_unchecked(ptr, len, capacity)
}


#[inline]
unsafe fn pack_unchecked<T>(ptr: *mut T, len: usize, capacity: usize) -> *mut [T] {
    slice_from_raw_parts_mut(
        ptr as *mut T,
        (len & MASK_LO) | ((capacity & MASK_HI) << std::mem::size_of::<u32>() * 8)
    )
}

#[inline]
// fn unpack<T>(ptr: NonNull<[T]>) -> (*mut T, usize, usize) {
fn unpack<T>(ptr: *mut [T]) -> (*mut T, usize, usize) {
    // let caplen = unsafe { ptr.as_ref().len() };
    let caplen = unsafe { (&*ptr).len() };

    (
        ptr as *mut T,
        caplen & MASK_LO,
        (caplen & MASK_HI) >> std::mem::size_of::<u32>() * 8,
    )
}
