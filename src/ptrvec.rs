//! [`PtrVec`] is used to return data from game implementations.
//!
//! The overhead is minimized by writing data directly into the caller-provided
//! buffer.

use std::{
    fmt::Write,
    num::NonZeroU8,
    ops::{Index, IndexMut},
    os::raw::c_char,
};

/// Vector implementation over a memory buffer with a fixed, run-time capacity.
///
/// [`PtrVec`] allows to perform vector operations on memory not allocated by
/// a [`Vec`].
/// This is especially useful for buffers provided via FFI.
pub struct PtrVec<T> {
    buf: *mut T,
    len: usize,
    capacity: usize,
}

impl<T> PtrVec<T> {
    #[inline]
    pub(crate) unsafe fn new(buf: *mut T, capacity: usize) -> Self {
        Self {
            buf,
            len: 0,
            capacity,
        }
    }

    /// Length of the vector.
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns whether [`Self::len()`] is zero.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Length of the underlying buffer and therefore the maximum for
    /// [`Self::len()`].
    #[inline]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Returns whether [`Self::len()`]` == `[`Self::capacity()`].
    #[inline]
    pub fn is_full(&self) -> bool {
        self.len >= self.capacity
    }

    /// Append to the buffer at index [`Self::len()`].
    ///
    /// # Panics
    /// Panics if the vector is full.
    #[inline]
    pub fn push(&mut self, value: T) {
        assert!(self.len < self.capacity, "cannot push into full PtrVec");

        unsafe {
            self.buf.add(self.len).write(value);
        }
        self.len += 1;
    }

    #[inline]
    fn assert_index(&self, index: usize) {
        assert!(index < self.len, "cannot access element outside PtrVec");
    }
}

impl<T: Clone> PtrVec<T> {
    /// Resizes the vector to `new_len`.
    ///
    /// If `new_len` is smaller than the current length, the vector is extended
    /// with clones of `value`.
    ///
    /// # Panics
    /// Panics if `new_len` is larger than the capacity.
    pub fn resize(&mut self, new_len: usize, value: T) {
        assert!(
            new_len <= self.capacity,
            "cannot resize PtrVec over capacity"
        );

        if new_len <= self.len {
            for _ in new_len..self.len {
                self.len -= 1;
                unsafe {
                    self.buf.add(self.len).drop_in_place();
                }
            }
        } else {
            for _ in self.len..new_len {
                self.push(value.clone());
            }
        }
    }

    /// Appends all elements of `other` to the vector.
    ///
    /// # Panics
    /// Panics if `other` is larger than the number of free slots.
    pub fn extend_from_slice(&mut self, other: &[T]) {
        assert!(
            other.len() <= self.capacity - self.len,
            "not enough free space in PtrVec"
        );

        for value in other.iter() {
            unsafe {
                self.buf.add(self.len).write(value.clone());
            }
            self.len += 1;
        }
    }
}

impl<T> Index<usize> for PtrVec<T> {
    type Output = T;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        self.assert_index(index);

        unsafe { &*self.buf.add(index) }
    }
}

impl<T> IndexMut<usize> for PtrVec<T> {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.assert_index(index);

        unsafe { &mut *self.buf.add(index) }
    }
}

impl PtrVec<NonZeroU8> {
    /// Size must be at least 1.
    #[inline]
    pub(crate) unsafe fn from_c_char(buf: *mut c_char, size: usize) -> Self {
        Self {
            buf: buf.cast(),
            len: 0,
            capacity: size
                .checked_sub(1)
                .expect("C string buffer must not be of size zero"),
        }
    }

    #[inline]
    pub(crate) unsafe fn nul_terminate(&mut self) {
        self.buf.add(self.len).cast::<c_char>().write(0);
    }
}

impl Write for PtrVec<NonZeroU8> {
    /// Appends the bytes of `s` to the vector.
    ///
    /// Returns an error when inserting NUL bytes or when the vector is full.
    ///
    /// # Example
    /// ```no_run
    /// # use surena_game::*;
    /// # use std::{ptr::null_mut, fmt::Write};
    /// # #[allow(deref_nullptr)]
    /// # let ptr_vec = unsafe { &mut *null_mut::<StrBuf>() };
    /// write!(ptr_vec, "example string").expect("failed to write PtrVec");
    /// ```
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        for b in s.bytes() {
            let b = NonZeroU8::new(b).ok_or_else(Default::default)?;
            if self.is_full() {
                return Err(Default::default());
            }
            self.push(b);
        }
        Ok(())
    }
}
