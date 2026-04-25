# Hello World — Vais Language Examples

These five programs demonstrate the **stable subset** of Vais language
features as of Phase 0 kickoff.

| File | Demonstrates | Expected exit |
|------|--------------|---------------|
| `01_hello.vais` | minimum viable program | 0 |
| `02_arithmetic.vais` | int arithmetic | 5 |
| `03_struct.vais` | struct + method via `X` impl block | 7 |
| `04_option.vais` | `Option<T>` + match expression | 3 |
| `05_recursion.vais` | recursive fibonacci | 21 |

## Build & run

```bash
# Each example builds standalone — no stdlib required.
cd /Users/sswoo/study/projects/vais/compiler/examples/hello_world_v2

for f in 0*.vais; do
    base=$(basename "$f" .vais)
    ~/.cargo/bin/vaisc build "$f" --emit-ir -o "/tmp/${base}.ll" --force-rebuild
    clang -O0 -o "/tmp/${base}" /tmp/${base}*.ll \
        ../../selfhost/runtime.c ../../std/sync_runtime.c -lm
    "/tmp/${base}"
    echo "$base exit=$?"
done
```

Or use `Makefile`:
```bash
make check
```

## Phase 0 plan

These 5 are the seed of the eventual 12-example Phase 0.D suite. Adding
file I/O, concurrency, traits, and FFI demos requires those features to
graduate from "experimental" to "core". See
[../docs/PHASE_0_LANGUAGE_STABILIZATION.md](../docs/PHASE_0_LANGUAGE_STABILIZATION.md).
