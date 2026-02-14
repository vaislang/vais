# インストール

このガイドでは、Vaisコンパイラをシステムにインストールする方法を説明します。

## クイックインストール(推奨)

Vaisを始める最も早い方法です。

### Homebrew (macOS / Linux)

```bash
brew tap vaislang/tap
brew install vais
```

### ビルド済みバイナリ

[GitHubリリース](https://github.com/vaislang/vais/releases/tag/v1.0.0)からダウンロード:

| プラットフォーム | ダウンロード |
|----------|----------|
| macOS ARM (Apple Silicon) | [vais-v1.0.0-aarch64-apple-darwin.tar.gz](https://github.com/vaislang/vais/releases/download/v1.0.0/vais-v1.0.0-aarch64-apple-darwin.tar.gz) |
| macOS Intel | [vais-v1.0.0-x86_64-apple-darwin.tar.gz](https://github.com/vaislang/vais/releases/download/v1.0.0/vais-v1.0.0-x86_64-apple-darwin.tar.gz) |
| Linux x86_64 | [vais-v1.0.0-x86_64-unknown-linux-gnu.tar.gz](https://github.com/vaislang/vais/releases/download/v1.0.0/vais-v1.0.0-x86_64-unknown-linux-gnu.tar.gz) |
| Windows x86_64 | [vais-v1.0.0-x86_64-pc-windows-msvc.zip](https://github.com/vaislang/vais/releases/download/v1.0.0/vais-v1.0.0-x86_64-pc-windows-msvc.zip) |

```bash
# 例: macOS ARM
curl -LO https://github.com/vaislang/vais/releases/download/v1.0.0/vais-v1.0.0-aarch64-apple-darwin.tar.gz
tar -xzf vais-v1.0.0-aarch64-apple-darwin.tar.gz
sudo cp vais/vaisc /usr/local/bin/
```

### 前提条件: clang

Vaisは生成されたLLVM IRをネイティブバイナリにコンパイルするために`clang`を使用します:

- **macOS**: `xcode-select --install`
- **Linux**: `sudo apt install clang` または `sudo dnf install clang`
- **Windows**: https://releases.llvm.org からインストール

## ソースからビルド

ソースからビルドする場合、または開発に貢献する場合。

### システム要件

- **Rust 1.70+** - VaisコンパイラはRustで書かれています
- **LLVM/Clang 17+** - LLVMバックエンドのコード生成用
- **Git** - リポジトリのクローン用
- **CMake 3.15+** - LLVMビルドツールに必要

### macOS

```bash
# Rustをインストール
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Homebrewで依存関係をインストール
brew install llvm@17 clang cmake pkg-config

# 環境設定
export LLVM_DIR=/usr/local/opt/llvm@17
export PATH="/usr/local/opt/llvm@17/bin:$PATH"

# クローンしてビルド
git clone https://github.com/vaislang/vais.git
cd vais
cargo build --release
```

### Linux (Ubuntu/Debian)

```bash
# Rustをインストール
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# 依存関係をインストール
sudo apt-get update
sudo apt-get install -y build-essential cmake pkg-config \
    llvm-17 clang-17 libllvm-17-ocaml-dev

# 環境設定
export LLVM_DIR=/usr/lib/llvm-17
export PATH="/usr/lib/llvm-17/bin:$PATH"

# クローンしてビルド
git clone https://github.com/vaislang/vais.git
cd vais
cargo build --release
```

### Windows

```bash
# https://rustup.rs/ からRustをインストール
# Visual Studio Build Toolsをインストール
# https://releases.llvm.org からLLVM 17をインストール
# https://cmake.org/download/ からCMakeをインストール

# クローンしてビルド
git clone https://github.com/vaislang/vais.git
cd vais
cargo build --release
```

## 検証

インストールをテスト:

```bash
# コンパイラのバージョンを確認
vaisc --version

# サンプルをコンパイル
vaisc build examples/hello.vais -o hello
./hello
```

期待される出力:

```
Hello, World!
```

## トラブルシューティング

### LLVMが見つからない

**エラー:** `error: LLVM 17 not found`

**解決策:**
- LLVM 17がインストールされているか確認: `llvm-config-17 --version`
- `LLVM_DIR`を正しく設定
- macOSの場合: `export LLVM_DIR=$(brew --prefix llvm@17)`

### 標準ライブラリが見つからない

**エラー:** `error: standard library not found`

**解決策:**
- プロジェクトルートに`std/`ディレクトリが存在することを確認
- `VAIS_STD_PATH`を設定: `export VAIS_STD_PATH=$(pwd)/std`

## 次のステップ

- [クイックスタート](./quick-start.md)ガイドに従う
- [チュートリアル](./tutorial.md)を読む
- [サンプルプログラム](https://github.com/vaislang/vais/tree/main/examples)を探索
