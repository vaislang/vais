# Contributing to homebrew-vais

Thank you for your interest in contributing to the Vais Homebrew tap!

## Updating the Formula

When updating the Vais formula for a new release:

1. Update the `url` field to point to the new release tarball
2. Download the tarball and calculate its SHA256:
   ```bash
   curl -L https://github.com/sswoo88/vais/archive/refs/tags/vX.Y.Z.tar.gz | shasum -a 256
   ```
3. Update the `sha256` field with the calculated hash
4. Test the formula locally:
   ```bash
   brew install --build-from-source ./Formula/vais.rb
   brew test vais
   brew audit --strict vais
   ```
5. Submit a pull request with your changes

## Testing Locally

Before submitting changes, test the formula:

```bash
# Install from local formula
brew install --build-from-source ./Formula/vais.rb

# Run tests
brew test vais

# Audit the formula
brew audit --strict vais

# Uninstall when done testing
brew uninstall vais
```

## Formula Guidelines

- Follow [Homebrew Formula Cookbook](https://docs.brew.sh/Formula-Cookbook) guidelines
- Keep dependencies minimal
- Ensure the test block validates basic functionality
- Use proper Ruby syntax and formatting

## Release Process

1. Tag a new release in the main vais repository
2. Update this formula with the new version and SHA256
3. Test thoroughly
4. Commit and push to the tap repository

## Questions?

Open an issue in the [main Vais repository](https://github.com/sswoo88/vais/issues).
