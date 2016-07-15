use std::{ ptr, str, slice, fmt };
use std::ops::Deref;

pub const MAX_LEN: usize = 30;

#[derive(Clone, Copy, PartialEq)]
pub struct Short {
    value: [u8; MAX_LEN],
    len: u8,
}

/// A `Short` is a small string, up to `MAX_LEN` bytes, that can be managed without
/// the expensive heap allocation performed for the regular `String` type.
impl Short {
    /// Creates a `Short` from a `&str` slice. This method can cause buffer
    /// overflow if the length of the slice is larger than `MAX_LEN`. Typically
    /// you should never want to create your own `Short`s, instead creating a
    /// `JsonValue` (using `.into()` or `JsonValue::from(slice)`) out of a slice
    /// will automatically decide on `String` or `Short` for you.
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

    /// Cheaply obtain a `&str` slice out of the `Short`.
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
        fmt::Debug::fmt(self.as_str(), f)
    }
}

impl fmt::Display for Short {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self.as_str(), f)
    }
}

/// Implements `Deref` for `Short` means that, just like `String`, you can
/// pass `&Short` to functions that expect `&str` and have the conversion happen
/// automagically. On top of that, all methods present on `&str` can be called on
/// an instance of `Short`.
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
