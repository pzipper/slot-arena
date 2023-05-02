#![doc = include_str!("../README.md")]

mod r#ref;

use std::fmt::Debug;

pub use r#ref::*;

/// A block of memory accessed using 32-bit [Ref]s rather than 64-bit memory addresses.
#[derive(Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct SlotArena<T> {
    raw: Vec<T>,
    free: Vec<Ref<T>>,
}

impl<T> SlotArena<T> {
    /// Creates an empty [SlotArena].  Does not pre-allocate any memory.
    #[inline]
    pub const fn new() -> Self {
        Self {
            raw: Vec::new(),
            free: Vec::new(),
        }
    }

    /// Creates an empty [SlotArena], pre-allocated for the provided capacity.
    #[inline]
    pub fn with_capacity(capacity: u32) -> Self {
        Self {
            raw: Vec::with_capacity(capacity as usize),
            free: Vec::new(),
        }
    }

    /// Frees the provided value.  A value should not be used once it is freed, as it may be
    /// replaced by another value.
    #[inline]
    pub fn free(&mut self, value: Ref<T>) {
        self.free.push(value);
    }

    /// Inserts a value into the [SlotArena], returning a [Ref] to it.
    ///
    /// # Panics
    /// Panics if the number of items in this [SlotArena] exceeds `u32::MAX`.
    pub fn insert(&mut self, value: T) -> Ref<T> {
        match self.free.pop() {
            Some(idx) => {
                self.raw[idx.to_raw() as usize] = value;
                idx
            }
            None => {
                let idx = Ref::from_raw(self.raw.len() as u32);
                self.raw.push(value);
                idx
            }
        }
    }

    /// Attempts to insert a value into the [SlotArena], returning [`None`] if it is full.
    pub fn try_insert(&mut self, value: T) -> Option<Ref<T>> {
        match self.free.pop() {
            Some(idx) => {
                self.raw[idx.to_raw() as usize] = value;
                Some(idx)
            }
            None => {
                if self.raw.len() == u32::MAX as usize {
                    return None;
                }

                let idx = Ref::from_raw(self.raw.len() as u32);
                self.raw.push(value);
                Some(idx)
            }
        }
    }

    /// Returns `true` if the provided reference is valid (if the reference is in the bounds of the
    /// memory block AND the reference is not free).
    #[inline]
    pub fn is_valid(&self, value: Ref<T>) -> bool {
        !self.free.contains(&value) && (value.to_raw() as usize) < self.raw.len()
    }

    /// Returns a non-opaque reference to the provided value.
    ///
    /// # Panics
    /// Panics if the provided reference is invalid.
    #[inline]
    pub fn get(&self, value: Ref<T>) -> &T {
        debug_assert!(self.is_valid(value));
        &self.raw[value.to_raw() as usize]
    }

    /// Attempts to get the value of the provided reference, returns [`None`] if the reference was
    /// invalid.
    pub fn try_get(&self, value: Ref<T>) -> Option<&T> {
        if self.is_valid(value) {
            Some(&self.raw[value.to_raw() as usize])
        } else {
            None
        }
    }

    /// Returns a non-opaque reference to the provided value.
    ///
    /// # Panics
    /// Panics if the provided reference is invalid.
    #[inline]
    pub fn get_mut(&mut self, value: Ref<T>) -> &mut T {
        debug_assert!(self.is_valid(value));
        &mut self.raw[value.to_raw() as usize]
    }

    /// Attempts to get the value of the provided reference, returns [`None`] if the reference was
    /// invalid.
    pub fn try_get_mut(&mut self, value: Ref<T>) -> Option<&mut T> {
        if self.is_valid(value) {
            Some(&mut self.raw[value.to_raw() as usize])
        } else {
            None
        }
    }

    /// Returns an iterator through the alive items in the [SlotArena].
    pub fn iter(&self) -> impl Iterator<Item = (Ref<T>, &T)> {
        self.raw
            .iter()
            .enumerate()
            .map(|(idx, item)| (Ref::from_raw(idx as u32), item))
            .filter(|(idx, _)| !self.free.contains(&idx))
    }

    /// Returns an iterator through the alive items in the [SlotArena].
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (Ref<T>, &mut T)> {
        self.raw
            .iter_mut()
            .enumerate()
            .map(|(idx, item)| (Ref::from_raw(idx as u32), item))
            .filter(|(idx, _)| !self.free.contains(&idx))
    }
}

impl<T: Debug> Debug for SlotArena<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map().entries(self.iter()).finish()
    }
}
