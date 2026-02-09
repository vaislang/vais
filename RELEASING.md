# Releasing Vais

This document outlines the complete release process for Vais, from version bumping through post-release verification.

## Table of Contents

1. [Version Scheme](#version-scheme)
2. [Pre-Release Checklist](#pre-release-checklist)
3. [Release Execution](#release-execution)
4. [Automated Workflows](#automated-workflows)
5. [Post-Release Verification](#post-release-verification)
6. [Rollback Procedures](#rollback-procedures)
7. [Troubleshooting](#troubleshooting)

## Version Scheme

Vais follows [Semantic Versioning 2.0.0](https://semver.org/):

- **MAJOR.MINOR.PATCH** (e.g., `1.2.3`)
  - **MAJOR**: Breaking changes to language syntax, stdlib APIs, or compiler CLI
  - **MINOR**: Backward-compatible feature additions
  - **PATCH**: Backward-compatible bug fixes

### Pre-Release Versions

Use pre-release identifiers for unstable versions:

- **Alpha**: `v1.1.0-alpha.1` - Early testing, API unstable
- **Beta**: `v1.1.0-beta.1` - Feature-complete, testing for bugs
- **Release Candidate**: `v1.1.0-rc.1` - Final testing before stable release

Pre-release versions are automatically marked as "pre-release" on GitHub Releases.

## Pre-Release Checklist

### 1. Version Bump

Update the version in all workspace crates:

```bash
# Update workspace version in Cargo.toml
vim Cargo.toml  # Set workspace.package.version = "1.1.0"

# Verify all crates use workspace version
grep -r 'version.workspace = true' crates/*/Cargo.toml
```

**Crates to update:**
- All 33 workspace members in `crates/` directory
- Each crate should use `version.workspace = true` in their `Cargo.toml`
- Main workspace version is in root `Cargo.toml` under `[workspace.package]`

### 2. Update CHANGELOG.md

Follow [Keep a Changelog](https://keepachangelog.com/) format:

```markdown
## [1.1.0] - 2026-02-15

### Added
- New feature X with description
- Another feature Y

### Changed
- Modified behavior of Z

### Fixed
- Bug fix for issue #123

### Deprecated
- Old API will be removed in v2.0.0

### Removed
- Removed deprecated feature from v1.0.0

### Security
- Fixed CVE-XXXX-YYYY
```

Move items from `## [Unreleased]` section to the new version section.

### 3. Run Full Test Suite

Ensure all tests pass and code quality checks succeed:

```bash
# 1. Type check
cargo check

# 2. Run all tests (2,500+ tests across workspace)
cargo test --workspace

# 3. Lint with Clippy (must be 0 warnings)
cargo clippy --workspace --exclude vais-python --exclude vais-node -- -D warnings

# 4. Check formatting
cargo fmt --check

# 5. Run E2E tests specifically
cargo test --test e2e_tests
cargo test --test integration_tests

# 6. Run benchmarks to verify performance (optional but recommended)
cargo bench --no-run
```

**Expected Results:**
- All tests pass: 2,500+ tests
- Clippy warnings: 0
- E2E tests: 467+ passing

### 4. Test Example Programs

Verify that key examples compile and run:

```bash
# Test core examples
cargo run --bin vaisc -- examples/hello.vais
cargo run --bin vaisc -- examples/functions.vais
cargo run --bin vaisc -- examples/structs.vais
cargo run --bin vaisc -- examples/traits.vais

# Test JavaScript target
cargo run --bin vaisc -- --target js examples/hello.vais

# Test WASM target
cargo run --bin vaisc -- --target wasm32-unknown-unknown examples/hello.vais
```

### 5. Update Documentation

Ensure documentation is current:

```bash
# Build documentation site
cd docs-site
mdbook build

# Generate API docs
cargo doc --workspace --no-deps --exclude vais-python --exclude vais-node
```

### 6. Security Audit

Run security checks:

```bash
# Check for known vulnerabilities
cargo audit

# Run supply chain analysis
cargo run --bin vaisc -- supply-chain audit
```

### 7. Final Commit

Create a version bump commit:

```bash
git add Cargo.toml Cargo.lock CHANGELOG.md
git commit -m "chore: bump version to v1.1.0

- Update workspace version to 1.1.0
- Update CHANGELOG.md with release notes
- All 467+ E2E tests passing
- Clippy clean (0 warnings)

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>"

git push origin main
```

## Release Execution

### 1. Create and Push Git Tag

```bash
# Create annotated tag
git tag -a v1.1.0 -m "Release v1.1.0

- Feature X: Description
- Feature Y: Description
- Bug fix for issue #123
"

# Push tag to trigger release workflows
git push origin v1.1.0
```

**This single command triggers all automated workflows:**
- GitHub Release with platform binaries
- Package publishing (crates.io + Vais registry)
- Homebrew formula update

### 2. Monitor Workflow Execution

Visit https://github.com/vaislang/vais/actions and monitor:

1. **Release Workflow** (`release.yml`)
   - Builds 4 platform binaries in parallel
   - Creates GitHub Release with archives
   - Status: ~15-20 minutes

2. **Publish Package Workflow** (`publish.yml`)
   - Validates version matches tag
   - Builds and tests package
   - Publishes to registries
   - Status: ~10 minutes

3. **Homebrew Update Workflow** (`homebrew.yml`)
   - Downloads release archives
   - Computes SHA256 checksums
   - Updates Formula/vais.rb
   - Status: ~5 minutes

## Automated Workflows

### Release Workflow (release.yml)

**Trigger:** Push tag matching `v*`

**Platforms Built:**
1. `x86_64-unknown-linux-gnu` (Ubuntu latest)
2. `x86_64-apple-darwin` (macOS latest, Intel)
3. `aarch64-apple-darwin` (macOS latest, Apple Silicon)
4. `x86_64-pc-windows-msvc` (Windows latest)

**Artifacts Created:**
- `vais-v1.1.0-x86_64-unknown-linux-gnu.tar.gz`
- `vais-v1.1.0-x86_64-apple-darwin.tar.gz`
- `vais-v1.1.0-aarch64-apple-darwin.tar.gz`
- `vais-v1.1.0-x86_64-pc-windows-msvc.zip`
- `sha256sums.txt` (checksums for all archives)

**Archive Contents:**
```
vais/
├── vaisc              # Compiler binary
├── std/               # Standard library (78 .vais files)
├── LICENSE
└── README.md
```

**GitHub Release:**
- Title: `Vais Compiler v1.1.0`
- Body: Auto-generated from CHANGELOG.md + installation instructions
- Assets: 4 platform archives + SHA256 checksums
- Pre-release flag: Automatically set for alpha/beta/rc tags

### Publish Package Workflow (publish.yml)

**Trigger:** Push tag matching `v*` or manual workflow dispatch

**Steps:**
1. **Validate:** Extract version, check `vais.toml` exists and matches tag
2. **Build:** Compile release binary, run tests, package tarball
3. **Publish:** Upload to Vais registry (when available)

**Note:** Currently the registry publish step is commented out until the registry server is deployed. The workflow will still run and create artifacts.

### Homebrew Update Workflow (homebrew.yml)

**Trigger:** GitHub Release published event

**Steps:**
1. Checkout `vaislang/homebrew-tap` repository
2. Download x86_64 and aarch64 Darwin archives
3. Compute SHA256 checksums
4. Update `Formula/vais.rb` with new version and checksums
5. Commit and push to homebrew-tap repo

**Homebrew Formula Structure:**
```ruby
class Vais < Formula
  desc "AI-optimized systems programming language with single-character keywords"
  homepage "https://github.com/vaislang/vais"
  version "1.1.0"
  license "MIT"

  if Hardware::CPU.intel?
    url "https://github.com/vaislang/vais/releases/download/v1.1.0/..."
    sha256 "..."
  elsif Hardware::CPU.arm?
    url "https://github.com/vaislang/vais/releases/download/v1.1.0/..."
    sha256 "..."
  end

  def install
    bin.install "vaisc"
    prefix.install "std"
    doc.install "README.md" if File.exist?("README.md")
  end

  test do
    assert_match "vais", shell_output("#{bin}/vaisc --version")
  end
end
```

## Post-Release Verification

### 1. GitHub Release Page

Visit: https://github.com/vaislang/vais/releases/tag/v1.1.0

**Verify:**
- [ ] Release title is correct: `Vais Compiler v1.1.0`
- [ ] 4 platform archives are attached:
  - [ ] `vais-v1.1.0-x86_64-unknown-linux-gnu.tar.gz`
  - [ ] `vais-v1.1.0-x86_64-apple-darwin.tar.gz`
  - [ ] `vais-v1.1.0-aarch64-apple-darwin.tar.gz`
  - [ ] `vais-v1.1.0-x86_64-pc-windows-msvc.zip`
- [ ] `sha256sums.txt` is attached
- [ ] Release notes are populated (auto-generated or manual)
- [ ] Pre-release checkbox matches version type
- [ ] Installation instructions are correct

### 2. SHA256 Checksums

Verify archive integrity:

```bash
# Download sha256sums.txt from release page
wget https://github.com/vaislang/vais/releases/download/v1.1.0/sha256sums.txt

# Download archives
wget https://github.com/vaislang/vais/releases/download/v1.1.0/vais-v1.1.0-x86_64-unknown-linux-gnu.tar.gz

# Verify checksums
sha256sum -c sha256sums.txt
```

**Expected Output:**
```
vais-v1.1.0-x86_64-unknown-linux-gnu.tar.gz: OK
vais-v1.1.0-x86_64-apple-darwin.tar.gz: OK
vais-v1.1.0-aarch64-apple-darwin.tar.gz: OK
vais-v1.1.0-x86_64-pc-windows-msvc.zip: OK
```

### 3. Binary Installation Test

Test each platform's binary (if available):

**Linux/macOS:**
```bash
# Extract archive
tar -xzf vais-v1.1.0-x86_64-unknown-linux-gnu.tar.gz
cd vais

# Test version
./vaisc --version
# Expected: vais 1.1.0

# Test compilation
echo 'F main() { println("Hello!") }' > test.vais
./vaisc test.vais
./test
# Expected: Hello!
```

**Windows:**
```powershell
# Extract archive
Expand-Archive vais-v1.1.0-x86_64-pc-windows-msvc.zip
cd vais-v1.1.0-x86_64-pc-windows-msvc\vais

# Test version
.\vaisc.exe --version

# Test compilation
Set-Content test.vais 'F main() { println("Hello!") }'
.\vaisc.exe test.vais
.\test.exe
```

### 4. Cargo Install (When Published to crates.io)

```bash
# Install from crates.io
cargo install vaisc --version 1.1.0

# Verify installation
vaisc --version

# Test with example
echo 'F main() { println("Cargo install works!") }' > test.vais
vaisc test.vais
./test
```

**Note:** Currently the project is not published to crates.io. This step will be relevant once crates.io publishing is enabled.

### 5. Homebrew Installation (macOS)

```bash
# Update tap
brew update

# Install latest version
brew install vaislang/tap/vais

# Verify version
vaisc --version
# Expected: vais 1.1.0

# Test compilation
echo 'F main() { println("Homebrew works!") }' > test.vais
vaisc test.vais
./test
# Expected: Homebrew works!

# Verify standard library is accessible
ls $(brew --prefix vais)/std/
# Expected: core.vais, io.vais, collections.vais, etc.
```

### 6. Docker Image (When Available)

If Docker images are built in the future:

```bash
# Pull image
docker pull vaislang/vais:1.1.0

# Test version
docker run --rm vaislang/vais:1.1.0 --version

# Compile example
docker run --rm -v $(pwd):/workspace vaislang/vais:1.1.0 /workspace/hello.vais
```

### 7. Smoke Test Standard Library

Verify standard library works with new release:

```bash
# Test collections
cat > test_collections.vais << 'EOF'
U std::collections::Vec

F main() {
    v := Vec::new()
    v.push(1)
    v.push(2)
    v.push(3)
    println(v.len())  # Should print 3
}
EOF

vaisc test_collections.vais
./test_collections

# Test I/O
cat > test_io.vais << 'EOF'
U std::io::File

F main() {
    F write_file() -> Result<(), str> {
        file := File::create("test.txt")?
        file.write_all("Hello from Vais!")?
        R Ok(())
    }
    M write_file() {
        Ok(_) => println("File written successfully"),
        Err(e) => println("Error: ~{e}"),
    }
}
EOF

vaisc test_io.vais
./test_io

# Test async runtime (if applicable)
cat > test_async.vais << 'EOF'
U std::async::spawn

A F async_task() -> i32 {
    R 42
}

F main() {
    task := spawn(async_task())
    result := task.await
    println("Result: ~{result}")
}
EOF

vaisc test_async.vais
./test_async
```

## Rollback Procedures

If critical issues are discovered after release, follow these steps:

### 1. Delete Git Tag

```bash
# Delete local tag
git tag -d v1.1.0

# Delete remote tag
git push origin :refs/tags/v1.1.0
```

### 2. Delete GitHub Release

```bash
# Using GitHub CLI
gh release delete v1.1.0 --yes

# Or manually:
# 1. Visit https://github.com/vaislang/vais/releases/tag/v1.1.0
# 2. Click "Delete" button
# 3. Confirm deletion
```

### 3. Yank from crates.io (When Published)

```bash
# Yank version from crates.io
cargo yank --version 1.1.0 vaisc

# Note: This doesn't delete the version, but prevents new users from downloading it
# Existing Cargo.lock files with this version will still work
```

### 4. Revert Homebrew Formula

```bash
# Clone homebrew-tap
git clone https://github.com/vaislang/homebrew-tap
cd homebrew-tap

# Revert the formula update commit
git revert HEAD
git push origin main
```

### 5. Publish Hotfix Release

If a critical bug is found:

1. Create a hotfix branch from the tag
2. Fix the bug
3. Bump to patch version (e.g., `v1.1.0` → `v1.1.1`)
4. Follow normal release process
5. Announce hotfix and recommend users upgrade

```bash
# Create hotfix branch
git checkout -b hotfix/v1.1.1 v1.1.0

# Fix bug
vim crates/vaisc/src/bug.rs
cargo test

# Commit and tag
git commit -m "fix: critical bug in X"
git tag -a v1.1.1 -m "Hotfix v1.1.1 - Fix critical bug in X"
git push origin v1.1.1
```

## Troubleshooting

### Build Failures

**Problem:** Release workflow fails to build for a specific platform

**Solution:**
1. Check GitHub Actions logs for error details
2. Reproduce locally with cross-compilation:
   ```bash
   # Install target
   rustup target add x86_64-unknown-linux-gnu

   # Build for target
   cargo build --release --target x86_64-unknown-linux-gnu -p vaisc
   ```
3. Fix issues and push new tag with patch version

### LLVM Version Mismatch

**Problem:** Build fails with LLVM linking errors

**Solution:**
- Ensure all platforms install LLVM 17 correctly
- Check `LLVM_SYS_170_PREFIX` environment variable in workflow
- Update LLVM installation steps in `.github/workflows/release.yml`

### Homebrew Formula Fails

**Problem:** `homebrew.yml` workflow fails to update formula

**Solution:**
1. Check `HOMEBREW_TAP_TOKEN` secret is valid
2. Verify `vaislang/homebrew-tap` repository exists and is accessible
3. Manually update formula if needed:
   ```bash
   git clone https://github.com/vaislang/homebrew-tap
   cd homebrew-tap
   # Edit Formula/vais.rb
   git commit -m "Manual update for v1.1.0"
   git push
   ```

### SHA256 Mismatch

**Problem:** Users report SHA256 checksum failures

**Solution:**
1. Re-download archives from GitHub Release page
2. Recalculate SHA256:
   ```bash
   sha256sum vais-v1.1.0-*.tar.gz vais-v1.1.0-*.zip
   ```
3. Update `sha256sums.txt` in the release manually
4. Investigate why CI generated incorrect checksums

### Cargo.toml Version Mismatch

**Problem:** `publish.yml` fails with version mismatch error

**Solution:**
1. Verify `Cargo.toml` workspace version matches tag:
   ```bash
   grep 'version = ' Cargo.toml | head -1
   ```
2. Update version and create new tag:
   ```bash
   vim Cargo.toml  # Fix version
   git commit -am "fix: correct version to 1.1.0"
   git push
   git tag -d v1.1.0
   git push origin :refs/tags/v1.1.0
   git tag -a v1.1.0 -m "Release v1.1.0"
   git push origin v1.1.0
   ```

### Test Failures in CI

**Problem:** Tests pass locally but fail in release workflow

**Solution:**
1. Check for environment-specific issues (file paths, permissions)
2. Run tests in release mode locally:
   ```bash
   cargo test --release
   ```
3. Check for race conditions or timing issues in tests
4. Add graceful skip patterns for tests requiring external dependencies (e.g., clang):
   ```rust
   if !has_clang() {
       println!("Skipping test: clang not found");
       return;
   }
   ```

## Release Communication

After successful release verification:

### 1. Announce on GitHub Discussions

Create announcement post with:
- Version number and release date
- Major features and changes
- Upgrade instructions
- Link to CHANGELOG.md
- Known issues (if any)

### 2. Update Website

If project website exists:
- Update version number in downloads page
- Publish blog post about new features
- Update documentation to reflect new APIs

### 3. Social Media (Optional)

Share release announcement on:
- Twitter/X
- Reddit (r/rust, r/programming)
- Hacker News
- Discord/Slack communities

## Release Checklist Summary

Use this quick checklist for each release:

**Pre-Release:**
- [ ] Bump version in `Cargo.toml`
- [ ] Update `CHANGELOG.md`
- [ ] Run `cargo test --workspace` (all pass)
- [ ] Run `cargo clippy` (0 warnings)
- [ ] Test example programs
- [ ] Commit version bump
- [ ] Push to main branch

**Release:**
- [ ] Create and push tag: `git tag -a vX.Y.Z && git push origin vX.Y.Z`
- [ ] Monitor `release.yml` workflow (4 platform builds)
- [ ] Monitor `publish.yml` workflow (package publishing)
- [ ] Monitor `homebrew.yml` workflow (formula update)

**Post-Release:**
- [ ] Verify GitHub Release has 4 archives + SHA256
- [ ] Download and verify SHA256 checksums
- [ ] Test binary installation on at least one platform
- [ ] Test `brew install vaislang/tap/vais` (macOS)
- [ ] Test `cargo install vaisc` (when available)
- [ ] Smoke test standard library examples
- [ ] Announce release on GitHub Discussions
- [ ] Update website/documentation

**If Issues Found:**
- [ ] Delete git tag (local + remote)
- [ ] Delete GitHub Release
- [ ] Yank from crates.io (if published)
- [ ] Revert Homebrew formula
- [ ] Prepare hotfix release

---

## Additional Resources

- [Semantic Versioning](https://semver.org/)
- [Keep a Changelog](https://keepachangelog.com/)
- [GitHub Releases Documentation](https://docs.github.com/en/repositories/releasing-projects-on-github)
- [Cargo Publishing Documentation](https://doc.rust-lang.org/cargo/reference/publishing.html)
- [Homebrew Formula Cookbook](https://docs.brew.sh/Formula-Cookbook)

For questions or issues with the release process, open an issue at https://github.com/vaislang/vais/issues
