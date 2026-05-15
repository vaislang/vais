/// Integrity test matrix for the Vais compiler — Phase 0.2.
///
/// Implements the test harness defined in docs/COMPILER_STAGES.md §3 and §4.
/// Sub-modules provide stage-gate tests, syntax coverage, and ecosystem health
/// measurements. These tests COUNT failures rather than hard-blocking on 100%.
/// The baseline numbers produced here feed Phase 0.3.
///
/// Structure:
///   integrity.rs              — this file: harness helpers + module declarations
///   integrity/compiler_syntax.rs   — 30 syntax coverage tests
///   integrity/compiler_stages.rs   — stage gate tests (lex/parse/tc/codegen/run)
///   integrity/ecosystem_health.rs  — std/ and vaisdb/ measurement tests

mod integrity {
    pub mod compiler_stages;
    pub mod compiler_syntax;
    pub mod ecosystem_health;

    use std::path::{Path, PathBuf};
    use std::process::Command;

    // -----------------------------------------------------------------------
    // vaisc binary path
    // -----------------------------------------------------------------------

    /// Return the path to the freshly-built vaisc binary.
    ///
    /// Prefers the path injected by cargo's test harness
    /// (`CARGO_BIN_EXE_vaisc`). Falls back to the `VAISC` env var and then to
    /// `~/.cargo/bin/vaisc`. Using the cargo-injected path guarantees the binary
    /// matches the source under test (§5 "Installed vaisc drifts").
    pub fn vaisc_path() -> PathBuf {
        let built = env!("CARGO_BIN_EXE_vaisc");
        if !built.is_empty() {
            return PathBuf::from(built);
        }
        if let Ok(p) = std::env::var("VAISC") {
            return PathBuf::from(p);
        }
        if let Some(home) = std::env::var_os("HOME") {
            let mut p = PathBuf::from(home);
            p.push(".cargo/bin/vaisc");
            return p;
        }
        PathBuf::from("vaisc")
    }

    // -----------------------------------------------------------------------
    // Stdlib symlink helper (§4 "stdlib symlink" in COMPILER_STAGES.md)
    // -----------------------------------------------------------------------

    /// Ensure `/tmp/vais-lib/std` exists as a symlink to `<project>/std`.
    ///
    /// Called once per test binary. Idempotent — if the symlink already points
    /// at the right target it is left untouched.
    pub fn setup_std_symlink() {
        let project_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("failed to canonicalize project root");
        let std_src = project_root.join("std");
        let link_dir = PathBuf::from("/tmp/vais-lib");
        let link_target = link_dir.join("std");

        if !link_dir.exists() {
            std::fs::create_dir_all(&link_dir).unwrap_or_else(|e| {
                eprintln!("setup_std_symlink: mkdir /tmp/vais-lib failed: {}", e)
            });
        }

        if link_target.exists() {
            if let Ok(real) = link_target.canonicalize() {
                if real == std_src {
                    return; // already correct
                }
            }
            let _ = std::fs::remove_file(&link_target);
            let _ = std::fs::remove_dir(&link_target);
        }

        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;
            symlink(&std_src, &link_target).unwrap_or_else(|e| {
                eprintln!(
                    "setup_std_symlink: symlink {:?} -> {:?} failed: {}",
                    link_target, std_src, e
                )
            });
        }
        #[cfg(not(unix))]
        {
            eprintln!(
                "setup_std_symlink: symlinks unsupported on this platform; \
                 copy {:?} to {:?} manually",
                std_src, link_target
            );
        }
    }

    // -----------------------------------------------------------------------
    // §3 helpers — bash semantics translated to Rust
    // -----------------------------------------------------------------------

    /// Stage 2: Parse OK?
    ///
    /// Runs `vaisc check <path>` and returns `true` if exit code is 0.
    /// NOTE: `--show-ast` actually triggers full codegen in this compiler, so
    /// it cannot be used for a pure parse check. `check` stops at type-check,
    /// which is strictly stronger than parse-only but still skips codegen —
    /// close enough to "stage 2 OK" for integrity tests. A true parse-only
    /// gate is Phase 1.7 territory.
    ///
    /// Step 11 root fix (2026-05-08): pass VAIS_STD_PATH + VAIS_DEP_PATHS so
    /// `use std::io` and friends resolve under default-strict imports. Without
    /// these, the spawned vaisc inherits the test runner's cwd
    /// (`crates/vaisc/`) which has no `std/` subdirectory, and strict-default
    /// reports E_IMPORT_NOT_FOUND. Previously hidden by the silent fallback.
    pub fn ok_parse(path: &Path) -> bool {
        let vaisc = vaisc_path();
        let std_path = "/tmp/vais-lib/std";
        let output = Command::new(&vaisc)
            .arg("check")
            .arg(path)
            .env("VAIS_STD_PATH", std_path)
            .env("VAIS_DEP_PATHS", std_path)
            .output();
        match output {
            Err(e) => {
                eprintln!(
                    "ok_parse: failed to spawn {:?} for {}: {}",
                    vaisc,
                    path.display(),
                    e
                );
                false
            }
            Ok(out) => {
                if !out.status.success() {
                    let stderr = String::from_utf8_lossy(&out.stderr);
                    let snip: String = stderr.chars().take(300).collect();
                    eprintln!(
                        "ok_parse FAIL  stage=parse  file={}\n  stderr: {}",
                        path.display(),
                        snip
                    );
                }
                out.status.success()
            }
        }
    }

    /// Stage 3: Type-check OK?
    ///
    /// Runs `vaisc check <path>` and returns `true` on exit 0.
    ///
    /// Step 11 root fix (2026-05-08): same VAIS_STD_PATH / VAIS_DEP_PATHS
    /// injection as ok_parse — see that function's comment.
    pub fn ok_tc(path: &Path) -> bool {
        let vaisc = vaisc_path();
        let std_path = "/tmp/vais-lib/std";
        let output = Command::new(&vaisc)
            .arg("check")
            .arg(path)
            .env("VAIS_STD_PATH", std_path)
            .env("VAIS_DEP_PATHS", std_path)
            .output();
        match output {
            Err(e) => {
                eprintln!(
                    "ok_tc: failed to spawn {:?} for {}: {}",
                    vaisc,
                    path.display(),
                    e
                );
                false
            }
            Ok(out) => {
                if !out.status.success() {
                    let stderr = String::from_utf8_lossy(&out.stderr);
                    let snip: String = stderr.chars().take(300).collect();
                    eprintln!(
                        "ok_tc FAIL  stage=tc  file={}\n  stderr: {}",
                        path.display(),
                        snip
                    );
                }
                out.status.success()
            }
        }
    }

    /// Compute a unique per-input IR output path so parallel codegen
    /// tests don't race on `/tmp/__ok.ll`.
    ///
    /// **Background (Phase Ω P1.4 iter 114)**: prior to this commit,
    /// `ok_codegen` / `ok_codegen_pkg` both wrote to a single shared
    /// `/tmp/__ok.ll`. `cargo test` runs the integrity tests in parallel
    /// (multiple test threads inside this single test binary, plus
    /// possibly multiple test binaries). When two `vaisc build` invocations
    /// raced on that single `-o` target, one would clobber the other's
    /// IR mid-write, sometimes corrupting the file the second `vaisc`
    /// process was still reading back as part of its own pipeline (or
    /// crashing one of the two with a write-after-truncate). The result
    /// was a non-deterministic vaisdb pass count in the 217–223 range
    /// even on byte-identical source trees (documented in ROADMAP iter
    /// 109/113 retro). This unique-path scheme aligns codegen with the
    /// pattern `unique_exe_path` already uses for `ok_build`.
    fn unique_ir_path(input: &Path) -> PathBuf {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut h = DefaultHasher::new();
        input.to_string_lossy().hash(&mut h);
        std::process::id().hash(&mut h);
        // Include the test thread id so concurrent invocations within a
        // single test process also separate cleanly.
        format!("{:?}", std::thread::current().id()).hash(&mut h);
        PathBuf::from(format!("/tmp/__ok_ir_{:016x}.ll", h.finish()))
    }

    /// Stage 4: Codegen OK? (produces LLVM IR, does not link)
    ///
    /// Runs `vaisc build <path> --emit-ir -o <unique> --force-rebuild`.
    pub fn ok_codegen(path: &Path) -> bool {
        let vaisc = vaisc_path();
        // Phase 5.24: provide VAIS_STD_PATH + VAIS_DEP_PATHS so std files
        // that import other std modules (hashmap -> option/hash/stringmap)
        // can resolve. Without these the test reports spurious "outside
        // allowed directories" / "Cannot find Vais standard library"
        // failures even when the file builds OK interactively.
        let std_path = "/tmp/vais-lib/std";
        let ir_out = unique_ir_path(path);
        let output = Command::new(&vaisc)
            .arg("build")
            .arg(path)
            .arg("--emit-ir")
            .arg("-o")
            .arg(&ir_out)
            .arg("--force-rebuild")
            .env("VAIS_STD_PATH", std_path)
            .env("VAIS_DEP_PATHS", std_path)
            .output();
        // Best-effort cleanup so /tmp doesn't fill up over a full run.
        let _ = std::fs::remove_file(&ir_out);
        match output {
            Err(e) => {
                eprintln!(
                    "ok_codegen: failed to spawn {:?} for {}: {}",
                    vaisc,
                    path.display(),
                    e
                );
                false
            }
            Ok(out) => {
                if !out.status.success() {
                    let stderr = String::from_utf8_lossy(&out.stderr);
                    let snip: String = stderr.chars().take(300).collect();
                    eprintln!(
                        "ok_codegen FAIL  stage=codegen  file={}\n  stderr: {}",
                        path.display(),
                        snip
                    );
                }
                out.status.success()
            }
        }
    }

    /// Stage 4 (package variant): Codegen OK with package-level dep paths.
    ///
    /// `deps` — colon-separated dep path string.
    /// `stdroot` — path to std root.
    pub fn ok_codegen_pkg(path: &Path, deps: &str, stdroot: &str) -> bool {
        let vaisc = vaisc_path();
        let ir_out = unique_ir_path(path);
        let output = Command::new(&vaisc)
            .arg("build")
            .arg(path)
            .arg("--emit-ir")
            .arg("-o")
            .arg(&ir_out)
            .arg("--force-rebuild")
            .env("VAIS_DEP_PATHS", deps)
            .env("VAIS_STD_PATH", stdroot)
            .output();
        let _ = std::fs::remove_file(&ir_out);
        match output {
            Err(e) => {
                eprintln!(
                    "ok_codegen_pkg: failed to spawn {:?} for {}: {}",
                    vaisc,
                    path.display(),
                    e
                );
                false
            }
            Ok(out) => {
                if !out.status.success() {
                    let stderr = String::from_utf8_lossy(&out.stderr);
                    let snip: String = stderr.chars().take(300).collect();
                    eprintln!(
                        "ok_codegen_pkg FAIL  stage=codegen  file={}\n  stderr: {}",
                        path.display(),
                        snip
                    );
                }
                out.status.success()
            }
        }
    }

    /// Stage 5: Full build OK? (link + executable present)
    ///
    /// Runs `vaisc build <path> -o /tmp/__ok_exe --force-rebuild` then
    /// verifies the executable exists and is executable.
    /// Compute a unique per-input exe output path so parallel tests don't
    /// clobber each other. Uses the canonical input path's DefaultHasher digest.
    fn unique_exe_path(input: &Path) -> PathBuf {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut h = DefaultHasher::new();
        input.to_string_lossy().hash(&mut h);
        std::process::id().hash(&mut h);
        PathBuf::from(format!("/tmp/__ok_exe_{:016x}", h.finish()))
    }

    pub fn ok_build(path: &Path) -> bool {
        let vaisc = vaisc_path();
        let exe_out = unique_exe_path(path);
        let output = Command::new(&vaisc)
            .arg("build")
            .arg(path)
            .arg("-o")
            .arg(&exe_out)
            .arg("--force-rebuild")
            .output();
        match output {
            Err(e) => {
                eprintln!(
                    "ok_build: failed to spawn {:?} for {}: {}",
                    vaisc,
                    path.display(),
                    e
                );
                return false;
            }
            Ok(out) => {
                if !out.status.success() {
                    let stderr = String::from_utf8_lossy(&out.stderr);
                    let snip: String = stderr.chars().take(300).collect();
                    eprintln!(
                        "ok_build FAIL  stage=link  file={}\n  stderr: {}",
                        path.display(),
                        snip
                    );
                    return false;
                }
            }
        }
        let exe_path = exe_out.clone();
        if !exe_path.exists() {
            eprintln!(
                "ok_build FAIL  stage=link  file={}  reason=executable-not-found",
                path.display()
            );
            return false;
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            match std::fs::metadata(&exe_path) {
                Ok(meta) => {
                    let mode = meta.permissions().mode();
                    if mode & 0o111 == 0 {
                        eprintln!(
                            "ok_build FAIL  stage=link  file={}  reason=not-executable (mode={:o})",
                            path.display(),
                            mode
                        );
                        return false;
                    }
                }
                Err(e) => {
                    eprintln!(
                        "ok_build FAIL  stage=link  file={}  reason=stat-error: {}",
                        path.display(),
                        e
                    );
                    return false;
                }
            }
        }
        true
    }

    /// Stage 6: Run OK with expected exit code?
    ///
    /// Calls `ok_build` first; if that fails returns `false`.
    /// Then executes `/tmp/__ok_exe` and checks the exit code.
    pub fn ok_run(path: &Path, expected_exit: i32) -> bool {
        if !ok_build(path) {
            eprintln!(
                "ok_run FAIL  stage=build  file={}  reason=build-failed",
                path.display()
            );
            return false;
        }
        let exe_path = unique_exe_path(path);
        let run_result = Command::new(&exe_path).output();
        match run_result {
            Err(e) => {
                eprintln!(
                    "ok_run FAIL  stage=run  file={}  reason=exec-error: {}",
                    path.display(),
                    e
                );
                false
            }
            Ok(out) => {
                let actual = out.status.code().unwrap_or(-1);
                if actual != expected_exit {
                    let stderr = String::from_utf8_lossy(&out.stderr);
                    eprintln!(
                        "ok_run FAIL  stage=run  file={}  expected_exit={}  actual_exit={}\n  stderr: {}",
                        path.display(),
                        expected_exit,
                        actual,
                        stderr.chars().take(200).collect::<String>()
                    );
                    return false;
                }
                true
            }
        }
    }
}
