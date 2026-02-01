class Vais < Formula
  desc "AI-optimized systems programming language with LLVM backend"
  homepage "https://github.com/vaislang/vais"
  version "1.0.0"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/vaislang/vais/releases/download/v1.0.0/vais-v1.0.0-aarch64-apple-darwin.tar.gz"
      sha256 "b4f7df6940073b8e82a70670d0fa548daf5718d707c242c78fad1acfe1a681e9"
    else
      url "https://github.com/vaislang/vais/releases/download/v1.0.0/vais-v1.0.0-x86_64-apple-darwin.tar.gz"
      sha256 "cdff8c3d45ac59825b90e8bfb32ac790fcd8f85493e3539724230432c5fa65d3"
    end
  end

  on_linux do
    url "https://github.com/vaislang/vais/releases/download/v1.0.0/vais-v1.0.0-x86_64-unknown-linux-gnu.tar.gz"
    sha256 "bb41c41197d02013c22a11033c60fd4d300a17a60f0fe65efe22e111a36c1936"
  end

  def install
    # tar extracts to vais/ directory, Homebrew may or may not cd into it
    if File.exist?("vaisc")
      bin.install "vaisc"
      (share/"vais/std").install Dir["std/*"]
    elsif File.exist?("vais/vaisc")
      bin.install "vais/vaisc"
      (share/"vais/std").install Dir["vais/std/*"]
    end
  end

  def caveats
    <<~EOS
      The Vais standard library is installed at:
        #{share}/vais/std

      To compile Vais programs, you need clang in your PATH.
      On macOS, install Xcode Command Line Tools: xcode-select --install

      To get started:
        vaisc --help
        vaisc repl
    EOS
  end

  test do
    assert_match "vaisc 1.0.0", shell_output("#{bin}/vaisc --version")
  end
end
