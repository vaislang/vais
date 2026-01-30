Name:           vais
Version:        0.2.0
Release:        1%{?dist}
Summary:        AI-optimized systems programming language

License:        MIT
URL:            https://github.com/sswoo88/vais
Source0:        %{name}-%{version}.tar.gz

BuildRequires:  rust >= 1.70
BuildRequires:  cargo
BuildRequires:  llvm-devel >= 17
Requires:       clang
Requires:       llvm-libs >= 17

%description
Vais is a modern systems programming language with an LLVM backend,
designed for AI-assisted development. It combines low-level control
with high-level abstractions, featuring static typing with type
inference, memory safety with ownership tracking, native LLVM code
generation, and rich standard library.

%prep
%setup -q

%build
cargo build --release --bin vaisc

%install
rm -rf %{buildroot}
mkdir -p %{buildroot}%{_bindir}
mkdir -p %{buildroot}%{_datadir}/vais/std

# Install binary
install -m 0755 target/release/vaisc %{buildroot}%{_bindir}/vaisc

# Install standard library
cp -r std/* %{buildroot}%{_datadir}/vais/std/

%files
%license LICENSE
%doc README.md CHANGELOG.md
%{_bindir}/vaisc
%{_datadir}/vais/

%changelog
* Thu Jan 30 2026 Steve <steve@vais-lang.org> - 0.2.0-1
- Implement array mutation, format function, and stdlib utilities
- Add first-class string operations with runtime support
- Implement print/println built-in functions
- Resolve critical codegen bugs
- Complete IDE experience with inlay hints and refactoring tools

* Mon Jan 13 2026 Steve <steve@vais-lang.org> - 0.1.0-1
- Initial RPM release
- Core language features and LLVM backend
- Standard library with collections, I/O, and networking
- LSP and DAP support
