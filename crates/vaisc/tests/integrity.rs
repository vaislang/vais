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
    pub mod compiler_syntax;
    pub mod compiler_stages;
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
    /// Runs `vaisc --show-ast <path>` and returns `true` if exit code is 0.
    pub fn ok_parse(path: &Path) -> bool {
        let vaisc = vaisc_path();
        let output = Command::new(&vaisc).arg("--show-ast").arg(path).output();
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
    pub fn ok_tc(path: &Path) -> bool {
        let vaisc = vaisc_path();
        let output = Command::new(&vaisc).arg("check").arg(path).output();
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

    /// Stage 4: Codegen OK? (produces LLVM IR, does not link)
    ///
    /// Runs `vaisc build <path> --emit-ir -o /tmp/__ok.ll --force-rebuild`.
    pub fn ok_codegen(path: &Path) -> bool {
        let vaisc = vaisc_path();
        let output = Command::new(&vaisc)
            .arg("build")
            .arg(path)
            .arg("--emit-ir")
            .arg("-o")
            .arg("/tmp/__ok.ll")
            .arg("--force-rebuild")
            .output();
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
        let output = Command::new(&vaisc)
            .arg("build")
            .arg(path)
            .arg("--emit-ir")
            .arg("-o")
            .arg("/tmp/__ok.ll")
            .arg("--force-rebuild")
            .env("VAIS_DEP_PATHS", deps)
            .env("VAIS_STD_PATH", stdroot)
            .output();
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
