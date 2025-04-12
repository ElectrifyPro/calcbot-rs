use std::ops::{Add, AddAssign, Deref, Sub, SubAssign};

/// A wrapper around [`usize`] that is clamped to a range. When adding or subtracting to this
/// wrapper, the value will wrap around to the other end of the range.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Clamped {
    /// The inner value.
    value: usize,

    /// The maximum value of the wrapper. This is **non-inclusive**.
    max: usize,
}

impl Clamped {
    /// Creates a new [`Clamped`] with the given value and maximum.
    pub fn new(value: usize, max: usize) -> Self {
        Self { value: value % max, max }
    }

    /// Returns the inner value.
    pub fn value(&self) -> usize {
        self.value
    }

    /// Returns the maximum value.
    pub fn max(&self) -> usize {
        self.max
    }
}

impl Deref for Clamped {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl Add<usize> for Clamped {
    type Output = Self;

    fn add(self, rhs: usize) -> Self::Output {
        Self::new((self.value + rhs) % self.max, self.max)
    }
}

impl AddAssign<usize> for Clamped {
    fn add_assign(&mut self, rhs: usize) {
        self.value = (self.value + rhs) % self.max;
    }
}

impl Sub<usize> for Clamped {
    type Output = Self;

    fn sub(self, rhs: usize) -> Self::Output {
        Self::new((self.value + self.max - rhs) % self.max, self.max)
    }
}

impl SubAssign<usize> for Clamped {
    fn sub_assign(&mut self, rhs: usize) {
        self.value = (self.value + self.max - rhs) % self.max;
    }
}
