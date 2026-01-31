//! Generational Garbage Collector
//!
//! Implements a generational GC with Young/Old generation separation,
//! Minor/Major GC distinction, and card marking for remembered sets.
//!
//! # Design
//!
//! - **Young Generation**: Small, frequently collected. New objects go here.
//! - **Old Generation**: Large, infrequently collected. Promoted objects live here.
//! - **Minor GC**: Only collects young generation (fast).
//! - **Major GC**: Collects both generations (thorough).
//! - **Card Marking**: Tracks old→young pointers for efficient minor GC.
//! - **Promotion**: Objects surviving N minor GCs are promoted to old generation.

use std::collections::{HashMap, HashSet};

/// Generation identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Generation {
    /// Young generation - new objects, frequently collected.
    Young,
    /// Old generation - long-lived objects, infrequently collected.
    Old,
}

/// Object header for generational GC.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct GenGcObjectHeader {
    /// Size of object data in bytes.
    pub size: usize,
    /// Mark bit for mark-and-sweep.
    pub marked: bool,
    /// Which generation this object belongs to.
    pub generation: Generation,
    /// Number of GC cycles this object has survived.
    pub age: u8,
    /// Type ID for debugging.
    pub type_id: u32,
}

impl GenGcObjectHeader {
    pub fn new(size: usize, type_id: u32) -> Self {
        Self {
            size,
            marked: false,
            generation: Generation::Young,
            age: 0,
            type_id,
        }
    }
}

/// GC object with generational header.
#[derive(Debug)]
pub struct GenGcObject {
    pub header: GenGcObjectHeader,
    pub data: Vec<u8>,
}

impl GenGcObject {
    pub fn new(size: usize, type_id: u32) -> Self {
        Self {
            header: GenGcObjectHeader::new(size, type_id),
            data: vec![0u8; size],
        }
    }

    pub fn data_ptr(&self) -> *const u8 {
        self.data.as_ptr()
    }

    pub fn data_ptr_mut(&mut self) -> *mut u8 {
        self.data.as_mut_ptr()
    }
}

/// Card marking table for tracking old→young references.
///
/// Memory is divided into "cards" (regions). When a pointer in the old
/// generation is updated to point to a young generation object, the
/// corresponding card is marked dirty. During minor GC, only dirty
/// cards need to be scanned for roots into the young generation.
pub struct CardTable {
    /// One byte per card: 0 = clean, 1 = dirty.
    cards: Vec<u8>,
    /// Size of each card in bytes (typically 512).
    card_size: usize,
    /// Base address of the managed heap region.
    base_addr: usize,
}

impl CardTable {
    /// Creates a new card table covering `heap_size` bytes.
    pub fn new(heap_size: usize, card_size: usize) -> Self {
        let num_cards = (heap_size + card_size - 1) / card_size;
        Self {
            cards: vec![0u8; num_cards],
            card_size,
            base_addr: 0,
        }
    }

    /// Sets the base address of the heap region.
    pub fn set_base(&mut self, base: usize) {
        self.base_addr = base;
    }

    /// Marks the card containing the given address as dirty.
    pub fn mark_dirty(&mut self, addr: usize) {
        if addr >= self.base_addr {
            let offset = addr - self.base_addr;
            let card_index = offset / self.card_size;
            if card_index < self.cards.len() {
                self.cards[card_index] = 1;
            }
        }
    }

    /// Checks if the card containing the given address is dirty.
    pub fn is_dirty(&self, addr: usize) -> bool {
        if addr >= self.base_addr {
            let offset = addr - self.base_addr;
            let card_index = offset / self.card_size;
            if card_index < self.cards.len() {
                return self.cards[card_index] != 0;
            }
        }
        false
    }

    /// Clears all dirty cards.
    pub fn clear_all(&mut self) {
        for card in &mut self.cards {
            *card = 0;
        }
    }

    /// Returns indices of all dirty cards.
    pub fn dirty_cards(&self) -> Vec<usize> {
        self.cards
            .iter()
            .enumerate()
            .filter(|(_, &v)| v != 0)
            .map(|(i, _)| i)
            .collect()
    }

    /// Returns the address range for a card index.
    pub fn card_range(&self, card_index: usize) -> (usize, usize) {
        let start = self.base_addr + card_index * self.card_size;
        let end = start + self.card_size;
        (start, end)
    }
}

/// Remembered set for tracking old→young pointers explicitly.
pub struct RememberedSet {
    /// Set of (old_object_ptr, young_object_ptr) pairs.
    entries: HashSet<(usize, usize)>,
}

impl RememberedSet {
    pub fn new() -> Self {
        Self {
            entries: HashSet::new(),
        }
    }

    /// Records an old→young pointer.
    pub fn add(&mut self, old_ptr: usize, young_ptr: usize) {
        self.entries.insert((old_ptr, young_ptr));
    }

    /// Removes entries involving a specific young object (e.g., after promotion).
    pub fn remove_young(&mut self, young_ptr: usize) {
        self.entries.retain(|&(_, y)| y != young_ptr);
    }

    /// Returns all young pointers referenced from old generation.
    pub fn young_roots(&self) -> Vec<usize> {
        self.entries.iter().map(|&(_, y)| y).collect()
    }

    /// Clears all entries.
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Number of entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl Default for RememberedSet {
    fn default() -> Self {
        Self::new()
    }
}

/// Collection type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollectionType {
    /// Minor GC - young generation only.
    Minor,
    /// Major GC - both generations.
    Major,
}

/// Statistics for generational GC.
#[derive(Debug, Clone, Default)]
pub struct GenGcStats {
    /// Total minor collections.
    pub minor_collections: u64,
    /// Total major collections.
    pub major_collections: u64,
    /// Objects in young generation.
    pub young_objects: usize,
    /// Objects in old generation.
    pub old_objects: usize,
    /// Bytes in young generation.
    pub young_bytes: usize,
    /// Bytes in old generation.
    pub old_bytes: usize,
    /// Total objects promoted.
    pub total_promoted: u64,
    /// Objects freed in last minor GC.
    pub last_minor_freed: usize,
    /// Objects freed in last major GC.
    pub last_major_freed: usize,
    /// Remembered set size.
    pub remembered_set_size: usize,
}

/// Configuration for generational GC.
#[derive(Debug, Clone)]
pub struct GenGcConfig {
    /// Young generation threshold (bytes) - triggers minor GC.
    pub young_threshold: usize,
    /// Old generation threshold (bytes) - triggers major GC.
    pub old_threshold: usize,
    /// Number of minor GCs an object must survive to be promoted.
    pub promotion_age: u8,
    /// Card size for card table (bytes).
    pub card_size: usize,
    /// Maximum heap size for card table coverage.
    pub max_heap_size: usize,
}

impl Default for GenGcConfig {
    fn default() -> Self {
        Self {
            young_threshold: 256 * 1024,     // 256 KB
            old_threshold: 4 * 1024 * 1024,  // 4 MB
            promotion_age: 3,
            card_size: 512,
            max_heap_size: 64 * 1024 * 1024, // 64 MB
        }
    }
}

/// Generational Garbage Collector.
pub struct GenerationalGc {
    /// Young generation objects (ptr -> object).
    young: HashMap<usize, GenGcObject>,
    /// Old generation objects (ptr -> object).
    old: HashMap<usize, GenGcObject>,
    /// Root set.
    roots: HashSet<usize>,
    /// Remembered set (old→young references).
    remembered_set: RememberedSet,
    /// Card table for write barrier optimization.
    card_table: CardTable,
    /// Configuration.
    config: GenGcConfig,
    /// Statistics.
    stats: GenGcStats,
    /// Bytes allocated in young generation since last minor GC.
    young_bytes_since_gc: usize,
    /// Bytes allocated in old generation since last major GC.
    old_bytes_since_gc: usize,
}

impl GenerationalGc {
    /// Creates a new generational GC with default configuration.
    pub fn new() -> Self {
        Self::with_config(GenGcConfig::default())
    }

    /// Creates a new generational GC with custom configuration.
    pub fn with_config(config: GenGcConfig) -> Self {
        let card_table = CardTable::new(config.max_heap_size, config.card_size);
        Self {
            young: HashMap::new(),
            old: HashMap::new(),
            roots: HashSet::new(),
            remembered_set: RememberedSet::new(),
            card_table,
            config,
            stats: GenGcStats::default(),
            young_bytes_since_gc: 0,
            old_bytes_since_gc: 0,
        }
    }

    /// Allocates a new object in the young generation.
    pub fn alloc(&mut self, size: usize, type_id: u32) -> *mut u8 {
        // Check if minor GC is needed
        self.young_bytes_since_gc += size;
        if self.young_bytes_since_gc >= self.config.young_threshold {
            self.collect_minor();
        }

        let mut obj = GenGcObject::new(size, type_id);
        let ptr = obj.data_ptr_mut() as usize;

        self.stats.young_objects += 1;
        self.stats.young_bytes += size;
        self.young.insert(ptr, obj);

        ptr as *mut u8
    }

    /// Registers a root pointer.
    pub fn add_root(&mut self, ptr: usize) {
        if ptr != 0 {
            self.roots.insert(ptr);
        }
    }

    /// Unregisters a root pointer.
    pub fn remove_root(&mut self, ptr: usize) {
        self.roots.remove(&ptr);
    }

    /// Write barrier - called when a pointer field is modified.
    ///
    /// If an old generation object is updated to point to a young generation
    /// object, we record this in the remembered set and card table.
    pub fn write_barrier(&mut self, source: usize, _old_target: usize, new_target: usize) {
        if new_target == 0 {
            return;
        }

        // Check if source is in old generation and target is in young generation
        let source_is_old = self.old.contains_key(&source);
        let target_is_young = self.young.contains_key(&new_target);

        if source_is_old && target_is_young {
            self.remembered_set.add(source, new_target);
            self.card_table.mark_dirty(source);
        }
    }

    /// Performs a minor GC (young generation only).
    pub fn collect_minor(&mut self) {
        self.stats.minor_collections += 1;

        // Clear marks on young generation
        for obj in self.young.values_mut() {
            obj.header.marked = false;
        }

        // Mark phase: roots + remembered set
        let root_ptrs: Vec<usize> = self.roots.iter().copied().collect();
        let remembered_young: Vec<usize> = self.remembered_set.young_roots();

        // Mark from roots (only young objects)
        for ptr in &root_ptrs {
            self.mark_young(*ptr);
        }

        // Mark from remembered set (old→young references)
        for ptr in &remembered_young {
            self.mark_young(*ptr);
        }

        // Increment age of surviving young objects first
        for obj in self.young.values_mut() {
            if obj.header.marked {
                obj.header.age = obj.header.age.saturating_add(1);
            }
        }

        // Promotion: surviving young objects with sufficient age → old generation
        let mut to_promote = Vec::new();
        let mut to_free = Vec::new();

        for (&ptr, obj) in &self.young {
            if obj.header.marked {
                if obj.header.age >= self.config.promotion_age {
                    to_promote.push(ptr);
                }
            } else {
                to_free.push(ptr);
            }
        }

        // Promote objects
        for ptr in &to_promote {
            if let Some(mut obj) = self.young.remove(ptr) {
                obj.header.generation = Generation::Old;
                self.stats.young_objects = self.stats.young_objects.saturating_sub(1);
                self.stats.young_bytes = self.stats.young_bytes.saturating_sub(obj.header.size);
                self.stats.old_objects += 1;
                self.stats.old_bytes += obj.header.size;
                self.stats.total_promoted += 1;
                self.old_bytes_since_gc += obj.header.size;
                self.remembered_set.remove_young(*ptr);
                self.old.insert(*ptr, obj);
            }
        }

        // Free unreachable young objects
        for ptr in &to_free {
            if let Some(obj) = self.young.remove(ptr) {
                self.stats.young_objects = self.stats.young_objects.saturating_sub(1);
                self.stats.young_bytes = self.stats.young_bytes.saturating_sub(obj.header.size);
                self.remembered_set.remove_young(*ptr);
            }
        }

        self.stats.last_minor_freed = to_free.len();

        // Clear card table
        self.card_table.clear_all();
        self.young_bytes_since_gc = 0;

        // Check if major GC is needed
        if self.old_bytes_since_gc >= self.config.old_threshold {
            self.collect_major();
        }

        self.stats.remembered_set_size = self.remembered_set.len();
    }

    /// Performs a major GC (both generations).
    pub fn collect_major(&mut self) {
        self.stats.major_collections += 1;

        // Clear marks on all objects
        for obj in self.young.values_mut() {
            obj.header.marked = false;
        }
        for obj in self.old.values_mut() {
            obj.header.marked = false;
        }

        // Mark from roots (all objects)
        let root_ptrs: Vec<usize> = self.roots.iter().copied().collect();
        for ptr in &root_ptrs {
            self.mark_all(*ptr);
        }

        // Sweep young generation
        let young_to_free: Vec<usize> = self.young
            .iter()
            .filter(|(_, obj)| !obj.header.marked)
            .map(|(&ptr, _)| ptr)
            .collect();

        for ptr in &young_to_free {
            if let Some(obj) = self.young.remove(ptr) {
                self.stats.young_objects = self.stats.young_objects.saturating_sub(1);
                self.stats.young_bytes = self.stats.young_bytes.saturating_sub(obj.header.size);
            }
        }

        // Sweep old generation
        let old_to_free: Vec<usize> = self.old
            .iter()
            .filter(|(_, obj)| !obj.header.marked)
            .map(|(&ptr, _)| ptr)
            .collect();

        for ptr in &old_to_free {
            if let Some(obj) = self.old.remove(ptr) {
                self.stats.old_objects = self.stats.old_objects.saturating_sub(1);
                self.stats.old_bytes = self.stats.old_bytes.saturating_sub(obj.header.size);
            }
        }

        self.stats.last_major_freed = young_to_free.len() + old_to_free.len();

        // Rebuild remembered set (scan old generation for young pointers)
        self.remembered_set.clear();
        let young_ptrs: HashSet<usize> = self.young.keys().copied().collect();
        for (&old_ptr, old_obj) in &self.old {
            let children = self.scan_object_data(&old_obj.data, old_obj.header.size);
            for child in children {
                if young_ptrs.contains(&child) {
                    self.remembered_set.add(old_ptr, child);
                    self.card_table.mark_dirty(old_ptr);
                }
            }
        }

        self.old_bytes_since_gc = 0;
        self.stats.remembered_set_size = self.remembered_set.len();
    }

    /// Force a full collection (minor + major).
    pub fn collect_full(&mut self) {
        self.collect_minor();
        self.collect_major();
    }

    /// Mark a young generation object and its young children.
    fn mark_young(&mut self, ptr: usize) {
        let should_scan = if let Some(obj) = self.young.get_mut(&ptr) {
            if obj.header.marked {
                false
            } else {
                obj.header.marked = true;
                true
            }
        } else {
            false
        };

        if should_scan {
            let (data_clone, size) = {
                let obj = self.young.get(&ptr).unwrap();
                (obj.data.clone(), obj.header.size)
            };
            let children = self.scan_object_data(&data_clone, size);
            for child in children {
                if self.young.contains_key(&child) {
                    self.mark_young(child);
                }
            }
        }
    }

    /// Mark any object (young or old) and its children.
    fn mark_all(&mut self, ptr: usize) {
        // Try young first
        let in_young = if let Some(obj) = self.young.get_mut(&ptr) {
            if obj.header.marked {
                return;
            }
            obj.header.marked = true;
            true
        } else {
            false
        };

        if !in_young {
            // Try old
            if let Some(obj) = self.old.get_mut(&ptr) {
                if obj.header.marked {
                    return;
                }
                obj.header.marked = true;
            } else {
                return;
            }
        }

        // Scan for children
        let (data_clone, size) = if in_young {
            let obj = self.young.get(&ptr).unwrap();
            (obj.data.clone(), obj.header.size)
        } else {
            let obj = self.old.get(&ptr).unwrap();
            (obj.data.clone(), obj.header.size)
        };

        let children = self.scan_object_data(&data_clone, size);
        for child in children {
            self.mark_all(child);
        }
    }

    /// Conservative pointer scanning on raw data.
    fn scan_object_data(&self, data: &[u8], size: usize) -> Vec<usize> {
        let ptr_size = std::mem::size_of::<usize>();
        let mut children = Vec::new();

        for offset in (0..size).step_by(ptr_size) {
            if offset + ptr_size <= data.len() {
                let potential_ptr = unsafe {
                    std::ptr::read(data.as_ptr().add(offset) as *const usize)
                };
                if self.young.contains_key(&potential_ptr) || self.old.contains_key(&potential_ptr) {
                    children.push(potential_ptr);
                }
            }
        }

        children
    }

    /// Returns whether an object is alive.
    pub fn is_alive(&self, ptr: usize) -> bool {
        self.young.contains_key(&ptr) || self.old.contains_key(&ptr)
    }

    /// Returns which generation an object belongs to.
    pub fn get_generation(&self, ptr: usize) -> Option<Generation> {
        if self.young.contains_key(&ptr) {
            Some(Generation::Young)
        } else if self.old.contains_key(&ptr) {
            Some(Generation::Old)
        } else {
            None
        }
    }

    /// Returns total number of live objects.
    pub fn object_count(&self) -> usize {
        self.young.len() + self.old.len()
    }

    /// Returns GC statistics.
    pub fn get_stats(&self) -> GenGcStats {
        self.stats.clone()
    }

    /// Sets GC configuration.
    pub fn set_config(&mut self, config: GenGcConfig) {
        self.config = config;
    }

    /// Sets the young generation threshold.
    pub fn set_young_threshold(&mut self, threshold: usize) {
        self.config.young_threshold = threshold;
    }

    /// Sets the old generation threshold.
    pub fn set_old_threshold(&mut self, threshold: usize) {
        self.config.old_threshold = threshold;
    }

    /// Sets the promotion age.
    pub fn set_promotion_age(&mut self, age: u8) {
        self.config.promotion_age = age;
    }
}

impl Default for GenerationalGc {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_allocation() {
        let mut gc = GenerationalGc::new();

        let ptr1 = gc.alloc(100, 1);
        let ptr2 = gc.alloc(200, 2);

        assert!(!ptr1.is_null());
        assert!(!ptr2.is_null());
        assert_ne!(ptr1, ptr2);

        assert_eq!(gc.object_count(), 2);
        assert_eq!(gc.get_generation(ptr1 as usize), Some(Generation::Young));
        assert_eq!(gc.get_generation(ptr2 as usize), Some(Generation::Young));
    }

    #[test]
    fn test_minor_gc_frees_unreachable() {
        let mut gc = GenerationalGc::with_config(GenGcConfig {
            young_threshold: 1024 * 1024, // Don't auto-trigger
            ..Default::default()
        });

        let ptr1 = gc.alloc(100, 1) as usize;
        let ptr2 = gc.alloc(200, 2) as usize;

        // Only root ptr1
        gc.add_root(ptr1);

        gc.collect_minor();

        assert!(gc.is_alive(ptr1));
        assert!(!gc.is_alive(ptr2));

        let stats = gc.get_stats();
        assert_eq!(stats.minor_collections, 1);
        assert_eq!(stats.last_minor_freed, 1);
    }

    #[test]
    fn test_promotion_after_age() {
        let mut gc = GenerationalGc::with_config(GenGcConfig {
            young_threshold: 1024 * 1024,
            promotion_age: 2, // Promote after 2 minor GCs
            ..Default::default()
        });

        let ptr = gc.alloc(100, 1) as usize;
        gc.add_root(ptr);

        assert_eq!(gc.get_generation(ptr), Some(Generation::Young));

        // First minor GC - age becomes 1
        gc.collect_minor();
        assert_eq!(gc.get_generation(ptr), Some(Generation::Young));

        // Second minor GC - age becomes 2, triggers promotion
        gc.collect_minor();
        assert_eq!(gc.get_generation(ptr), Some(Generation::Old));

        let stats = gc.get_stats();
        assert_eq!(stats.total_promoted, 1);
        assert_eq!(stats.old_objects, 1);
    }

    #[test]
    fn test_major_gc_collects_old() {
        let mut gc = GenerationalGc::with_config(GenGcConfig {
            young_threshold: 1024 * 1024,
            old_threshold: 1024 * 1024,
            promotion_age: 0, // Immediate promotion
            ..Default::default()
        });

        let ptr1 = gc.alloc(100, 1) as usize;
        let ptr2 = gc.alloc(200, 2) as usize;
        gc.add_root(ptr1);
        gc.add_root(ptr2);

        // Promote both to old generation
        gc.collect_minor();

        assert_eq!(gc.get_generation(ptr1), Some(Generation::Old));
        assert_eq!(gc.get_generation(ptr2), Some(Generation::Old));

        // Remove ptr2 root, then major GC should free it
        gc.remove_root(ptr2);
        gc.collect_major();

        assert!(gc.is_alive(ptr1));
        assert!(!gc.is_alive(ptr2));
    }

    #[test]
    fn test_write_barrier_and_remembered_set() {
        let mut gc = GenerationalGc::with_config(GenGcConfig {
            young_threshold: 1024 * 1024,
            old_threshold: 1024 * 1024,
            promotion_age: 0,
            ..Default::default()
        });

        // Create and promote an old object
        let old_ptr = gc.alloc(100, 1) as usize;
        gc.add_root(old_ptr);
        gc.collect_minor(); // Promotes to old

        // Create a young object
        let young_ptr = gc.alloc(200, 2) as usize;
        // Don't root young_ptr directly

        // Write barrier: old object now points to young object
        gc.write_barrier(old_ptr, 0, young_ptr);

        // Minor GC should keep young_ptr alive via remembered set
        gc.collect_minor();

        assert!(gc.is_alive(young_ptr));
    }

    #[test]
    fn test_card_table() {
        let mut card_table = CardTable::new(4096, 512);
        card_table.set_base(0);

        assert!(!card_table.is_dirty(100));

        card_table.mark_dirty(100);
        assert!(card_table.is_dirty(100));
        // Same card (0-511)
        assert!(card_table.is_dirty(0));
        assert!(card_table.is_dirty(511));

        // Different card
        assert!(!card_table.is_dirty(512));

        card_table.mark_dirty(512);
        assert!(card_table.is_dirty(512));

        let dirty = card_table.dirty_cards();
        assert_eq!(dirty.len(), 2);

        card_table.clear_all();
        assert!(!card_table.is_dirty(100));
        assert!(!card_table.is_dirty(512));
    }

    #[test]
    fn test_remembered_set() {
        let mut rs = RememberedSet::new();

        rs.add(100, 200);
        rs.add(100, 300);
        rs.add(400, 200);

        assert_eq!(rs.len(), 3);

        let young = rs.young_roots();
        assert!(young.contains(&200));
        assert!(young.contains(&300));

        rs.remove_young(200);
        assert_eq!(rs.len(), 1);
    }

    #[test]
    fn test_stress_allocation() {
        let mut gc = GenerationalGc::with_config(GenGcConfig {
            young_threshold: 5000, // Low threshold to trigger frequent minor GCs
            old_threshold: 50000,
            promotion_age: 2,
            ..Default::default()
        });

        // Allocate many objects, keeping only some rooted
        let mut rooted = Vec::new();
        for i in 0..200 {
            let ptr = gc.alloc(100, i as u32) as usize;
            if i % 10 == 0 {
                gc.add_root(ptr);
                rooted.push(ptr);
            }
        }

        // All rooted objects should be alive
        for ptr in &rooted {
            assert!(gc.is_alive(*ptr));
        }

        let stats = gc.get_stats();
        // Should have triggered some minor GCs
        assert!(stats.minor_collections > 0);
    }

    #[test]
    fn test_full_collection() {
        let mut gc = GenerationalGc::with_config(GenGcConfig {
            young_threshold: 1024 * 1024,
            old_threshold: 1024 * 1024,
            promotion_age: 0,
            ..Default::default()
        });

        let ptr1 = gc.alloc(100, 1) as usize;
        let ptr2 = gc.alloc(200, 2) as usize;
        let ptr3 = gc.alloc(300, 3) as usize;

        gc.add_root(ptr1);
        gc.add_root(ptr3);

        gc.collect_full();

        assert!(gc.is_alive(ptr1));
        assert!(!gc.is_alive(ptr2));
        assert!(gc.is_alive(ptr3));

        let stats = gc.get_stats();
        assert!(stats.minor_collections >= 1);
        assert!(stats.major_collections >= 1);
    }

    #[test]
    fn test_generation_stats() {
        let mut gc = GenerationalGc::with_config(GenGcConfig {
            young_threshold: 1024 * 1024,
            old_threshold: 1024 * 1024,
            promotion_age: 1,
            ..Default::default()
        });

        gc.alloc(100, 1);
        gc.alloc(200, 2);

        let stats = gc.get_stats();
        assert_eq!(stats.young_objects, 2);
        assert_eq!(stats.old_objects, 0);
        assert_eq!(stats.young_bytes, 300);
    }

    #[test]
    fn test_auto_trigger_minor_gc() {
        let mut gc = GenerationalGc::with_config(GenGcConfig {
            young_threshold: 500, // Very low threshold
            old_threshold: 1024 * 1024,
            promotion_age: 3,
            ..Default::default()
        });

        // Allocate enough to trigger auto minor GC
        for i in 0..20 {
            let ptr = gc.alloc(100, i as u32);
            if i < 5 {
                gc.add_root(ptr as usize);
            }
        }

        let stats = gc.get_stats();
        assert!(stats.minor_collections > 0, "Minor GC should have been auto-triggered");
    }
}
