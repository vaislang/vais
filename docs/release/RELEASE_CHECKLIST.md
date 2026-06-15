# Vais Release Checklist

This checklist is the pre-tag release contract for the Vais mainline.

## Current Release Line

- Current source release: `v0.2.1`
- Next planned source release: `v0.2.2`
- Stable target: `v1.0.0` after the v1 completion roadmap is gate-backed

Use `vMAJOR.MINOR.PATCH` source tags. Do not move a public tag after release
archives have been published.

## Pre-Tag Requirements

1. Start from `main` with a clean worktree.
2. Confirm `CHANGELOG.md` has a dated section for the tag being cut.
3. Confirm `README.md`, `docs/README.md`, `docs/reference/LANGUAGE.md`,
   `std/PRELUDE.md`, `compiler/self/SELF_HOST.md`, and `website/` describe only
   the gate-backed release surface.
4. Run the release gate:

   ```bash
   bash scripts/test-release-gates.sh
   ```

5. Confirm the generated standalone archive exists:

   ```bash
   ls dist/vais-*.tar.gz
   ```

6. Create the annotated release tag only after the gate is green:

   ```bash
   git tag -a v0.2.2 -m "Vais v0.2.2"
   git push origin v0.2.2
   ```

## GitHub Release Archives

Source tags matching `v*` trigger `.github/workflows/release-archives.yml`.
The workflow builds standalone archives on Linux x64, macOS arm64, and macOS
x64, smoke-tests the packaged `vaisc`, creates the matching GitHub Release when
needed, and uploads the archives.

To run the archive workflow manually for an existing tag:

```bash
gh workflow run release-archives.yml -f tag=v0.2.2
```

Only run the manual workflow for a tag that already points to the intended
release commit.

## Post-Tag Verification

1. Watch the release archive workflow:

   ```bash
   gh run list --workflow "Release Archives" --limit 5
   gh run watch <run-id> --exit-status
   ```

2. Confirm the GitHub Release exists and has uploaded archives:

   ```bash
   gh release view v0.2.2
   ```

3. Confirm the website deploy for the same commit succeeds:

   ```bash
   gh run list --workflow "Deploy Website" --branch main --limit 5
   ```

4. Verify the live site still describes the released compiler path:

   ```bash
   curl -fsSL https://vaislang.dev/ | grep -F "scripts/vaisc"
   ```

## Stop Conditions

Do not tag if any release gate fails, if `CHANGELOG.md` still has release notes
only under `Unreleased`, if the working tree is dirty, or if public docs mention
ungated language features as current release claims.
