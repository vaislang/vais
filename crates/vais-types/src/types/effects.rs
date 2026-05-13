//! Effect system for tracking function side effects

use std::collections::HashSet;

// ============================================================================
// Effect System
// ============================================================================

/// Effect kinds representing different types of side effects
///
/// The effect system tracks what side effects a function may have,
/// enabling purity checking, optimization, and formal verification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Effect {
    /// Pure - no observable side effects
    /// Can be safely memoized, reordered, or eliminated
    Pure,

    /// Read - reads from shared/global state
    /// Can be reordered with other reads
    Read,

    /// Write - writes to shared/global state
    /// Cannot be reordered with reads or writes
    Write,

    /// Allocate - allocates memory (heap allocation)
    /// Generally side-effect free but may fail
    Alloc,

    /// IO - performs input/output operations
    /// Console, file system, network
    IO,

    /// Async - may suspend execution
    /// Async/await, yield, sleep
    Async,

    /// Panic - may panic or abort
    /// unwrap, assert, divide by zero
    Panic,

    /// NonDet - non-deterministic (random, time)
    /// Different results on each call
    NonDet,

    /// Unsafe - performs unsafe operations
    /// Raw pointer dereference, FFI calls
    Unsafe,

    /// Diverge - may not terminate
    /// Infinite loops, recursion without base case
    Diverge,
}

impl std::fmt::Display for Effect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Effect::Pure => write!(f, "pure"),
            Effect::Read => write!(f, "read"),
            Effect::Write => write!(f, "write"),
            Effect::Alloc => write!(f, "alloc"),
            Effect::IO => write!(f, "io"),
            Effect::Async => write!(f, "async"),
            Effect::Panic => write!(f, "panic"),
            Effect::NonDet => write!(f, "nondet"),
            Effect::Unsafe => write!(f, "unsafe"),
            Effect::Diverge => write!(f, "diverge"),
        }
    }
}

/// Effect set - represents the combination of effects a function may have
///
/// Effect sets form a lattice where:
/// - Bottom (Pure) ⊆ All effects
/// - Top (All effects) is the supremum
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EffectSet {
    /// Set of individual effects
    effects: HashSet<Effect>,
}

impl EffectSet {
    /// Create a new empty effect set (pure)
    pub fn pure() -> Self {
        Self {
            effects: HashSet::new(),
        }
    }

    /// Create an effect set with a single effect
    pub fn single(effect: Effect) -> Self {
        let mut effects = HashSet::new();
        if effect != Effect::Pure {
            effects.insert(effect);
        }
        Self { effects }
    }

    /// Create an effect set with multiple effects
    pub fn from_effects(effects: impl IntoIterator<Item = Effect>) -> Self {
        let mut set = HashSet::new();
        for effect in effects {
            if effect != Effect::Pure {
                set.insert(effect);
            }
        }
        Self { effects: set }
    }

    /// Check if this is a pure (empty) effect set
    pub fn is_pure(&self) -> bool {
        self.effects.is_empty()
    }

    /// Check if this effect set is read-only (no writes, IO, etc.)
    pub fn is_readonly(&self) -> bool {
        !self.effects.contains(&Effect::Write)
            && !self.effects.contains(&Effect::IO)
            && !self.effects.contains(&Effect::Alloc)
    }

    /// Check if this effect set contains a specific effect
    pub fn contains(&self, effect: Effect) -> bool {
        if effect == Effect::Pure {
            return self.effects.is_empty();
        }
        self.effects.contains(&effect)
    }

    /// Add an effect to this set
    pub fn add(&mut self, effect: Effect) {
        if effect != Effect::Pure {
            self.effects.insert(effect);
        }
    }

    /// Union two effect sets (combines all effects)
    pub fn union(&self, other: &EffectSet) -> EffectSet {
        EffectSet {
            effects: self.effects.union(&other.effects).copied().collect(),
        }
    }

    /// Intersection of two effect sets
    pub fn intersection(&self, other: &EffectSet) -> EffectSet {
        EffectSet {
            effects: self.effects.intersection(&other.effects).copied().collect(),
        }
    }

    /// Check if this effect set is a subset of another
    pub fn is_subset_of(&self, other: &EffectSet) -> bool {
        self.effects.is_subset(&other.effects)
    }

    /// Get all effects in this set
    pub fn effects(&self) -> impl Iterator<Item = &Effect> {
        self.effects.iter()
    }

    /// Create common effect sets
    pub fn io() -> Self {
        Self::from_effects([Effect::IO, Effect::Panic])
    }

    pub fn alloc() -> Self {
        Self::from_effects([Effect::Alloc, Effect::Panic])
    }

    pub fn read_write() -> Self {
        Self::from_effects([Effect::Read, Effect::Write])
    }

    pub fn total() -> Self {
        Self::from_effects([
            Effect::Read,
            Effect::Write,
            Effect::Alloc,
            Effect::IO,
            Effect::Async,
            Effect::Panic,
            Effect::NonDet,
            Effect::Unsafe,
            Effect::Diverge,
        ])
    }
}

impl std::fmt::Display for EffectSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.effects.is_empty() {
            write!(f, "pure")
        } else {
            let effects: Vec<_> = self.effects.iter().map(|e| e.to_string()).collect();
            write!(f, "{{{}}}", effects.join(", "))
        }
    }
}

impl std::hash::Hash for EffectSet {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // Hash in a deterministic order
        let mut effects: Vec<_> = self.effects.iter().collect();
        effects.sort_by_key(|e| format!("{:?}", e));
        for effect in effects {
            effect.hash(state);
        }
    }
}

/// Function effect annotation - how effects are declared/inferred
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum EffectAnnotation {
    /// No annotation - infer from body
    #[default]
    Infer,
    /// Explicitly declared as pure
    Pure,
    /// Explicitly declared with specific effects
    Declared(EffectSet),
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========== Effect Display ==========

    #[test]
    fn test_effect_display() {
        assert_eq!(Effect::Pure.to_string(), "pure");
        assert_eq!(Effect::Read.to_string(), "read");
        assert_eq!(Effect::Write.to_string(), "write");
        assert_eq!(Effect::Alloc.to_string(), "alloc");
        assert_eq!(Effect::IO.to_string(), "io");
        assert_eq!(Effect::Async.to_string(), "async");
        assert_eq!(Effect::Panic.to_string(), "panic");
        assert_eq!(Effect::NonDet.to_string(), "nondet");
        assert_eq!(Effect::Unsafe.to_string(), "unsafe");
        assert_eq!(Effect::Diverge.to_string(), "diverge");
    }

    // ========== EffectSet constructors ==========

    #[test]
    fn test_effect_set_pure() {
        let set = EffectSet::pure();
        assert!(set.is_pure());
        assert_eq!(set.to_string(), "pure");
    }

    #[test]
    fn test_effect_set_single() {
        let set = EffectSet::single(Effect::IO);
        assert!(!set.is_pure());
        assert!(set.contains(Effect::IO));
        assert!(!set.contains(Effect::Read));
    }

    #[test]
    fn test_effect_set_single_pure_ignored() {
        let set = EffectSet::single(Effect::Pure);
        assert!(set.is_pure());
    }

    #[test]
    fn test_effect_set_from_effects() {
        let set = EffectSet::from_effects([Effect::Read, Effect::Write, Effect::Pure]);
        assert!(set.contains(Effect::Read));
        assert!(set.contains(Effect::Write));
        assert!(!set.is_pure());
    }

    // ========== EffectSet operations ==========

    #[test]
    fn test_effect_set_add() {
        let mut set = EffectSet::pure();
        set.add(Effect::IO);
        assert!(set.contains(Effect::IO));
        assert!(!set.is_pure());
    }

    #[test]
    fn test_effect_set_add_pure_no_op() {
        let mut set = EffectSet::pure();
        set.add(Effect::Pure);
        assert!(set.is_pure());
    }

    #[test]
    fn test_effect_set_union() {
        let a = EffectSet::single(Effect::Read);
        let b = EffectSet::single(Effect::Write);
        let result = a.union(&b);
        assert!(result.contains(Effect::Read));
        assert!(result.contains(Effect::Write));
    }

    #[test]
    fn test_effect_set_intersection() {
        let a = EffectSet::from_effects([Effect::Read, Effect::Write]);
        let b = EffectSet::from_effects([Effect::Write, Effect::IO]);
        let result = a.intersection(&b);
        assert!(result.contains(Effect::Write));
        assert!(!result.contains(Effect::Read));
        assert!(!result.contains(Effect::IO));
    }

    #[test]
    fn test_effect_set_is_subset_of() {
        let small = EffectSet::single(Effect::Read);
        let big = EffectSet::from_effects([Effect::Read, Effect::Write]);
        assert!(small.is_subset_of(&big));
        assert!(!big.is_subset_of(&small));
    }

    #[test]
    fn test_effect_set_pure_is_subset_of_everything() {
        let pure = EffectSet::pure();
        let any = EffectSet::single(Effect::IO);
        assert!(pure.is_subset_of(&any));
    }

    // ========== EffectSet predicates ==========

    #[test]
    fn test_effect_set_is_readonly() {
        let pure = EffectSet::pure();
        assert!(pure.is_readonly());

        let read_only = EffectSet::single(Effect::Read);
        assert!(read_only.is_readonly());

        let with_write = EffectSet::single(Effect::Write);
        assert!(!with_write.is_readonly());

        let with_io = EffectSet::single(Effect::IO);
        assert!(!with_io.is_readonly());

        let with_alloc = EffectSet::single(Effect::Alloc);
        assert!(!with_alloc.is_readonly());
    }

    #[test]
    fn test_effect_set_contains_pure() {
        let pure = EffectSet::pure();
        assert!(pure.contains(Effect::Pure));

        let non_pure = EffectSet::single(Effect::IO);
        assert!(!non_pure.contains(Effect::Pure));
    }

    // ========== Common constructors ==========

    #[test]
    fn test_effect_set_io() {
        let set = EffectSet::io();
        assert!(set.contains(Effect::IO));
        assert!(set.contains(Effect::Panic));
        assert!(!set.contains(Effect::Read));
    }

    #[test]
    fn test_effect_set_alloc() {
        let set = EffectSet::alloc();
        assert!(set.contains(Effect::Alloc));
        assert!(set.contains(Effect::Panic));
    }

    #[test]
    fn test_effect_set_read_write() {
        let set = EffectSet::read_write();
        assert!(set.contains(Effect::Read));
        assert!(set.contains(Effect::Write));
    }

    #[test]
    fn test_effect_set_total() {
        let set = EffectSet::total();
        assert!(set.contains(Effect::Read));
        assert!(set.contains(Effect::Write));
        assert!(set.contains(Effect::Alloc));
        assert!(set.contains(Effect::IO));
        assert!(set.contains(Effect::Async));
        assert!(set.contains(Effect::Panic));
        assert!(set.contains(Effect::NonDet));
        assert!(set.contains(Effect::Unsafe));
        assert!(set.contains(Effect::Diverge));
    }

    // ========== EffectAnnotation ==========

    #[test]
    fn test_effect_annotation_default() {
        let ann: EffectAnnotation = Default::default();
        assert_eq!(ann, EffectAnnotation::Infer);
    }

    // ========== Display ==========

    #[test]
    fn test_effect_set_display_pure() {
        assert_eq!(EffectSet::pure().to_string(), "pure");
    }

    #[test]
    fn test_effect_set_display_single() {
        let set = EffectSet::single(Effect::IO);
        let display = set.to_string();
        assert!(display.contains("io"));
    }

    // ========== Equality & Hash ==========

    #[test]
    fn test_effect_set_equality() {
        let a = EffectSet::from_effects([Effect::Read, Effect::Write]);
        let b = EffectSet::from_effects([Effect::Write, Effect::Read]);
        assert_eq!(a, b);
    }

    #[test]
    fn test_effect_set_default_is_pure() {
        let set: EffectSet = Default::default();
        assert!(set.is_pure());
    }
}
