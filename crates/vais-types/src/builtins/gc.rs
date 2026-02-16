//! Garbage collector built-in functions

use super::*;

impl TypeChecker {
    pub(super) fn register_gc_builtins(&mut self) {
        // ===== GC functions used by gc examples =====
        let gc_fns = vec![
            ("gc_init", vec![], ResolvedType::I64),
            (
                "gc_alloc",
                vec![
                    ("size".to_string(), ResolvedType::I64, false),
                    ("type_id".to_string(), ResolvedType::I64, false),
                ],
                ResolvedType::I64,
            ),
            ("gc_collect", vec![], ResolvedType::I64),
            (
                "gc_add_root",
                vec![("ptr".to_string(), ResolvedType::I64, false)],
                ResolvedType::I64,
            ),
            (
                "gc_remove_root",
                vec![("ptr".to_string(), ResolvedType::I64, false)],
                ResolvedType::I64,
            ),
            ("gc_bytes_allocated", vec![], ResolvedType::I64),
            ("gc_objects_count", vec![], ResolvedType::I64),
            ("gc_collections", vec![], ResolvedType::I64),
            (
                "gc_set_threshold",
                vec![("threshold".to_string(), ResolvedType::I64, false)],
                ResolvedType::I64,
            ),
            ("gc_print_stats", vec![], ResolvedType::I64),
        ];
        for (name, params, ret) in gc_fns {
            self.functions.insert(
                name.to_string(),
                FunctionSig {
                    name: name.to_string(),
                    params,
                    ret,
                    ..Default::default()
                },
            );
        }
    }
}
