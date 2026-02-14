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
