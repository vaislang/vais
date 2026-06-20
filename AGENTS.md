# Vais Agent Notes

## Project Boundary

- This repo is the Vais mainline at `/Users/sswoo/study/projects/vais`.
- Checked-in language sources use `.vais`.
- Public commands are `scripts/vaisc` and `scripts/vais-check`.
- Do not add alternate source extensions.

## Compiler Paths

- `scripts/vaisc` is the product-facing compiler CLI.
- `tools/vaisc_native.c` implements the native public driver for `emit-ir`,
  `build`, `run`, `doctor`, `--version`, and the native direct engine.
- `scripts/vais-build-env.sh` builds trusted self-host helper sources through
  the native `scripts/vaisc` trust-root path.
- Retired checker helper prototypes are not public commands; public checker use
  goes through `scripts/vais-check`.
- `tools/embed_self_source.vais` is the Vais-native self-source embedding
  helper used by self-host gates for source-file retargeting and raw compact
  program/call embedding.
- `tools/normalize_stage_ir.vais` is the Vais-native stage IR normalizer used
  by self-host stage-comparison gates.
- `compiler/self/fixpoint_full.vais` is the trusted full self-host compiler source.
- `compiler/self/vaisc_core.ll` is the reusable self-host compiler core used by `scripts/vaisc`.
- `scripts/build-vaisc-native.sh` builds the native public driver.
- `scripts/install-vaisc.sh`, `scripts/uninstall-vaisc.sh`, and
  `scripts/package-vaisc-release.sh` manage standalone native compiler/checker
  installs and release archives.
- `.github/workflows/release-archives.yml` publishes standalone archives for
  source tags.
- `scripts/build.sh` and `scripts/vais-build-env.sh` are internal core-refresh tools.

## Documentation Paths

- `README.md` and `docs/README.md` are the first-read documentation entry points.
- `docs/reference/LANGUAGE.md` is the current gate-backed language guide.
- `compiler/self/SELF_HOST.md` is the compiler-internals guide.
- Public GitHub or site copy must be synced from these files, not from older external docs.

## Gates

Use the smallest gate that covers the change, then broaden when touching shared compiler behavior.

```bash
bash -n scripts/*.sh
bash scripts/test-vais-check-vais.sh
bash scripts/test-vaisc-native.sh
bash scripts/test-vaisc-install.sh
bash scripts/test-vaisc.sh
bash scripts/test-vaisc-front.sh
bash scripts/test-vaisc-direct.sh
bash scripts/test-vaisc-errors.sh
bash scripts/test-vaisc-parity.sh
bash scripts/test-vaisc-host.sh
bash scripts/test-embed-self-source-vais.sh
bash scripts/test-normalize-stage-ir-vais.sh
bash scripts/test-fixpoint.sh
bash scripts/test-fixpoint2.sh
bash scripts/test.sh
```

For compiler-core regeneration work, also run:

```bash
bash scripts/test-fixpoint-full.sh
bash scripts/test-fixpoint-full-self.sh
```

## Current Priority

Sync GitHub/site-facing docs from the canonical docs, then keep the self-host core and release gates green as the compiler surface expands.
