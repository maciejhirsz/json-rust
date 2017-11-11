/// This should be identical to the `Cell` implementation in the standard
/// library, but always require that the internal type implements `Copy`
/// and implements Copy itself.
#[derive(Debug, PartialEq)]
pub struct CopyCell<T: Copy> {
    value: T
}

impl<T: Copy> CopyCell<T> {
    #[inline]
    pub fn new(value: T) -> Self {
        CopyCell {
            value
        }
    }

    #[inline]
    fn mut_ptr(&self) -> *mut T {
        &self.value as *const T as *mut T
    }

    #[inline]
    pub fn get(&self) -> T {
        unsafe {
            *self.mut_ptr()
        }
    }

    #[inline]
    pub fn set(&self, value: T) {
        let ptr = unsafe { &mut *self.mut_ptr() };
        *ptr = value;
    }
}

impl<T: Copy> Clone for CopyCell<T> {
    #[inline]
    fn clone(&self) -> CopyCell<T> {
        CopyCell::new(self.get())
    }
}

impl<T: Copy> Copy for CopyCell<T> { }
