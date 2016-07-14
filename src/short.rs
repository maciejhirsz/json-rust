use std::{ ptr, str, slice, fmt };
use std::ops::Deref;

pub const MAX_LEN: usize = 23;

#[derive(Clone, Copy, PartialEq)]
pub struct Short {
    value: [u8; 23],
    len: u8,
}

impl Short {
    #[inline]
    pub unsafe fn from_slice(slice: &str) -> Self {
        let mut short = Short {
            // initializing memory with 0s makes things faster in the long run
            value: [0; MAX_LEN],
            len: slice.len() as u8,
        };

        ptr::copy_nonoverlapping(slice.as_ptr(), short.value.as_mut_ptr(), slice.len());

        short
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        unsafe {
            str::from_utf8_unchecked(
                slice::from_raw_parts(self.value.as_ptr(), self.len as usize)
            )
        }
    }
}

impl fmt::Debug for Short {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "s"));
        fmt::Debug::fmt(self.as_str(), f)
    }
}

impl fmt::Display for Short {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self.as_str(), f)
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

impl PartialEq<str> for Short {
    fn eq(&self, other: &str) -> bool {
        self.as_str().eq(other)
    }
}

impl PartialEq<Short> for str {
    fn eq(&self, other: &Short) -> bool {
        other.as_str().eq(self)
    }
}

impl PartialEq<String> for Short {
    fn eq(&self, other: &String) -> bool {
        self.as_str().eq(other)
    }
}

impl PartialEq<Short> for String {
    fn eq(&self, other: &Short) -> bool {
        other.as_str().eq(self)
    }
}
