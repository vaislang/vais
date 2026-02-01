# Setting Up the Homebrew Tap

This guide explains how to set up and publish the Vais Homebrew tap.

## Prerequisites

- GitHub account (vaislang)
- Homebrew installed on macOS
- Access to the vais repository

## Step 1: Create the Tap Repository

1. Create a new GitHub repository named `homebrew-vais` under the `vaislang` account
2. The repository name MUST be `homebrew-vais` (Homebrew convention)
3. Repository URL: https://github.com/vaislang/homebrew-vais

## Step 2: Push the Tap Files

```bash
cd /Users/sswoo/study/projects/vais/homebrew-vais

# Initialize git repository
git init
git add .
git commit -m "Initial commit: Add Vais formula"

# Add remote and push
git remote add origin https://github.com/vaislang/homebrew-vais.git
git branch -M main
git push -u origin main
```

## Step 3: Update the Formula for First Release

Before the first release, you need to:

1. Create a release tag in the main vais repository:
   ```bash
   cd /Users/sswoo/study/projects/vais
   git tag -a v0.1.0 -m "Release v0.1.0"
   git push origin v0.1.0
   ```

2. GitHub will automatically create a tarball at:
   `https://github.com/vaislang/vais/archive/refs/tags/v0.1.0.tar.gz`

3. Calculate the SHA256 of the tarball:
   ```bash
   curl -L https://github.com/vaislang/vais/archive/refs/tags/v0.1.0.tar.gz | shasum -a 256
   ```

4. Update the formula with the real SHA256:
   ```bash
   cd homebrew-vais
   # Edit Formula/vais.rb and replace PLACEHOLDER_SHA256 with the actual hash
   git add Formula/vais.rb
   git commit -m "Update SHA256 for v0.1.0"
   git push
   ```

## Step 4: Test the Tap Locally

```bash
# Add the tap
brew tap vaislang/vais

# Install from the tap
brew install vais

# Test the installation
vaisc --version
vaisc --help
vaisc repl

# Run Homebrew tests
brew test vais

# Audit the formula
brew audit --strict vais

# Uninstall when done testing
brew uninstall vais
brew untap vaislang/vais
```

## Step 5: Automate Updates (Optional)

The included GitHub Actions workflow can automatically update the formula when you create a new release.

### Option A: Manual Trigger

1. Go to GitHub Actions in the tap repository
2. Run "Update Formula on Release" workflow
3. Provide version number and SHA256

### Option B: Automatic from Main Repo

Add this to the main vais repository's release workflow:

```yaml
- name: Notify Homebrew tap
  run: |
    VERSION=$(echo ${{ github.ref }} | sed 's/refs\/tags\/v//')
    SHA256=$(curl -L https://github.com/vaislang/vais/archive/refs/tags/v${VERSION}.tar.gz | shasum -a 256 | cut -d' ' -f1)

    curl -X POST \
      -H "Authorization: token ${{ secrets.TAP_UPDATE_TOKEN }}" \
      -H "Accept: application/vnd.github.v3+json" \
      https://api.github.com/repos/vaislang/homebrew-vais/dispatches \
      -d "{\"event_type\":\"new-release\",\"client_payload\":{\"version\":\"${VERSION}\",\"sha256\":\"${SHA256}\"}}"
```

## Step 6: Announce the Tap

Once everything is working:

1. Update the main vais README.md with installation instructions:
   ```markdown
   ## Installation

   ### macOS (Homebrew)
   ```bash
   brew tap vaislang/vais
   brew install vais
   ```
   ```

2. Announce on social media, mailing lists, etc.

## Maintenance

### For Each New Release

1. Tag and release in main repository
2. Calculate new SHA256
3. Update Formula/vais.rb with new version and SHA256
4. Test locally
5. Commit and push

### Testing Checklist

- [ ] Formula installs without errors
- [ ] `vaisc --version` shows correct version
- [ ] `vaisc --help` works
- [ ] `vaisc repl` starts successfully
- [ ] `brew test vais` passes
- [ ] `brew audit --strict vais` passes
- [ ] Standard library is accessible

## Troubleshooting

### "SHA256 mismatch" error

- Recalculate the SHA256 and update the formula
- Make sure you're using the correct release tag URL

### "No such file or directory" during install

- Verify the tarball URL is correct
- Check that the release exists on GitHub

### Tests fail

- Review the test block in the formula
- Make sure the Vais syntax in the test is correct
- Check that `vaisc check` command works

## Resources

- [Homebrew Formula Cookbook](https://docs.brew.sh/Formula-Cookbook)
- [Homebrew Acceptable Formulae](https://docs.brew.sh/Acceptable-Formulae)
- [How to Create and Maintain a Tap](https://docs.brew.sh/How-to-Create-and-Maintain-a-Tap)
