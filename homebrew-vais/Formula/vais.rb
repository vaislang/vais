class Vais < Formula
  desc "AI-optimized systems programming language with LLVM backend"
  homepage "https://github.com/sswoo88/vais"
  url "https://github.com/sswoo88/vais/archive/refs/tags/v1.0.0.tar.gz"
  sha256 "PLACEHOLDER_SHA256"
  license "MIT"
  head "https://github.com/sswoo88/vais.git", branch: "main"

  depends_on "rust" => :build
  depends_on "llvm@17"

  def install
    # Build with cargo
    system "cargo", "build", "--release", "--bin", "vaisc"

    # Install the compiler binary
    bin.install "target/release/vaisc"

    # Install the standard library
    (share/"vais/std").install Dir["std/*"]

    # Set up environment to help vaisc find the standard library
    (lib/"vais").mkpath
    File.write(lib/"vais/config.toml", <<~EOS)
      [paths]
      std_lib = "#{share}/vais/std"
    EOS
  end

  def caveats
    <<~EOS
      The Vais standard library is installed at:
        #{share}/vais/std

      To compile Vais programs, you need LLVM tools in your PATH.
      The vaisc compiler will use clang to compile generated LLVM IR.

      To get started:
        vaisc --help
        vaisc repl   # Start the REPL
    EOS
  end

  test do
    # Create a simple Vais program using actual Vais syntax
    (testpath/"hello.vais").write <<~EOS
      # Hello World test
      F main()->i64 {
          puts("Hello from Vais!")
          0
      }
    EOS

    # Test that vaisc can parse and check the file
    system bin/"vaisc", "check", "hello.vais"

    # Test --help flag
    system bin/"vaisc", "--help"

    # Test --version flag
    assert_match version.to_s, shell_output("#{bin}/vaisc --version")
  end
end
