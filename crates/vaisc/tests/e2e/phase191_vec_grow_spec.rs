//! Phase 191 #10: Vec_grow specialization scheduled from Vec_push body.
//!
//! Before this fix, `Vec_push$T` body contained a call to `@.grow()` which
//! resolved to `@Vec_grow` (unmangled) because the on-demand specialization
//! path only handled Strategy 2 (empty generics). When `Vec<T>.push` was
//! specialized with concrete T, its body's `@.grow()` went through the
//! all-concrete path in method_call.rs:189 which fell back to `base` instead
//! of scheduling `Vec_grow$T`. LLVM then failed to link `@Vec_grow`.
//!
//! Fix: when `mangled` is not yet in `types.functions`, also check
//! `generated_functions` and attempt on-demand specialization via
//! `try_generate_vec_specialization`. The generalized try-path handles any
//! generic struct with a known template (not just {Vec, HashMap, Option}).
//!
//! See ROADMAP Phase 191 #10, RFC-002 §9.8.

use super::helpers::*;

/// An inline Vec<i64> that triggers Vec_push$i64, which in turn calls
/// @.grow(). Before the fix, the generated IR referenced `@Vec_grow`
/// (unmangled) which was never defined → link failure.
///
/// We use an inline local Vec struct with 5 fields (data, len, cap,
/// elem_size, owned) to match stdlib Vec<T>'s layout (Phase 191 #2a Iter A).
/// This avoids the separate "U std/vec" dependency while still exercising
/// the same dispatch path.
#[test]
fn e2e_phase191_vec_grow_spec_from_push() {
    assert_exit_code(
        r#"
# Local Vec<T> definition mirroring std/vec layout
S Vec<T> {
    data: i64,
    len: i64,
    cap: i64,
    elem_size: i64,
    owned: i64
}

X Vec<T> {
    F grow(&self) -> i64 {
        new_cap := I self.cap * 2 < 8 { 8 } EL { self.cap * 2 }
        new_data := malloc(new_cap * self.elem_size)
        memcpy(new_data, self.data, self.len * self.elem_size)
        free(self.data)
        self.data = new_data
        self.cap = new_cap
        new_cap
    }

    F push(&self, value: T) -> i64 {
        I self.len >= self.cap {
            @.grow()
        }
        ptr := self.data + self.len * self.elem_size
        store_typed(ptr, value)
        self.len = self.len + 1
        self.len
    }

    F drop(&self) -> i64 {
        free(self.data)
        self.data = 0
        0
    }
}

F main() -> i64 {
    v := Vec { data: 0, len: 0, cap: 0, elem_size: 8, owned: 0 }
    # Triggers Vec_grow$i64 via Vec_push$i64's @.grow() path
    v.push(1)
    v.push(2)
    v.push(3)
    v.drop()
    0
}
"#,
        0,
    );
}
