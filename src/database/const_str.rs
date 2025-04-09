const BUFFER_SIZE: usize = 2 << 8;

/// A const-friendly string type used to build static string slices.
///
/// This type is used to build database query strings at compile time for small performance gains.
pub struct ConstStr {
    data: [u8; BUFFER_SIZE],
    len: usize,
}

impl ConstStr {
    /// Creates a new [`ConstStr`] with an empty buffer.
    pub const fn new() -> ConstStr {
        ConstStr {
            data: [0u8; BUFFER_SIZE],
            len: 0,
        }
    }

    /// Copies the contents of the given string slice and appends it to the internal buffer.
    pub const fn append(mut self, s: &str) -> Self {
        let b = s.as_bytes();
        let mut index = 0;
        while index < b.len() {
            self.data[self.len] = b[index];
            self.len += 1;
            index += 1;
        }

        self
    }

    /// Returns the string slice represented by the internal buffer.
    pub const fn as_str(&self) -> &str {
        let mut data: &[u8] = &self.data;
        let mut n = data.len() - self.len;
        while n > 0 {
            n -= 1;
            match data.split_last() {
                Some((_, rest)) => data = rest,
                None => panic!(),
            }
        }

        // SAFETY: `data` is valid UTF-8 because it is built from a `&str`.
        unsafe { std::str::from_utf8_unchecked(data) }
    }
}
