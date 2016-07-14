use std::{ ptr, mem, str, slice };
use std::ops::Deref;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Short {
    value: [u8; 30],
    len: u8,
}

impl Short {
    #[inline]
    pub unsafe fn from_slice_unchecked(slice: &str) -> Self {
        let mut short: Short = mem::uninitialized();

        ptr::copy_nonoverlapping(slice.as_ptr(), short.value.as_mut_ptr(), slice.len());

        short.len = slice.len() as u8;
        short
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        unsafe {
            str::from_utf8_unchecked(
                slice::from_raw_parts(self.value.as_ptr(), self.len())
            )
        }
    }
}

impl Deref for Short {
    type Target = str;

    fn deref(&self) -> &str {
        self.as_str()
    }
}

impl From<Short> for String {
    fn from(short: Short) -> String {
        String::from(short.as_str())
    }
}
