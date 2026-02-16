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

## システム要件

### 必要なソフトウェア

- **Rust 1.70+** - VaisコンパイラはRustで書かれています
- **LLVM/Clang 17+** - LLVMバックエンドのコード生成用
- **Git** - リポジトリのクローン用
- **CMake 3.15+** - LLVMビルドツールに必要

### ハードウェア要件

- 最低2GB RAM
- 少なくとも5GBのディスク容量(ビルド成果物とLLVM用)
- マルチコアプロセッサを推奨

### サポートされているプラットフォーム

- macOS 11+ (IntelとApple Silicon)
- Linux (Ubuntu 20.04+、Fedora 35+、Debian 11+)
- Windows 10+ (MSVCまたはWSL2)

## プラットフォーム別インストール

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

### Linux (Fedora/RHEL)

#### 1. Rustをインストール

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

#### 2. 依存関係をインストール

```bash
sudo dnf install -y \
    gcc-c++ \
    cmake \
    llvm-devel-17 \
    clang-tools-extra-17 \
    pkg-config
```

#### 3. 環境変数を設定

```bash
# ~/.bashrcに追加
export LLVM_DIR=/usr/lib64/llvm-17
export PATH="/usr/lib64/llvm-17/bin:$PATH"
```

#### 4. クローンしてビルド

```bash
git clone https://github.com/vaislang/vais.git
cd vais
cargo build --release
```

### Windows

#### 1. Rustをインストール

https://rustup.rs/ からインストーラーをダウンロードして実行

#### 2. Visual Studio Build Toolsをインストール

https://visualstudio.microsoft.com/visual-cpp-build-tools/ からダウンロード

"Desktop development with C++"を選択

#### 3. LLVM 17をインストール

https://releases.llvm.org/download.html からダウンロード(LLVM-17.x.x-win64.exe)

インストール中に、プロンプトが表示されたらLLVMをPATHに追加してください。

#### 4. CMakeをインストール

https://cmake.org/download/ からダウンロード

CMakeがPATHに追加されていることを確認してください。

#### 5. クローンしてビルド

```bash
git clone https://github.com/vaislang/vais.git
cd vais
cargo build --release
```

## ソースからのビルド

### 開発ビルド

開発中の高速コンパイル用:

```bash
cargo build
```

コンパイラは `./target/debug/vaisc` として利用可能になります

### リリースビルド

最適化されたプロダクションバイナリ用:

```bash
cargo build --release
```

コンパイラは `./target/release/vaisc` として利用可能になります

### 特定の機能でビルド

```bash
# LSPサポート付き(デフォルトで含まれる)
cargo build --release --all-features

# ビルド中にすべてのテストを実行
cargo build --release --all-targets
```

## 環境設定

### VaisをPATHに追加

どこからでもコンパイラに簡単にアクセスするため:

```bash
# リリースビルド用
export PATH="$PATH:$(pwd)/target/release"

# またはシンボリックリンクを作成
ln -s $(pwd)/target/release/vaisc /usr/local/bin/vaisc
```

### 標準ライブラリパスを設定

`VAIS_STD_PATH`環境変数を設定(オプション、デフォルトで自動検出):

```bash
export VAIS_STD_PATH=$(pwd)/std
```

### LSP設定

Vais Language Serverはコンパイラに組み込まれています:

```bash
vaisc lsp
```

これにより、標準入出力でlanguage serverが起動します。

## VSCode拡張機能のインストール

### マーケットプレイスから

1. VSCodeを開く
2. `Ctrl+Shift+X` (Windows/Linux) または `Cmd+Shift+X` (macOS) を押す
3. "vais"または"vais-language"を検索
4. インストールをクリック

### ソースから

```bash
cd vscode-vais
npm install
npm run build
code --install-extension vais-language-server-0.0.1.vsix
```

### 拡張機能の設定

`.vscode/settings.json`に追加:

```json
{
  "vais.compilerPath": "/path/to/vaisc",
  "vais.stdPath": "/path/to/vais/std",
  "vais.lsp.enable": true,
  "[vais]": {
    "editor.defaultFormatter": "vais.vais-language",
    "editor.formatOnSave": true
  }
}
```

## 検証

### インストールをテスト

```bash
# コンパイラのバージョンを確認
vaisc --version

# テストスイートを実行
cargo test --release

# サンプルをコンパイル
vaisc build examples/hello.vais -o hello
./hello
```

### 期待される出力

```bash
$ ./hello
Hello, World!
```

### 包括的なテストの実行

```bash
# ユニットテスト
cargo test --lib

# 統合テスト
cargo test --test '*'

# ベンチマークテスト
cargo bench
```

## トラブルシューティング

### LLVMが見つからない

**エラー:** `error: LLVM 17 not found`

**解決策:**
- LLVM 17がインストールされているか確認: `llvm-config-17 --version`
- `LLVM_DIR`を正しく設定
- macOSの場合: `export LLVM_DIR=$(brew --prefix llvm@17)`

### Rustコンパイルエラー

**エラー:** `error: could not compile vaisc`

**解決策:**
- Rustを更新: `rustup update`
- ビルド成果物をクリーン: `cargo clean && cargo build --release`
- Rustのバージョンを確認: `rustc --version` (1.70+である必要があります)

### ビルド中のメモリ不足

**解決策:**
- スワップ容量またはRAMを増やす
- 最適化なしでビルド: `cargo build` (`--release`の代わりに)
- 他のアプリケーションを閉じる

### 標準ライブラリが見つからない

**エラー:** `error: standard library not found`

**解決策:**
- プロジェクトルートに`std/`ディレクトリが存在することを確認
- `VAIS_STD_PATH`を設定: `export VAIS_STD_PATH=$(pwd)/std`

### VSCode拡張機能が動作しない

**解決策:**
- 設定のコンパイラパスが正しいことを確認
- VSCodeを再起動: `Cmd+Shift+P` → "Developer: Reload Window"
- LSP出力を確認: View → Output → "Vais Language Server"を選択

### プラットフォーム固有の問題

**macOS Apple Silicon (M1/M2):**
- ネイティブARM64ビルドを使用; Rosetta変換は問題を引き起こす可能性があります
- HomebrewとVAIS依存関係がARM64用にインストールされていることを確認

**Windows パスの問題:**
- フルパスを使用するか、システムのPATH環境変数に追加
- PATH変更後、コマンドプロンプト/PowerShellを再起動

**Linux GLIBC互換性:**
- エラー: `GLIBC_2.XX not found`
- 解決策: glibcを更新するか、`RUSTFLAGS="-C target-cpu=generic"`でコンパイル

## Dockerインストール

### ビルド済みイメージを使用

```bash
# プルして実行
docker run -it vaislang/vais:latest

# マウントされたボリュームで実行
docker run -it -v $(pwd):/workspace vaislang/vais:latest
```

### Dockerイメージをビルド

```bash
# Dockerfileからビルド
docker build -t vais:local .

# 実行
docker run -it vais:local
```

## 次のステップ

インストールが成功したら:

1. **チュートリアルを読む:** [チュートリアル](./tutorial.md)
2. **言語仕様:** [言語仕様](../language/language-spec.md)
3. **標準ライブラリ:** [標準ライブラリ](../stdlib/index.md)
4. **サンプルプログラム:** `examples/`ディレクトリ

## ヘルプ

- GitHub Issues: https://github.com/vaislang/vais/issues
- ドキュメント: https://github.com/vaislang/vais/tree/main/docs
- 貢献: `CONTRIBUTING.md`を参照

## バージョン情報

- **Vaisバージョン:** 1.0.0
- **LLVM要件:** 17.x
- **Rust MSRV:** 1.70
- **ライセンス:** MIT
