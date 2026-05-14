# Vaisプログラミング言語

**Vais** (Vibe AI Language for Systems) は、AI支援開発とLLMコード生成に最適化されたシステムプログラミング言語です。現在の公開説明は canonical syntax と gate-backed claims を基準にしています。

> **現在の公開ステータス:** Vais は certified Core compiler と明示された promoted runtime gate を公開基準としています。product-complete v1.0 release ではありません。公開 claim の境界は [`PUBLIC_STATUS.md`](https://github.com/vaislang/vais/blob/main/PUBLIC_STATUS.md) を参照してください。

## 主な特徴

- **Canonical keywords** - `fn`, `struct`, `enum`, `else`, `match`, `return`, `use`, `pub` が現在の標準です。`F/S/E/EN/EL/M/R/T/U/P/W/X` は retired form です。
- **自己再帰演算子 `@`** - 現在の関数を再帰的に呼び出す
- **式指向** - すべてが式として値を返す
- **LLVMバックエンド** - LLVM 17による promoted native codegen path
- **型推論** - certified Core surface の最小限の型注釈。より広い推論機能は hardening 中
- **メモリ安全性** - Non-Lexical Lifetimes(NLL)によるborrow checker、`--strict-borrow`モード
- **スライス型** - `&[T]` / `&mut [T]` のfat pointer実装
- **並列コンパイル** - DAGベースの型チェックとコード生成 workbench
- **セルフホスティング workbench** - bootstrap と conformance work に使う 50,000+ LOC の Vais compiler source
- **エコシステム workbench** - std と downstream package は明示された gate で追跡

## クイック例

```vais
# 自己再帰によるフィボナッチ
fn fib(n:i64)->i64 = n<2 ? n : @(n-1) + @(n-2)

# 構造体定義
struct Point { x:f64, y:f64 }

# 配列の合計をループで計算
fn sum(arr:[i64])->i64 {
    s := 0
    L x:arr { s += x }
    s
}
```

## 構文概要

| キーワード | 意味 | 例 |
|---------|---------|---------|
| `fn` | Function | `fn add(a:i64,b:i64)->i64=a+b` |
| `struct` | Struct | `struct Point{x:f64,y:f64}` |
| `enum` / `else` | Enum / Else | `enum Option<T>{Some(T),None}` / `else {-1}` |
| `I` | If | `I x>0{1}else{-1}` |
| `LF` | Range loop | `LF i:0..10{print(i)}` |
| `match` | Match | `match opt{Some(v)=>v,None=>0}` |
| `@` | Self-call | `@(n-1)` (再帰呼び出し) |
| `:=` | Infer & assign | `x := 42` |

## プロジェクト構造

Vaisコンパイラは28以上のcrateで構成されたモジュール型システムです:

```
crates/
├── vais-ast/          # AST定義
├── vais-lexer/        # トークナイザ(logosベース)
├── vais-parser/       # 再帰下降パーサ
├── vais-types/        # 型チェッカと型推論
├── vais-codegen/      # LLVM IRコードジェネレータ(inkwell/, advanced_opt/)
├── vais-codegen-js/   # JavaScript(ESM)コードジェネレータ
├── vais-mir/          # Middle IR
├── vaisc/             # メインコンパイラCLI & REPL
├── vais-lsp/          # Language Server Protocol
├── vais-jit/          # Cranelift JITコンパイラ
├── vais-gpu/          # GPUコード生成(CUDA/Metal/OpenCL/WebGPU)
├── vais-bindgen/      # FFIバインディングジェネレータ(C/WASM-JS)
├── vais-registry-server/    # パッケージレジストリ(Axum/SQLite)
├── vais-playground-server/  # Webプレイグラウンドバックエンド
└── ... (その他多数)

std/               # 標準ライブラリ(74モジュール)
selfhost/          # セルフホスティングコンパイラ(51,190 LOC、58 .vaisファイル)
examples/          # サンプルプログラム(189ファイル)
```

## コンパイルパイプライン

```
.vais ソース → Lexer → Parser → AST → Type Checker → Codegen → .ll (LLVM IR) → clang → バイナリ
                                                      ↘ JS Codegen → .mjs (ESM)
                                                      ↘ WASM Codegen → .wasm (experimental unless gated)
```

## パフォーマンス

Vaisはコンパイル速度と実行速度の両方を重視して設計されています。

### コンパイル速度

現在の単一ファイル compile-speed benchmark
(`benches/lang-comparison/compile_bench.sh`, Hyperfine, 2026-05-13,
Apple ARM64/macOS) では、4つの benchmark program に対する Vais
`--emit-ir` の平均は 6.3ms です。これは Vais の LLVM IR emission と、
他 toolchain の full binary compilation を比較する scoped benchmark claim です。

| フェーズ | 時間(平均) | スループット | 備考 |
|-------|------------|------------|-------|
| Lexer | ~0.5ms/1K LOC | ~2M トークン/秒 | |
| Parser | ~1.2ms/1K LOC | ~800K AST ノード/秒 | 並列化で2.18倍高速化 |
| Type Checker | ~2.5ms/1K LOC | ~400K 型/秒 | DAGベース並列処理 |
| Code Generator | ~3.0ms/1K LOC | ~300K IR 行/秒 | 並列化で4.14倍高速化 |
| **完全パイプライン** | **~1.25ms/1K LOC** | **~800K 行/秒** | **50K 行 → 63ms** |

**セルフホスティング:** repository には bootstrap と conformance work に使う 50,000+ LOC の Vais compiler source があります。現在の正しさは certified Core gate と promoted runtime fixture で判断します。

### 実行速度

Historical Fibonacci(35) runtime snapshot (Apple M-series ARM64)。この数値は、
current compiler で runtime benchmark suite を refresh するまで scoped evidence
として保持します:

| 言語 | 時間 | 相対値 |
|----------|------|----------|
| C (clang -O3) | 32ms | 0.94x |
| Rust (release) | 33ms | 0.97x |
| **Vais** (clang -O2) | **34ms** | **1.0x** |
| Python | 3200ms | ~94倍遅い |

## ステータス

- ✅ Lexer (logosベースのトークナイザ)
- ✅ Parser (再帰下降)
- ✅ Type checker (Core-certified fixtures plus broader generics/traits workbench)
- ✅ Code generator (promoted LLVM IR native path; JavaScript ESM and WASM are experimental unless gated)
- ✅ Standard library (74モジュール: Vec、HashMap、String、File、Net、Async、GPUなど)
- ✅ Borrow checker (Non-Lexical Lifetimes、CFGベースデータフロー、`--strict-borrow`)
- ✅ スライス型 (`&[T]` / `&mut [T]` with fat pointers)
- ✅ 並列コンパイル (DAGベース依存関係解決、2-4倍高速化)
- ✅ セルフホスティング workbench (50,000+ LOC)
- ✅ LSPサポート (診断、補完、ホバー、定義へジャンプ、参照、リネーム)
- ✅ REPL (対話型環境)
- ✅ VSCode拡張 + IntelliJプラグイン (構文ハイライト、LSP統合)
- ✅ Optimizer (定数畳み込み、DCE、CSE、ループ展開、LICM、alias解析、ベクトル化)
- ✅ Formatter (`vaisc fmt`)
- ✅ Debugger (DWARFメタデータ、lldb/gdbサポート)
- ✅ エコシステムパッケージ (vais-aes、vais-base64、vais-crc32、vais-csv、vais-json、vais-lz4、vais-regex、vais-sha256、vais-uuid)

## スタートガイド

1. システムに[Vaisをインストール](./getting-started/installation.md)
2. [クイックスタート](./getting-started/quick-start.md)ガイドに従う
3. [チュートリアル](./getting-started/tutorial.md)で言語の基礎を学習
4. 完全なリファレンスは[言語仕様](./language/language-spec.md)を参照
5. [標準ライブラリ](./stdlib/index.md)で利用可能なモジュールを探索

## ドキュメント

### 公式ドキュメントサイト

包括的なドキュメントは対話型のmdBookサイトとして提供されています:

```bash
# ドキュメントをビルドして表示
cd docs-site
./serve.sh
```

[オンラインドキュメント](https://vaislang.dev/docs/)にアクセスするか、個別のファイルを参照してください:

- [言語仕様](./language/language-spec.md) - 完全な言語仕様
- [標準ライブラリ](./stdlib/index.md) - 標準ライブラリリファレンス
- [チュートリアル](./getting-started/tutorial.md) - 入門チュートリアル
- [アーキテクチャ](./advanced/architecture.md) - コンパイラアーキテクチャと設計

## リンク

| リソース | URL |
|----------|-----|
| **GitHub Organization** | https://github.com/vaislang |
| **リポジトリ** | https://github.com/vaislang/vais |
| **ドキュメント** | https://vaislang.dev/docs/ |
| **プレイグラウンド** | https://vaislang.dev/playground/ |
| **Webサイト** | https://vaislang.dev/ |
| **Docker Hub** | `vaislang/vais` |
| **Homebrew Tap** | `vaislang/tap` |
| **エコシステムパッケージ** | https://github.com/vaislang/vais/tree/main/packages (9パッケージ) |

## コミュニティ

- [GitHub Discussions](https://github.com/vaislang/vais/discussions) - 質問、アイデア、成果の共有
- [Contributing Guide](https://github.com/vaislang/vais/blob/main/CONTRIBUTING.md) - 貢献方法
- [CHANGELOG](https://github.com/vaislang/vais/blob/main/CHANGELOG.md) - リリース履歴

## ライセンス

MIT License - VaisはMITライセンスのオープンソースソフトウェアです。
