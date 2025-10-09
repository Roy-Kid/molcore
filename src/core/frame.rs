//! Frame: a dictionary mapping string keys to [`Block`].
//!
//! A Frame groups multiple [`Block`]s under string keys. It behaves like a
//! standard dictionary: `len()` returns the number of blocks, `contains_key`,
//! `get`, `insert`, `remove`, `clear`, and basic iterators are provided.
//!
//! Unlike [`Block`], `Frame` does not enforce any global axis-0 consistency
//! across contained blocks: each `Block` manages its own `nrows` invariant.

use std::collections::HashMap;

use super::block::Block;

/// A dictionary from string keys to [`Block`].
#[derive(Default)]
pub struct Frame {
    map: HashMap<String, Block>,
}

impl core::fmt::Debug for Frame {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut map = f.debug_map();
        for (k, b) in &self.map {
            map.entry(k, &format!("Block(nrows={:?}, ncols={})", b.nrows(), b.len()));
        }
        map.finish()
    }
}

impl Frame {
    /// Creates an empty Frame.
    pub fn new() -> Self { Self { map: HashMap::new() } }

    /// Creates an empty Frame with the specified capacity.
    pub fn with_capacity(cap: usize) -> Self { Self { map: HashMap::with_capacity(cap) } }

    /// Number of blocks (keys) in the frame.
    #[inline]
    pub fn len(&self) -> usize { self.map.len() }

    /// Returns true if the frame contains no blocks.
    #[inline]
    pub fn is_empty(&self) -> bool { self.map.is_empty() }

    /// Returns true if the frame contains the specified key.
    #[inline]
    pub fn contains_key(&self, key: &str) -> bool { self.map.contains_key(key) }

    /// Gets an immutable reference to the block for `key` if present.
    #[inline]
    pub fn get(&self, key: &str) -> Option<&Block> { self.map.get(key) }

    /// Gets a mutable reference to the block for `key` if present.
    #[inline]
    pub fn get_mut(&mut self, key: &str) -> Option<&mut Block> { self.map.get_mut(key) }

    /// Inserts a block under `key`. Returns the previous block if any.
    pub fn insert(&mut self, key: impl Into<String>, block: Block) -> Option<Block> {
        self.map.insert(key.into(), block)
    }

    /// Removes and returns the block for `key`, if present.
    pub fn remove(&mut self, key: &str) -> Option<Block> { self.map.remove(key) }

    /// Clears the frame, removing all keys.
    pub fn clear(&mut self) { self.map.clear(); }

    /// Returns an iterator over (&str, &Block).
    pub fn iter(&self) -> impl Iterator<Item = (&str, &Block)> {
        self.map.iter().map(|(k, v)| (k.as_str(), v))
    }

    /// Returns an iterator over keys.
    pub fn keys(&self) -> impl Iterator<Item = &str> { self.map.keys().map(|k| k.as_str()) }

    /// Returns an iterator over block references.
    pub fn values(&self) -> impl Iterator<Item = &Block> { self.map.values() }
}
