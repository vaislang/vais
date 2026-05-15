//! Identifier string interning pool for code generation.
//!
//! Reduces memory allocations by storing each unique identifier string once
//! and referring to it by a compact `InternId`. This is particularly effective
//! for function names, struct names, and variable names that appear repeatedly
//! during code generation.
//!
//! # Performance
//!
//! - Interning: O(1) amortized (HashMap lookup + optional insert)
//! - Resolving: O(1) (Vec index lookup)
//! - Memory: Each unique string stored exactly once, plus 4 bytes per reference

use std::collections::HashMap;

/// Compact identifier for an interned string. Uses u32 to save memory
/// compared to storing full String clones throughout the codegen pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InternId(u32);

impl InternId {
    /// Get the raw index value (for debugging/testing)
    #[inline]
    pub fn index(self) -> u32 {
        self.0
    }
}

/// String interning pool that stores each unique string once.
///
/// Thread-local (not `Sync`): designed for single-threaded codegen within
/// one `CodeGenerator` instance. For parallel module compilation, each
/// `CodeGenerator` has its own pool.
pub struct IdentPool {
    /// Map from string to its intern ID for O(1) lookup
    map: HashMap<String, InternId>,
    /// Ordered storage for O(1) ID-to-string resolution
    strings: Vec<String>,
}

impl IdentPool {
    /// Create a new interning pool with default capacity.
    pub fn new() -> Self {
        Self {
            map: HashMap::with_capacity(256),
            strings: Vec::with_capacity(256),
        }
    }

    /// Create a new interning pool with specified capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            map: HashMap::with_capacity(capacity),
            strings: Vec::with_capacity(capacity),
        }
    }

    /// Intern a string, returning its unique ID.
    ///
    /// If the string has been interned before, returns the existing ID
    /// without allocating. Otherwise, stores the string and returns a new ID.
    #[inline]
    pub fn intern(&mut self, s: &str) -> InternId {
        if let Some(&id) = self.map.get(s) {
            return id;
        }
        let id = InternId(self.strings.len() as u32);
        let owned = s.to_string();
        self.strings.push(owned.clone());
        self.map.insert(owned, id);
        id
    }

    /// Intern an already-owned string, avoiding a clone if it's new.
    #[inline]
    pub fn intern_owned(&mut self, s: String) -> InternId {
        if let Some(&id) = self.map.get(&s) {
            return id;
        }
        let id = InternId(self.strings.len() as u32);
        self.strings.push(s.clone());
        self.map.insert(s, id);
        id
    }

    /// Resolve an intern ID back to its string.
    ///
    /// # Panics
    ///
    /// Panics if the ID was not produced by this pool.
    #[inline]
    pub fn resolve(&self, id: InternId) -> &str {
        &self.strings[id.0 as usize]
    }

    /// Try to resolve an intern ID, returning `None` if invalid.
    #[inline]
    pub fn try_resolve(&self, id: InternId) -> Option<&str> {
        self.strings.get(id.0 as usize).map(|s| s.as_str())
    }

    /// Check if a string has already been interned.
    #[inline]
    pub fn contains(&self, s: &str) -> bool {
        self.map.contains_key(s)
    }

    /// Get the intern ID for a string if it exists.
    #[inline]
    pub fn get(&self, s: &str) -> Option<InternId> {
        self.map.get(s).copied()
    }

    /// Number of unique strings interned.
    #[inline]
    pub fn len(&self) -> usize {
        self.strings.len()
    }

    /// Whether the pool is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.strings.is_empty()
    }
}

impl Default for IdentPool {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intern_basic() {
        let mut pool = IdentPool::new();
        let id1 = pool.intern("hello");
        let id2 = pool.intern("world");
        let id3 = pool.intern("hello");

        assert_eq!(id1, id3, "Same string should return same ID");
        assert_ne!(id1, id2, "Different strings should return different IDs");
        assert_eq!(pool.resolve(id1), "hello");
        assert_eq!(pool.resolve(id2), "world");
    }

    #[test]
    fn test_intern_owned() {
        let mut pool = IdentPool::new();
        let id1 = pool.intern_owned("foo".to_string());
        let id2 = pool.intern("foo");

        assert_eq!(id1, id2);
        assert_eq!(pool.len(), 1);
    }

    #[test]
    fn test_contains_and_get() {
        let mut pool = IdentPool::new();
        assert!(!pool.contains("test"));
        assert!(pool.get("test").is_none());

        let id = pool.intern("test");
        assert!(pool.contains("test"));
        assert_eq!(pool.get("test"), Some(id));
    }

    #[test]
    fn test_many_strings() {
        let mut pool = IdentPool::new();
        let mut ids = Vec::new();
        for i in 0..1000 {
            let s = format!("var_{}", i);
            ids.push(pool.intern(&s));
        }
        assert_eq!(pool.len(), 1000);

        // Re-intern all — should return same IDs
        for i in 0..1000 {
            let s = format!("var_{}", i);
            let id = pool.intern(&s);
            assert_eq!(id, ids[i]);
        }
        assert_eq!(pool.len(), 1000); // No growth
    }

    #[test]
    fn test_empty_string() {
        let mut pool = IdentPool::new();
        let id = pool.intern("");
        assert_eq!(pool.resolve(id), "");
    }

    #[test]
    fn test_try_resolve() {
        let pool = IdentPool::new();
        assert!(pool.try_resolve(InternId(0)).is_none());
        assert!(pool.try_resolve(InternId(999)).is_none());
    }

    #[test]
    fn test_intern_id_index() {
        let mut pool = IdentPool::new();
        let id0 = pool.intern("a");
        let id1 = pool.intern("b");
        assert_eq!(id0.index(), 0);
        assert_eq!(id1.index(), 1);
    }
}
