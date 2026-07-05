# CLAUDE.md

## Project Rules

- Treat `scripts/vaisc` and `scripts/vais-check` as the public commands.
- Keep checked-in language sources on the `.vais` extension.
- Preserve the full self-host path through `compiler/self/fixpoint_full.vais`
  and the reusable core at `compiler/self/vaisc_core.ll`.
- Promote new language surface only with matching full, direct, front, parity,
  documentation, and example coverage.
- Do not revert unrelated dirty worktree changes.

## Validation

Use the smallest relevant gate first, then broaden when touching compiler or
runtime behavior.

```bash
bash scripts/test-vaisc-front.sh
bash scripts/test-vaisc-direct.sh
bash scripts/test-fixpoint-full.sh
bash scripts/test.sh
bash scripts/test-vaisc-parity.sh
bash scripts/test-fixpoint-full-self.sh
git diff --check
bash scripts/test-release-gates.sh
```

## Repository Notes

- Main native driver: `tools/vaisc_native.c`
- Full self-host compiler source: `compiler/self/fixpoint_full.vais`
- Reusable self-host core: `compiler/self/vaisc_core.ll`
- Public language reference: `docs/reference/LANGUAGE.md`
- Prelude/API status: `std/PRELUDE.md`
- Example corpus: `examples/`
