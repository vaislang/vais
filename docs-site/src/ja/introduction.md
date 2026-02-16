# Vaisプログラミング言語

**Vais** (Vibe AI Language for Systems) は、AI支援開発とLLMコード生成に最適化された、トークン効率の高いシステムプログラミング言語です。トークン使用量を最小化しながらコード表現力を最大化するよう設計されています。

## 主な特徴

- **単一文字キーワード** - `F`(function)、`S`(struct)、`E`(enum/else)、`I`(if)、`L`(loop)、`M`(match)
- **自己再帰演算子 `@`** - 現在の関数を再帰的に呼び出す
- **式指向** - すべてが式として値を返す
- **LLVMバックエンド** - LLVM 17によるネイティブパフォーマンス
- **型推論** - 完全な制約解決による最小限の型注釈
- **メモリ安全性** - Non-Lexical Lifetimes(NLL)によるborrow checker、`--strict-borrow`モード
- **スライス型** - `&[T]` / `&mut [T]` のfat pointer実装
- **並列コンパイル** - DAGベースの並列型チェックとコード生成(2-4倍の高速化)
- **セルフホスティング** - 50,000行以上のbootstrapコンパイラ、clang 21/21件100%成功
- **豊富なエコシステム** - 28以上のcrate、74の標準ライブラリモジュール、成長中のパッケージエコシステム

## クイック例

```vais
# 自己再帰によるフィボナッチ
F fib(n:i64)->i64 = n<2 ? n : @(n-1) + @(n-2)

# 構造体定義
S Point { x:f64, y:f64 }

# 配列の合計をループで計算
F sum(arr:[i64])->i64 {
    s := 0
    L x:arr { s += x }
    s
}
```

## 構文概要

| キーワード | 意味 | 例 |
|---------|---------|---------|
| `F` | Function | `F add(a:i64,b:i64)->i64=a+b` |
| `S` | Struct | `S Point{x:f64,y:f64}` |
| `E` | Enum/Else | `E Option<T>{Some(T),None}` |
| `I` | If | `I x>0{1}E{-1}` |
| `L` | Loop | `L i:0..10{print(i)}` |
| `M` | Match | `M opt{Some(v)=>v,None=>0}` |
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
selfhost/          # セルフホスティングコンパイラ(50,000+ LOC)
examples/          # サンプルプログラム(189ファイル)
```

## コンパイルパイプライン

```
.vais ソース → Lexer → Parser → AST → Type Checker → Codegen → .ll (LLVM IR) → clang → バイナリ
                                                      ↘ JS Codegen → .mjs (ESM)
                                                      ↘ WASM Codegen → .wasm (wasm32)
```

## パフォーマンス

Vaisはコンパイル速度と実行速度の両方を重視して設計されています。

### コンパイル速度

| フェーズ | 時間(平均) | スループット | 備考 |
|-------|------------|------------|-------|
| Lexer | ~0.5ms/1K LOC | ~2M トークン/秒 | |
| Parser | ~1.2ms/1K LOC | ~800K AST ノード/秒 | 並列化で2.18倍高速化 |
| Type Checker | ~2.5ms/1K LOC | ~400K 型/秒 | DAGベース並列処理 |
| Code Generator | ~3.0ms/1K LOC | ~300K IR 行/秒 | 並列化で4.14倍高速化 |
| **完全パイプライン** | **~1.25ms/1K LOC** | **~800K 行/秒** | **50K 行 → 63ms** |

**セルフホスティング Bootstrap:** 50,000+ LOC、clangコンパイル成功率 21/21 (100%)

### 実行速度

Fibonacci(35) ベンチマーク (Apple M-series ARM64, 2026-02-11):

| 言語 | 時間 | 相対値 |
|----------|------|----------|
| C (clang -O3) | 32ms | 0.94x |
| Rust (release) | 33ms | 0.97x |
| **Vais** (clang -O2) | **34ms** | **1.0x** |
| Python | 3200ms | ~94倍遅い |

## ステータス

- ✅ Lexer (logosベースのトークナイザ)
- ✅ Parser (再帰下降)
- ✅ Type checker (ジェネリクス、トレイト、型推論、GAT、object safety)
- ✅ Code generator (LLVM IR (inkwell経由)、JavaScript ESM、WASM)
- ✅ Standard library (74モジュール: Vec、HashMap、String、File、Net、Async、GPUなど)
- ✅ Borrow checker (Non-Lexical Lifetimes、CFGベースデータフロー、`--strict-borrow`)
- ✅ スライス型 (`&[T]` / `&mut [T]` with fat pointers)
- ✅ 並列コンパイル (DAGベース依存関係解決、2-4倍高速化)
- ✅ セルフホスティングコンパイラ (50,000+ LOC、clang 21/21成功、Bootstrap Phase 38)
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

[オンラインドキュメント](https://vais.dev/docs/)にアクセスするか、個別のファイルを参照してください:

- [言語仕様](./language/language-spec.md) - 完全な言語仕様
- [標準ライブラリ](./stdlib/index.md) - 標準ライブラリリファレンス
- [チュートリアル](./getting-started/tutorial.md) - 入門チュートリアル
- [アーキテクチャ](./advanced/architecture.md) - コンパイラアーキテクチャと設計

## リンク

| リソース | URL |
|----------|-----|
| **GitHub Organization** | https://github.com/vaislang |
| **リポジトリ** | https://github.com/vaislang/vais |
| **ドキュメント** | https://vais.dev/docs/ |
| **プレイグラウンド** | https://vais.dev/playground/ |
| **Webサイト** | https://vais.dev/ |
| **Docker Hub** | `vaislang/vais` |
| **Homebrew Tap** | `vaislang/tap` |
| **エコシステムパッケージ** | https://github.com/vaislang/vais/tree/main/packages (9パッケージ) |

## コミュニティ

- [GitHub Discussions](https://github.com/vaislang/vais/discussions) - 質問、アイデア、成果の共有
- [Contributing Guide](https://github.com/vaislang/vais/blob/main/CONTRIBUTING.md) - 貢献方法
- [CHANGELOG](https://github.com/vaislang/vais/blob/main/CHANGELOG.md) - リリース履歴

## ライセンス

MIT License - VaisはMITライセンスのオープンソースソフトウェアです。
