# Vais Agent Notes

## Project Boundary

- This repo is the Vais mainline at `/Users/sswoo/study/projects/vais`.
- Checked-in language sources use `.vais`.
- Public commands are `scripts/vaisc` and `tools/vais-check.py`.
- Do not add alternate source extensions.

## Compiler Paths

- `scripts/vaisc` is the product-facing compiler CLI.
- `tools/vaisc.py` implements `emit-ir`, `build`, and `run`.
- `compiler/self/fixpoint_full.vais` is the trusted full self-host compiler source.
- `compiler/self/vaisc_core.ll` is the reusable self-host compiler core used by `scripts/vaisc`.
- `scripts/build.sh` and `scripts/vais-build-env.sh` are internal core-refresh tools.

## Gates

Use the smallest gate that covers the change, then broaden when touching shared compiler behavior.

```bash
python3 -m py_compile tools/vaisc.py tools/vais-check.py tools/embed_self_source.py tests/vais_check_test.py
bash -n scripts/*.sh
python3 tests/vais_check_test.py
bash scripts/test-vaisc.sh
bash scripts/test-vaisc-front.sh
bash scripts/test-vaisc-direct.sh
bash scripts/test-vaisc-errors.sh
bash scripts/test-vaisc-parity.sh
bash scripts/test.sh
```

For compiler-core regeneration work, also run:

```bash
bash scripts/test-fixpoint-full.sh
bash scripts/test-fixpoint-full-self.sh
```

## Current Priority

Keep the self-host core and release gates green as the compiler surface expands.
