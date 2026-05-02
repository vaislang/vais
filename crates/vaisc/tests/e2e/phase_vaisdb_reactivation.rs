//! VaisDB reactivation fixtures.
//!
//! These tests isolate currently-known downstream failures before broad source
//! edits. Regressions here usually mean imported package type propagation
//! lost concrete collection element types before codegen.

use super::helpers::compile_to_ir;
use std::path::PathBuf;
use std::process::{Command, Output};

#[test]
fn e2e_vaisdb_synthetic_vec_index_field_still_compiles() {
    let source = r#"
U std/vec

S Policy {
    policy_name: str,
    active: bool,
}

X Policy {
    F is_active(self) -> bool {
        self.active
    }
}

F main() -> i64 {
    policies := mut Vec.new()
    policies.push(Policy { policy_name: "p1", active: true })
    I policies[0].policy_name == "p1" && policies[0].is_active() {
        R 0
    }
    R 1
}
"#;

    compile_to_ir(source).expect("synthetic Vec<struct>[i].field should compile");
}

#[test]
fn e2e_vaisdb_hashmap_get_preserves_struct_value_type() {
    let source = r#"
U std/hashmap
U std/option

S TableInfo {
    table_id: u32,
    name: str,
}

X TableInfo {
    F clone(self) -> TableInfo {
        TableInfo { table_id: self.table_id, name: self.name.clone() }
    }
}

F main() -> i64 {
    tables: HashMap<str, TableInfo> := mut HashMap.with_capacity(16)
    tables.insert("users", TableInfo { table_id: 42, name: "users" })

    table_info := mut M tables.get("users") {
        Some(info) => info.clone(),
        None => { R 1 },
    }

    I table_info.table_id == 42 {
        R 0
    }
    R 2
}
"#;

    compile_to_ir(source).expect("HashMap<str, Struct>.get should preserve V through Some binding");
}

#[test]
fn e2e_vaisdb_rag_chunking_graph_package_type_propagation_compiles() {
    let Some(output) = build_vaisdb_package_file("rag/chunking/graph.vais") else {
        return;
    };

    assert!(
        output.status.success(),
        "VaisDB package type propagation fixture should compile.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn e2e_vaisdb_security_policy_vec_binding_type_propagation_compiles() {
    let Some(output) = build_vaisdb_package_file("security/policy.vais") else {
        return;
    };

    assert!(
        output.status.success(),
        "VaisDB security policy Vec binding fixture should compile.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn e2e_vaisdb_sql_catalog_manager_api_drift_compiles() {
    let Some(output) = build_vaisdb_package_file("sql/catalog/manager.vais") else {
        return;
    };

    assert!(
        output.status.success(),
        "VaisDB SQL catalog manager fixture should compile.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn e2e_vaisdb_sql_executor_dml_api_drift_compiles() {
    let Some(output) = build_vaisdb_package_file("sql/executor/dml.vais") else {
        return;
    };

    assert!(
        output.status.success(),
        "VaisDB SQL DML executor fixture should compile.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn e2e_vaisdb_graph_mod_api_drift_compiles() {
    let Some(output) = build_vaisdb_package_file("graph/mod.vais") else {
        return;
    };

    assert!(
        output.status.success(),
        "VaisDB graph facade fixture should compile.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn e2e_vaisdb_rag_mod_api_drift_compiles() {
    let Some(output) = build_vaisdb_package_file("rag/mod.vais") else {
        return;
    };

    assert!(
        output.status.success(),
        "VaisDB RAG facade fixture should compile.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn e2e_vaisdb_planner_pipeline_fulltext_contract_deferred_compiles() {
    let Some(output) = build_vaisdb_package_file("planner/pipeline.vais") else {
        return;
    };

    assert!(
        output.status.success(),
        "VaisDB planner pipeline fixture should compile while fulltext execution is deferred.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn e2e_vaisdb_sql_executor_join_rowsource_contract_compiles() {
    let Some(output) = build_vaisdb_package_file("sql/executor/join.vais") else {
        return;
    };

    assert!(
        output.status.success(),
        "VaisDB SQL join executor fixture should compile with materialized RowSource contract.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn e2e_vaisdb_sql_executor_sort_agg_accumulator_contract_compiles() {
    let Some(output) = build_vaisdb_package_file("sql/executor/sort_agg.vais") else {
        return;
    };

    assert!(
        output.status.success(),
        "VaisDB SQL sort/aggregate executor fixture should compile with accumulator update contract.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn e2e_vaisdb_sql_executor_window_partition_contract_compiles() {
    let Some(output) = build_vaisdb_package_file("sql/executor/window.vais") else {
        return;
    };

    assert!(
        output.status.success(),
        "VaisDB SQL window executor fixture should compile with total partition and typed sort-key contract.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn e2e_vaisdb_sql_parser_expr_vec_contract_compiles() {
    let Some(output) = build_vaisdb_package_file("sql/parser/parser_expr.vais") else {
        return;
    };

    assert!(
        output.status.success(),
        "VaisDB SQL parser expression fixture should compile with typed expression Vec helpers.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn e2e_vaisdb_sql_parser_mod_token_ast_contract_compiles() {
    let Some(output) = build_vaisdb_package_file("sql/parser/mod.vais") else {
        return;
    };

    assert!(
        output.status.success(),
        "VaisDB SQL parser facade fixture should compile with explicit TokenKind disambiguation.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn e2e_vaisdb_storage_txn_conflict_decoded_lock_contract_compiles() {
    let Some(output) = build_vaisdb_package_file("storage/txn/conflict.vais") else {
        return;
    };

    assert!(
        output.status.success(),
        "VaisDB conflict detector fixture should compile with DecodedLockKey return contract.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn e2e_vaisdb_storage_txn_deadlock_edge_contract_compiles() {
    let Some(output) = build_vaisdb_package_file("storage/txn/deadlock.vais") else {
        return;
    };

    assert!(
        output.status.success(),
        "VaisDB deadlock detector fixture should compile with explicit wait-edge contract.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn e2e_vaisdb_storage_btree_insert_api_contract_compiles() {
    let Some(output) = build_vaisdb_package_file("storage/btree/insert.vais") else {
        return;
    };

    assert!(
        output.status.success(),
        "VaisDB B-tree insert fixture should compile with current latch/header APIs.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn e2e_vaisdb_vector_hnsw_cow_api_contract_compiles() {
    let Some(output) = build_vaisdb_package_file("vector/hnsw/cow.vais") else {
        return;
    };

    assert!(
        output.status.success(),
        "VaisDB HNSW CoW fixture should compile with current error, atomic, and totality contracts.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn e2e_vaisdb_vector_hnsw_insert_meta_contract_compiles() {
    let Some(output) = build_vaisdb_package_file("vector/hnsw/insert.vais") else {
        return;
    };

    assert!(
        output.status.success(),
        "VaisDB HNSW insert fixture should compile with the current HnswMeta emptiness contract.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn e2e_vaisdb_vector_hnsw_delete_store_contract_compiles() {
    let Some(output) = build_vaisdb_package_file("vector/hnsw/delete.vais") else {
        return;
    };

    assert!(
        output.status.success(),
        "VaisDB HNSW delete fixture should compile with separated NodeStore and NodeLookup contracts.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn e2e_vaisdb_vector_hnsw_wal_replay_contract_compiles() {
    let Some(output) = build_vaisdb_package_file("vector/hnsw/wal.vais") else {
        return;
    };

    assert!(
        output.status.success(),
        "VaisDB HNSW WAL fixture should compile with current replay store, buffer, and payload contracts.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn e2e_vaisdb_vector_hnsw_bulk_loader_contract_compiles() {
    let Some(output) = build_vaisdb_package_file("vector/hnsw/bulk.vais") else {
        return;
    };

    assert!(
        output.status.success(),
        "VaisDB HNSW bulk loader fixture should compile with current metadata, NodeStore, and WAL-manager contracts.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn e2e_vaisdb_vector_mod_facade_contract_compiles() {
    let Some(output) = build_vaisdb_package_file("vector/mod.vais") else {
        return;
    };

    assert!(
        output.status.success(),
        "VaisDB vector facade fixture should compile with current distance, storage, HNSW, WAL, and filtered-search contracts.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn build_vaisdb_package_file(relative_target: &str) -> Option<Output> {
    let compiler_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("failed to canonicalize compiler root");
    let workspace_root = compiler_root
        .parent()
        .expect("compiler root should have workspace parent");
    let vaisdb_src = workspace_root.join("lang/packages/vaisdb/src");
    let target = vaisdb_src.join(relative_target);

    if !target.is_file() {
        eprintln!("SKIP: vaisdb fixture not found at {}", target.display());
        return None;
    }

    let tmp_std_root = PathBuf::from("/tmp/vais-lib");
    std::fs::create_dir_all(&tmp_std_root).expect("failed to create /tmp/vais-lib");
    let std_link = tmp_std_root.join("std");
    let compiler_std = compiler_root.join("std");
    if std_link.exists() {
        let already_correct = std_link
            .canonicalize()
            .map(|path| path == compiler_std)
            .unwrap_or(false);
        if !already_correct {
            let _ = std::fs::remove_file(&std_link);
            let _ = std::fs::remove_dir(&std_link);
        }
    }
    if !std_link.exists() {
        #[cfg(unix)]
        std::os::unix::fs::symlink(&compiler_std, &std_link)
            .expect("failed to create /tmp/vais-lib/std symlink");
    }

    let out_ir = std::env::temp_dir().join(format!(
        "vaisdb_{}_{}.ll",
        relative_target.replace(['/', '.'], "_"),
        std::process::id()
    ));
    let dep_paths = format!("{}:{}", vaisdb_src.display(), std_link.display());
    let output = Command::new(env!("CARGO_BIN_EXE_vaisc"))
        .arg("build")
        .arg(&target)
        .arg("--emit-ir")
        .arg("-o")
        .arg(&out_ir)
        .arg("--force-rebuild")
        .env("VAIS_STD_PATH", &std_link)
        .env("VAIS_DEP_PATHS", dep_paths)
        .output()
        .expect("failed to spawn vaisc");
    let _ = std::fs::remove_file(&out_ir);
    Some(output)
}
