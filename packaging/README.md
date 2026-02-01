# Vais Linux Packaging

This directory contains packaging scripts for distributing Vais on Linux distributions.

## Available Packages

### Debian/Ubuntu (.deb)

**Location:** `deb/build-deb.sh`

**Build:**
```bash
cd packaging/deb
./build-deb.sh
```

**Install:**
```bash
sudo dpkg -i vais_0.2.0_amd64.deb
sudo apt-get install -f  # Install dependencies if needed
```

**Remove:**
```bash
sudo dpkg -r vais
```

**Requirements:**
- dpkg-deb
- cargo and rust toolchain
- clang (runtime dependency)

### Fedora/RHEL/CentOS (.rpm)

**Location:** `rpm/vais.spec`

**Build:**
```bash
# Create RPM build environment
mkdir -p ~/rpmbuild/{BUILD,RPMS,SOURCES,SPECS,SRPMS}

# Create source tarball
cd ../..
tar czf ~/rpmbuild/SOURCES/vais-0.2.0.tar.gz \
    --transform 's,^,vais-0.2.0/,' \
    --exclude=target \
    --exclude=.git \
    .

# Build RPM
rpmbuild -ba packaging/rpm/vais.spec
```

**Install:**
```bash
sudo rpm -i ~/rpmbuild/RPMS/x86_64/vais-0.2.0-1.*.rpm
# Or with dnf/yum:
sudo dnf install ~/rpmbuild/RPMS/x86_64/vais-0.2.0-1.*.rpm
```

**Remove:**
```bash
sudo rpm -e vais
```

**Requirements:**
- rpm-build
- rust >= 1.70
- cargo
- llvm-devel >= 17
- clang (runtime dependency)

### Arch Linux (PKGBUILD)

**Location:** `arch/PKGBUILD`

**Build and Install:**
```bash
cd packaging/arch

# Build package
makepkg -si

# Or build without installing
makepkg

# Then install manually
sudo pacman -U vais-0.2.0-1-x86_64.pkg.tar.zst
```

**Remove:**
```bash
sudo pacman -R vais
```

**Requirements:**
- base-devel
- rust
- cargo
- clang (runtime dependency)
- llvm>=17 (runtime dependency)

**Note:** Before building, you need to create a source tarball:
```bash
cd ../..
git archive --format=tar.gz --prefix=vais-0.2.0/ HEAD > packaging/arch/vais-0.2.0.tar.gz
```

## Package Contents

All packages install:

- **Binary:** `/usr/bin/vaisc` - The Vais compiler
- **Standard Library:** `/usr/share/vais/std/` - All standard library files
- **Documentation:** README.md, CHANGELOG.md (location varies by distribution)
- **License:** MIT license file

## Version Information

- **Current Version:** 0.2.0
- **Architecture:** x86_64 / amd64
- **License:** MIT

## Dependencies

All packages require:
- **clang** - C compiler for linking
- **LLVM >= 17** - Runtime libraries

Build dependencies (not needed after installation):
- Rust toolchain >= 1.70
- Cargo

## Testing Packages

After installation, verify the installation:

```bash
# Check binary is in PATH
which vaisc

# Check version
vaisc --version

# Check standard library
ls /usr/share/vais/std/

# Compile a test program
echo 'fn main() { println("Hello from Vais!"); }' > test.vais
vaisc test.vais
./test
```

## Publishing to Repositories

### Debian/Ubuntu PPA

1. Sign the package with your GPG key
2. Upload to a PPA or create your own repository
3. Use `dput` for uploading to Launchpad

### Fedora/RHEL Copr

1. Create a Copr project at https://copr.fedorainfracloud.org/
2. Upload the .spec file and source tarball
3. Copr will build packages automatically

### Arch User Repository (AUR)

1. Create an AUR account at https://aur.archlinux.org/
2. Clone the AUR git repository: `git clone ssh://aur@aur.archlinux.org/vais.git`
3. Copy PKGBUILD to the repository
4. Generate .SRCINFO: `makepkg --printsrcinfo > .SRCINFO`
5. Commit and push: `git add -A && git commit -m "Update to 0.2.0" && git push`

## Updating Version

To update the version number:

1. **deb/build-deb.sh:** Update `VERSION="0.2.0"`
2. **rpm/vais.spec:** Update `Version:` and add changelog entry
3. **arch/PKGBUILD:** Update `pkgver=` and regenerate checksum
4. Update this README

## Troubleshooting

**Missing dependencies during build:**
- Install development packages for LLVM and Rust
- Ensure cargo is in your PATH

**dpkg-deb fails:**
- Check that DEBIAN/control file has proper formatting
- Ensure all required files are in the correct locations

**rpmbuild fails:**
- Verify RPM build environment is set up correctly
- Check that source tarball is in SOURCES directory

**makepkg fails:**
- Ensure source tarball exists and matches PKGBUILD
- Run `updpkgsums` to update checksums

## Support

For issues with packaging or installation:
- GitHub Issues: https://github.com/vaislang/vais/issues
- Repository: https://github.com/vaislang/vais
