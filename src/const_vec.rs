#![allow(dead_code)]
use std::{fmt::Debug, hash::Hash, mem::MaybeUninit};

const DEFAULT_MAX_SIZE: usize = 2usize.pow(10);

/// [`ConstVec`] is a version of [`Vec`] that is usable in const contexts. It has
/// a fixed maximum size, but it can grow and shrink within that size limit
/// as needed.
///
/// # Example
/// ```rust
/// # use const_serialize::ConstVec;
/// const VEC: ConstVec<u8> = {
///     let mut vec = ConstVec::new();
///     vec.push(1);
///     vec.push(2);
///     vec.push(3);
///     vec.push(4);
///     assert!(vec.pop().unwrap() == 4);
///     vec
/// };
/// assert_eq!(VEC.as_ref(), &[1, 2, 3]);
/// ```
pub struct ConstVec<T, const MAX_SIZE: usize = DEFAULT_MAX_SIZE> {
    memory: [MaybeUninit<T>; MAX_SIZE],
    len: u32,
}

impl<T: Clone, const MAX_SIZE: usize> Clone for ConstVec<T, MAX_SIZE> {
    fn clone(&self) -> Self {
        let mut cloned = Self::new_with_max_size();
        for i in 0..self.len as usize {
            cloned.push(self.get(i).unwrap().clone());
        }
        cloned
    }
}

impl<T: Copy, const MAX_SIZE: usize> Copy for ConstVec<T, MAX_SIZE> {}

impl<T: PartialEq, const MAX_SIZE: usize> PartialEq for ConstVec<T, MAX_SIZE> {
    fn eq(&self, other: &Self) -> bool {
        self.as_ref() == other.as_ref()
    }
}

impl<T: Hash, const MAX_SIZE: usize> Hash for ConstVec<T, MAX_SIZE> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.as_ref().hash(state)
    }
}

impl<T, const MAX_SIZE: usize> Default for ConstVec<T, MAX_SIZE> {
    fn default() -> Self {
        Self::new_with_max_size()
    }
}

impl<T: Debug, const MAX_SIZE: usize> Debug for ConstVec<T, MAX_SIZE> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConstVec")
            .field("len", &self.len)
            .field("memory", &self.as_ref())
            .finish()
    }
}

impl<T> ConstVec<T> {
    /// Create a new empty [`ConstVec`]
    pub const fn new() -> Self {
        Self::new_with_max_size()
    }
}

impl<T, const MAX_SIZE: usize> ConstVec<T, MAX_SIZE> {
    /// Create a new empty [`ConstVec`] with a custom maximum size
    ///
    /// # Example
    /// ```rust
    /// # use const_serialize::ConstVec;
    /// const EMPTY: ConstVec<u8, 10> = ConstVec::new_with_max_size();
    /// ```
    pub const fn new_with_max_size() -> Self {
        Self {
            memory: [const { MaybeUninit::uninit() }; MAX_SIZE],
            len: 0,
        }
    }

    /// Push a value onto the end of the [`ConstVec`]
    ///
    /// # Example
    /// ```rust
    /// # use const_serialize::ConstVec;
    /// const VEC: ConstVec<u8> = {
    ///     let mut vec = ConstVec::new();
    ///     vec.push(1);
    ///     vec
    /// };
    /// assert_eq!(VEC.as_ref(), &[1]);
    /// ```
    pub const fn push(&mut self, value: T) {
        self.memory[self.len as usize] = MaybeUninit::new(value);
        self.len += 1;
    }

    /// Extend the [`ConstVec`] with the contents of a slice
    ///
    /// # Example
    /// ```rust
    /// # use const_serialize::ConstVec;
    /// const VEC: ConstVec<u8> = {
    ///     let mut vec = ConstVec::new();
    ///     vec.extend(&[1, 2, 3]);
    ///     vec
    /// };
    /// assert_eq!(VEC.as_ref(), &[1, 2, 3]);
    /// ```
    pub const fn extend(&mut self, other: &[T])
    where
        T: Copy,
    {
        let mut i = 0;
        while i < other.len() {
            self.push(other[i]);
            i += 1;
        }
    }

    /// Get a reference to the value at the given index
    ///
    /// # Example
    /// ```rust
    /// # use const_serialize::ConstVec;
    /// const VEC: ConstVec<u8> = {
    ///     let mut vec = ConstVec::new();
    ///     vec.push(1);
    ///     vec
    /// };
    /// assert_eq!(VEC.get(0), Some(&1));
    /// ```
    pub const fn get(&self, index: usize) -> Option<&T> {
        if index < self.len as usize {
            Some(unsafe { &*self.memory[index].as_ptr() })
        } else {
            None
        }
    }

    /// Get a copy of the value at the given index.
    pub const fn get_copy(&self, index: usize) -> Option<T>
    where
        T: Copy,
    {
        if index < self.len as usize {
            Some(unsafe { self.memory[index].assume_init() })
        } else {
            None
        }
    }

    /// Get a copy of the value at the given index, panicking if out of bounds.
    pub const fn at(&self, index: usize) -> T
    where
        T: Copy,
    {
        match self.get_copy(index) {
            Some(value) => value,
            None => panic!("Out of bounds"),
        }
    }

    /// Get the length of the [`ConstVec`]
    ///
    /// # Example
    /// ```rust
    /// # use const_serialize::ConstVec;
    /// const VEC: ConstVec<u8> = {
    ///     let mut vec = ConstVec::new();
    ///     vec.push(1);
    ///     vec
    /// };
    /// assert_eq!(VEC.len(), 1);
    /// ```
    pub const fn len(&self) -> usize {
        self.len as usize
    }

    /// Check if the [`ConstVec`] is empty
    ///
    /// # Example
    /// ```rust
    /// # use const_serialize::ConstVec;
    /// const EMPTY: ConstVec<u8> = ConstVec::new();
    /// assert!(EMPTY.is_empty());
    /// const ONE: ConstVec<u8> = {
    ///     let mut vec = ConstVec::new();
    ///     vec.push(1);
    ///     vec
    /// };
    /// assert!(!ONE.is_empty());
    /// ```
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Get a reference to the underlying slice
    ///
    /// # Example
    /// ```rust
    /// # use const_serialize::ConstVec;
    /// const VEC: ConstVec<u8> = {
    ///     let mut vec = ConstVec::new();
    ///     vec.push(1);
    ///     vec
    /// };
    /// assert_eq!(VEC.as_ref(), &[1]);
    /// ```
    pub const fn as_ref(&self) -> &[T] {
        unsafe {
            &*(self.memory.split_at(self.len as usize).0 as *const [MaybeUninit<T>] as *const [T])
        }
    }

    /// Swap the values at the given indices
    ///
    /// # Example
    /// ```rust
    /// # use const_serialize::ConstVec;
    /// const VEC: ConstVec<u8> = {
    ///     let mut vec = ConstVec::new();
    ///     vec.push(1);
    ///     vec.push(2);
    ///     vec.swap(0, 1);
    ///     vec
    /// };
    /// assert_eq!(VEC.as_ref(), &[2, 1]);
    /// ```
    pub const fn swap(&mut self, first: usize, second: usize)
    where
        T: Copy,
    {
        assert!(first < self.len as usize);
        assert!(second < self.len as usize);
        let temp = self.memory[first];
        self.memory[first] = self.memory[second];
        self.memory[second] = temp;
    }

    /// Pop a value off the end of the [`ConstVec`]
    ///
    /// # Example
    /// ```rust
    /// # use const_serialize::ConstVec;
    /// const POPPED: (ConstVec<u8>, Option<u8>) = {
    ///     let mut vec = ConstVec::new();
    ///     vec.push(1);
    ///     vec.push(2);
    ///     vec.push(3);
    ///     let popped = vec.pop();
    ///     (vec, popped)
    /// };
    /// assert_eq!(POPPED.0.as_ref(), &[1, 2]);
    /// assert_eq!(POPPED.1.unwrap(), 3);
    /// ```
    pub const fn pop(&mut self) -> Option<T>
    where
        T: Copy,
    {
        if self.len > 0 {
            self.len -= 1;
            let last = self.len as usize;
            let last_value = unsafe { self.memory[last].assume_init() };
            Some(last_value)
        } else {
            None
        }
    }

    /// Remove the value at the given index
    ///
    /// # Example
    /// ```rust
    /// # use const_serialize::ConstVec;
    /// const REMOVED: (ConstVec<u8>, Option<u8>) = {
    ///     let mut vec = ConstVec::new();
    ///     vec.push(1);
    ///     vec.push(2);
    ///     vec.push(3);
    ///     let removed = vec.remove(1);
    ///     (vec, removed)
    /// };
    /// assert_eq!(REMOVED.0.as_ref(), &[1, 3]);
    /// assert_eq!(REMOVED.1.unwrap(), 2);
    /// ```
    pub const fn remove(&mut self, index: usize) -> Option<T>
    where
        T: Copy,
    {
        if index < self.len as usize {
            let value = unsafe { self.memory[index].assume_init() };
            let mut swap_index = index;
            while swap_index + 1 < self.len as usize {
                self.memory[swap_index] = self.memory[swap_index + 1];
                swap_index += 1;
            }
            self.len -= 1;
            Some(value)
        } else {
            None
        }
    }

    /// Set the value at the given index
    ///
    /// # Example
    /// ```rust
    /// # use const_serialize::ConstVec;
    /// const VEC: ConstVec<u8> = {
    ///     let mut vec = ConstVec::new();
    ///     vec.push(1);
    ///     vec.set(0, 2);
    ///     vec
    /// };
    /// assert_eq!(VEC.as_ref(), &[2]);
    /// ```
    pub const fn set(&mut self, index: usize, value: T) {
        if index >= self.len as usize {
            panic!("Out of bounds")
        }
        self.memory[index] = MaybeUninit::new(value);
    }

    pub(crate) const fn into_parts(self) -> ([MaybeUninit<T>; MAX_SIZE], usize) {
        (self.memory, self.len as usize)
    }

    /// Split the [`ConstVec`] into two at the given index
    ///
    /// # Example
    /// ```rust
    /// # use const_serialize::ConstVec;
    /// const SPLIT: (ConstVec<u8>, ConstVec<u8>) = {
    ///     let mut vec = ConstVec::new();
    ///     vec.push(1);
    ///     vec.push(2);
    ///     vec.push(3);
    ///     vec.split_at(1)
    /// };
    /// assert_eq!(SPLIT.0.as_ref(), &[1]);
    /// assert_eq!(SPLIT.1.as_ref(), &[2, 3]);
    /// ```
    pub const fn split_at(&self, index: usize) -> (Self, Self)
    where
        T: Copy,
    {
        assert!(index <= self.len as usize);
        let slice = self.as_ref();
        let (left, right) = slice.split_at(index);
        let mut left_vec = Self::new_with_max_size();
        let mut i = 0;
        while i < left.len() {
            left_vec.push(left[i]);
            i += 1;
        }
        let mut right_vec = Self::new_with_max_size();
        i = 0;
        while i < right.len() {
            right_vec.push(right[i]);
            i += 1;
        }
        (left_vec, right_vec)
    }
}

#[test]
fn test_const_vec() {
    const VEC: ConstVec<u32> = {
        let mut vec = ConstVec::new();
        vec.push(1234);
        vec.push(5678);
        vec
    };
    assert_eq!(VEC.as_ref(), &[1234, 5678]);
    let mut vec = VEC;
    assert_eq!(vec.pop(), Some(5678));
    assert_eq!(vec.pop(), Some(1234));
    assert_eq!(vec.pop(), None);
    assert_eq!(vec.as_ref(), &[]);
}

#[test]
fn test_const_vec_len() {
    const VEC: ConstVec<u32> = {
        let mut vec = ConstVec::new();
        vec.push(1234);
        vec.push(5678);
        vec
    };
    assert_eq!(VEC.len(), 2);
}

#[test]
fn test_const_vec_get() {
    const VEC: ConstVec<u32> = {
        let mut vec = ConstVec::new();
        vec.push(1234);
        vec.push(5678);
        vec
    };
    assert_eq!(VEC.get(0), Some(&1234));
    assert_eq!(VEC.get(1), Some(&5678));
    assert_eq!(VEC.get(2), None);
}

#[test]
fn test_const_vec_swap() {
    const VEC: ConstVec<u32> = {
        let mut vec = ConstVec::new();
        vec.push(1234);
        vec.push(5678);
        vec
    };
    let mut vec = VEC;
    assert_eq!(vec.as_ref(), &[1234, 5678]);
    vec.swap(0, 1);
    assert_eq!(vec.as_ref(), &[5678, 1234]);
    vec.swap(0, 1);
    assert_eq!(vec.as_ref(), &[1234, 5678]);
}

#[test]
fn test_const_vec_remove() {
    const VEC: ConstVec<u32> = {
        let mut vec = ConstVec::new();
        vec.push(1234);
        vec.push(5678);
        vec
    };
    let mut vec = VEC;
    println!("{:?}", vec);
    assert_eq!(vec.as_ref(), &[1234, 5678]);
    assert_eq!(vec.remove(0), Some(1234));
    assert_eq!(vec.as_ref(), &[5678]);
    assert_eq!(vec.remove(0), Some(5678));
    assert_eq!(vec.as_ref(), &[]);
}

#[test]
fn test_const_vec_extend() {
    const VEC: ConstVec<u32> = {
        let mut vec = ConstVec::new();
        vec.push(1234);
        vec.push(5678);
        vec.extend(&[91011, 1213]);
        vec
    };
    let vec = VEC;
    println!("{:?}", vec);
    assert_eq!(vec.as_ref(), &[1234, 5678, 91011, 1213]);
}
