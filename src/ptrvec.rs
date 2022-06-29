//! [`PtrVec`] is used to return data from game implementations.
//!
//! The overhead is minimized by writing data directly into the caller-provided
//! buffer.

use std::{
    fmt::Write,
    mem::{size_of, transmute, MaybeUninit},
    num::NonZeroU8,
    ops::{Index, IndexMut},
    os::raw::c_char,
    ptr::NonNull,
    slice,
};

/// Vector implementation over a memory buffer with a fixed, run-time capacity.
///
/// [`PtrVec`] allows to perform vector operations on memory not allocated by
/// a [`Vec`].
/// This is especially useful for buffers provided via FFI.
///
/// The lifetime bound makes sure that [`PtrVec`]s with different lifetimes
/// cannot be [`std::mem::swap`]ped.
pub struct PtrVec<'b, T> {
    buf: &'b mut [MaybeUninit<T>],
    len: usize,
}

impl<'b, T> PtrVec<'b, T> {
    /// Create a [`PtrVec`] which uses the memory at `buf` up to length
    /// `capacity` as backing storage.
    ///
    /// # Safety
    /// If `T` is not zero-sized and the `capacity` is not zero, `buf` must
    /// fullfil the requirements of [`slice::from_raw_parts_mut()`].
    #[inline]
    pub(crate) unsafe fn new(mut buf: *mut T, capacity: usize) -> Self {
        if size_of::<T>() == 0 || capacity == 0 {
            buf = NonNull::dangling().as_ptr();
        }

        Self {
            buf: slice::from_raw_parts_mut(buf.cast::<MaybeUninit<T>>(), capacity),
            len: 0,
        }
    }

    /// Create a new [`PtrVec`] using a [`Vec`] as backing storage.
    ///
    /// It starts with length 0.
    ///
    /// # Example
    /// ```
    /// # use surena_game::PtrVec;
    /// let mut vec = vec![0; 3];
    /// let mut ptr_vec = PtrVec::from_vec(&mut vec);
    /// assert!(ptr_vec.is_empty());
    /// ptr_vec.push(42);
    /// assert_eq!(3, ptr_vec.capacity());
    /// assert_eq!(1, ptr_vec.len());
    /// assert_eq!(42, vec[0]);
    /// ```
    #[inline]
    pub fn from_vec(buf: &'b mut Vec<T>) -> Self {
        Self {
            buf: unsafe { transmute::<&'b mut [T], &'b mut [MaybeUninit<T>]>(buf) },
            len: 0,
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
        self.buf.len()
    }

    /// Returns whether [`Self::len()`]` == `[`Self::capacity()`].
    #[inline]
    pub fn is_full(&self) -> bool {
        self.len >= self.capacity()
    }

    /// Append to the buffer at index [`Self::len()`].
    ///
    /// # Panics
    /// Panics if the vector is full.
    #[inline]
    pub fn push(&mut self, value: T) {
        self.buf
            .get_mut(self.len)
            .expect("cannot push into full PtrVec")
            .write(value);
        self.len += 1;
    }
}

impl<'b, T: Clone> PtrVec<'b, T> {
    /// Resizes the vector to `new_len`.
    ///
    /// If `new_len` is smaller than the current length, the vector is extended
    /// with clones of `value`.
    ///
    /// # Panics
    /// Panics if `new_len` is larger than the capacity.
    pub fn resize(&mut self, new_len: usize, value: T) {
        assert!(
            new_len <= self.capacity(),
            "cannot resize PtrVec over capacity"
        );

        if new_len <= self.len {
            for _ in new_len..self.len {
                self.len -= 1;
                unsafe {
                    self.buf[self.len].assume_init_drop();
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
            other.len() <= self.capacity() - self.len,
            "not enough free space in PtrVec"
        );

        for value in other.iter() {
            self.push(value.clone());
        }
    }
}

const ACCESS_ERROR: &str = "cannot access element outside PtrVec";

impl<'b, T> Index<usize> for PtrVec<'b, T> {
    type Output = T;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        unsafe { self.buf.get(index).expect(ACCESS_ERROR).assume_init_ref() }
    }
}

impl<'b, T> IndexMut<usize> for PtrVec<'b, T> {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        unsafe {
            self.buf
                .get_mut(index)
                .expect(ACCESS_ERROR)
                .assume_init_mut()
        }
    }
}

impl<'b> PtrVec<'b, NonZeroU8> {
    /// Size must be at least 1.
    #[inline]
    pub(crate) unsafe fn from_c_char(buf: *mut c_char, size: usize) -> Self {
        Self::new(
            buf.cast(),
            size.checked_sub(1)
                .expect("C string buffer must not be of size zero"),
        )
    }
}

impl<'b> Write for PtrVec<'b, NonZeroU8> {
    /// Appends the bytes of `s` to the vector.
    ///
    /// Returns an error when inserting NUL bytes or when the vector is full.
    ///
    /// # Example
    /// ```
    /// # use surena_game::*;
    /// # use std::{ptr::null_mut, fmt::Write};
    /// # #[allow(deref_nullptr)]
    /// # let mut vec = vec![1.try_into().unwrap(); 14];
    /// # let mut ptr_vec = StrBuf::from_vec(&mut vec);
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
