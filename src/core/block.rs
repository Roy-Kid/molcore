//! Block: dict-like keyed arrays with consistent axis-0 length
//!
//! A Block stores heterogeneous arrays (any type implementing [`Array`]) keyed by
//! strings, enforcing that all stored arrays share the same axis-0 length
//! (nrows).
//!
//! - `len()` is the number of keys
//! - `nrows()` is the common axis-0 length of all arrays (or `None` if empty)
//! - `insert`, `get`, `remove`, `contains_key`, `clear`, and basic iterators

use std::collections::HashMap;

use super::array::Array;

/// Errors that can occur when manipulating a [`Block`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockError {
    /// Inserted array has rank 0 (no axis-0 length can be defined)
    RankZero {
        /// The key for which the error occurred
        key: String,
    },
    /// Inserted array's axis-0 length does not match the Block's `nrows`
    RaggedAxis0 {
        /// The key for which the mismatch was detected
        key: String,
        /// The axis-0 length the block expects
        expected: usize,
        /// The axis-0 length provided by the inserted array
        got: usize,
    },
}

impl core::fmt::Display for BlockError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            BlockError::RankZero { key } => write!(f, "array for key '{}' has rank 0; expected at least 1D", key),
            BlockError::RaggedAxis0 { key, expected, got } => write!(
                f,
                "array for key '{}' has axis-0 length {} but block expects {}",
                key, got, expected
            ),
        }
    }
}

/// A dictionary from string keys to heterogeneous arrays with a consistent axis-0 length.
#[derive(Default)]
pub struct Block {
    map: HashMap<String, Box<dyn Array>>, // heterogeneous arrays via trait objects
    nrows: Option<usize>,                  // enforced leading dimension, if any
}

impl core::fmt::Debug for Block {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut map = f.debug_map();
        for (k, v) in &self.map {
            map.entry(k, &v.shape());
        }
        map.finish()
    }
}

impl Block {
    /// Creates an empty Block.
    pub fn new() -> Self { Self { map: HashMap::new(), nrows: None } }

    /// Creates an empty Block with the specified capacity.
    pub fn with_capacity(cap: usize) -> Self { Self { map: HashMap::with_capacity(cap), nrows: None } }

    /// Number of keys (columns).
    #[inline]
    pub fn len(&self) -> usize { self.map.len() }

    /// Returns true if there are no arrays in the block.
    #[inline]
    pub fn is_empty(&self) -> bool { self.map.is_empty() }

    /// Returns the common axis-0 length of all arrays, or `None` if empty.
    #[inline]
    pub fn nrows(&self) -> Option<usize> { self.nrows }

    /// Back-compat: alias for `nrows()`.
    #[inline]
    pub fn len0(&self) -> Option<usize> { self.nrows() }

    /// Back-compat: alias for `len()`.
    #[inline]
    pub fn ncols(&self) -> usize { self.len() }

    /// Returns true if the Block contains the specified key.
    #[inline]
    pub fn contains_key(&self, key: &str) -> bool { self.map.contains_key(key) }

    /// Gets an immutable reference to the array for `key` if present.
    #[inline]
    pub fn get(&self, key: &str) -> Option<&dyn Array> { self.map.get(key).map(|b| b.as_ref()) }

    /// Gets a mutable reference to the array for `key` if present.
    #[inline]
    pub fn get_mut(&mut self, key: &str) -> Option<&mut Box<dyn Array>> { self.map.get_mut(key) }

    /// Inserts an array under `key`, enforcing consistent axis-0 length.
    ///
    /// - Returns `Ok(prev)` where `prev` is the previous value for `key`, if any.
    /// - Returns `Err` if the array has rank 0 or violates the Block's `nrows`.
    pub fn insert<A>(&mut self, key: impl Into<String>, arr: A) -> Result<Option<Box<dyn Array>>, BlockError>
    where
        A: Array + 'static,
    {
        let key = key.into();
        let shape = arr.shape();
        if shape.is_empty() { return Err(BlockError::RankZero { key }); }
        let len0 = shape[0];

        match self.nrows {
            None => {
                // First insertion defines nrows
                self.nrows = Some(len0);
                Ok(self.map.insert(key, Box::new(arr)))
            }
            Some(expected) => {
                if len0 != expected {
                    return Err(BlockError::RaggedAxis0 { key, expected, got: len0 });
                }
                Ok(self.map.insert(key, Box::new(arr)))
            }
        }
    }

    /// Removes and returns the array for `key`, if present. If the Block becomes
    /// empty after removal, resets `nrows` to `None`.
    pub fn remove(&mut self, key: &str) -> Option<Box<dyn Array>> {
        let out = self.map.remove(key);
        if self.map.is_empty() { self.nrows = None; }
        out
    }

    /// Clears the Block, removing all keys and resetting `nrows`.
    pub fn clear(&mut self) {
        self.map.clear();
        self.nrows = None;
    }

    /// Returns an iterator over (&str, &dyn Array).
    pub fn iter(&self) -> impl Iterator<Item = (&str, &dyn Array)> {
        self.map.iter().map(|(k, v)| (k.as_str(), v.as_ref() as &dyn Array))
    }

    /// Returns an iterator over keys.
    pub fn keys(&self) -> impl Iterator<Item = &str> { self.map.keys().map(|k| k.as_str()) }

    /// Returns an iterator over array references.
    pub fn values(&self) -> impl Iterator<Item = &dyn Array> { self.map.values().map(|v| v.as_ref()) }
}