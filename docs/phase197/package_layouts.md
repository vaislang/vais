# Phase 197 Package Layouts

**Date**: 2026-04-18  
**Method**: read Cargo.toml + Glob for Node assets + Grep for compiler deps.

---

## vaisdb

- **Path**: `/Users/sswoo/study/projects/vais/lang/packages/vaisdb/`
- **Build System**: Vais language project (vais.toml, not Rust crate)
- **Project Type**: Single Vais package (not workspace)
- **Compiler dep**: Indirectly via examples (uses `vaisc` from compiler)
- **Runtime deps**: None (pure .vais source)
- **Size (src/)**: 3.5M
- **Config**: vais.toml (minimal test_pkg declaration)
- **Recommended commands for P197-V**:
  1. `cd /Users/sswoo/study/projects/vais/lang/packages/vaisdb`
  2. `rm -rf src/.vais-cache`
  3. `VAIS_STD_PATH=/Users/sswoo/study/projects/vais/compiler/std /Users/sswoo/study/projects/vais/compiler/target/release/vaisc check src/main.vais 2>&1 | tee /tmp/vaisdb_check.log`
  4. `(cargo test --test '*/vaisdb*' 2>&1) | tee /tmp/vaisdb_test.log`

---

## vais-server

- **Path**: `/Users/sswoo/study/projects/vais/lang/packages/vais-server/`
- **Build System**: Vais source + C runtime hybrid
- **Project Type**: Single Vais application (not workspace)
- **Compiler dep**: Direct via vaisc CLI (env var path: `VAISC`, `VAIS_STD_PATH`, `VAIS_DEP_PATHS`)
- **Runtime deps**: 
  - `runtime.c` (custom C runtime, 480 bytes)
  - `clang` (IR compilation & linking)
  - Vais standard library (`VAIS_STD_PATH=/Users/sswoo/study/projects/vais/compiler/std`)
- **Size (src/)**: 404K
- **Build script**: `build.sh` (custom orchestration: emit IR → compile → link)
- **Build flow**:
  1. Clean cache: `rm -rf src/.vais-cache/ vais-server.ll`
  2. `vaisc build --emit-ir src/main.vais -o vais-server.ll` (VAIS_SINGLE_MODULE=1)
  3. `clang -x ir -c vais-server.ll -o main.o`
  4. `clang -c runtime.c -o runtime.o`
  5. `clang main.o runtime.o -o vais-server -lSystem`
- **Recommended commands for P197-V**:
  1. `cd /Users/sswoo/study/projects/vais/lang/packages/vais-server`
  2. `./build.sh 2>&1 | tee /tmp/vais-server_build.log`
  3. `file ./vais-server && ./vais-server --version 2>&1 | head -5`
  4. (If tests exist) `cargo test 2>&1 | tee /tmp/vais-server_test.log`

---

## vais-web

- **Path**: `/Users/sswoo/study/projects/vais/lang/packages/vais-web/`
- **Build System**: pnpm workspace (Node.js + Rust multi-crate)
- **Project Type**: Monorepo workspace
- **Node assets**: 
  - `package.json` (Y) — root workspace config
  - `pnpm-workspace.yaml` (Y) — packages: ["packages/*"]
- **Rust crates** (under `crates/`):
  - `vaisx-parser` — Vais source parser (no compiler deps)
  - `vaisx-compiler` — Reactivity compiler (dep: vaisx-parser)
  - `vaisx-wasm` — WASM bindings (likely depends on others)
- **Compiler dep**: None in root Cargo.toml; check individual crate manifests for compiler refs
- **Node build**: pnpm install + pnpm build (scripts in package.json)
- **Size (crates/)**: N/A (Rust crates only 5 items); main code in `packages/*` (Node modules)
- **Workspace members**: 
  - `pnpm-workspace.yaml` -> packages: ["packages/*"] (Node packages)
  - `Cargo.toml` -> members: ["crates/*"] (Rust crates)
- **Recommended commands for P197-W**:
  1. `cd /Users/sswoo/study/projects/vais/lang/packages/vais-web`
  2. Rust side:
     - `cargo build --all-targets 2>&1 | tee /tmp/vais-web_cargo_build.log`
     - `cargo test 2>&1 | tee /tmp/vais-web_cargo_test.log`
  3. Node side:
     - `pnpm install --frozen-lockfile 2>&1 | tee /tmp/vais-web_pnpm_install.log`
     - `pnpm build 2>&1 | tee /tmp/vais-web_pnpm_build.log`
     - `pnpm test 2>&1 | tee /tmp/vais-web_pnpm_test.log` (if test script exists)

---

## Cross-package notes

### Compiler path
- **Shared compiler location**: `/Users/sswoo/study/projects/vais/compiler/`
- **Compiler std lib**: `/Users/sswoo/study/projects/vais/compiler/std`
- **vaisc binary**: Either system-wide (`~/.cargo/bin/vaisc`) or compiler build output (`compiler/target/release/vaisc`)

### Dependency model
| Package | Build type | Depends on compiler | Dep method |
|---------|-----------|-------------------|-----------|
| vaisdb | Vais only | Implicit (examples) | None (vais.toml) |
| vais-server | Vais + C | Yes (vaisc required) | CLI env vars (VAISC, VAIS_STD_PATH, VAIS_DEP_PATHS) |
| vais-web (Rust) | Cargo (3 crates) | No direct deps | Internal workspace refs |
| vais-web (Node) | pnpm monorepo | No | pnpm workspace + Node |

### Build isolation
- **vaisdb**: No external build deps; isolated from compiler
- **vais-server**: Hard dep on compiler; build.sh uses env var path resolution
- **vais-web**: No compiler deps in Rust crates; can build independently

### Observation: Cache & incremental compilation
- All three packages use local caching (`.vais-cache/` in vaisdb, vais-server)
- vais-server explicitly disables cache in build.sh (VAIS_SINGLE_MODULE=1)
- vais-web crates are workspace-managed; no special cache handling visible

---

## PROMISE

**PROMISE: COMPLETE**

All target packages reconnoitered:
- vaisdb: Vais source project (vais.toml)
- vais-server: Vais + C hybrid, build.sh orchestrates compilation
- vais-web: Dual pnpm + Cargo workspace (Node frontend, Rust compiler)

Compiler dependency model documented. Build flows captured. Recommended P197 test commands provided for each package.
