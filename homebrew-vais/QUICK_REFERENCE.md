# Quick Reference - Homebrew Vais Tap

## For Users

### Installation
```bash
brew tap vaislang/tap
brew install vais
```

### Usage
```bash
vaisc --help              # Show help
vaisc --version           # Show version
vaisc repl                # Start REPL
vaisc build file.vais     # Compile a file
vaisc run file.vais       # Run a file
vaisc check file.vais     # Check syntax
```

### Update
```bash
brew update
brew upgrade vais
```

### Uninstall
```bash
brew uninstall vais
brew untap vaislang/vais
```

## For Maintainers

### Release New Version

1. **Tag release in main repo**
   ```bash
   cd /Users/sswoo/study/projects/vais
   git tag -a v0.2.0 -m "Release v0.2.0"
   git push origin v0.2.0
   ```

2. **Calculate SHA256**
   ```bash
   VERSION=0.2.0
   curl -L https://github.com/vaislang/vais/archive/refs/tags/v${VERSION}.tar.gz | shasum -a 256
   ```

3. **Update formula**
   ```bash
   cd homebrew-vais
   # Edit Formula/vais.rb:
   # - Update url to new version
   # - Update sha256 with calculated hash
   ```

4. **Test locally**
   ```bash
   brew install --build-from-source ./Formula/vais.rb
   brew test vais
   brew audit --strict vais
   brew uninstall vais
   ```

5. **Commit and push**
   ```bash
   git add Formula/vais.rb
   git commit -m "Update to v0.2.0"
   git push
   ```

### Testing Commands

```bash
# Install from local formula
brew install --build-from-source ./Formula/vais.rb

# Run formula tests
brew test vais

# Audit formula
brew audit --strict vais

# Check formula style
brew style Formula/vais.rb

# Verbose install (for debugging)
brew install --verbose --debug ./Formula/vais.rb

# Clean up
brew uninstall vais
```

### Common Issues

**SHA256 mismatch**
```bash
# Recalculate and update
curl -L URL | shasum -a 256
```

**Build fails**
```bash
# Check build logs
brew install --verbose --debug ./Formula/vais.rb
# Fix dependencies in formula
```

**Test fails**
```bash
# Run test manually
brew test --verbose vais
# Update test block in formula
```

## File Locations

- Formula: `/usr/local/Homebrew/Library/Taps/vaislang/homebrew-vais/Formula/vais.rb`
- Binary: `/usr/local/bin/vaisc`
- Std lib: `/usr/local/share/vais/std/`
- Config: `/usr/local/lib/vais/config.toml`

## Useful Links

- Main repo: https://github.com/vaislang/vais
- Tap repo: https://github.com/vaislang/homebrew-vais
- Homebrew docs: https://docs.brew.sh
- Formula cookbook: https://docs.brew.sh/Formula-Cookbook
