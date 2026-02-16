# 目次

[はじめに](./introduction.md)

# スタートガイド

- [インストール](./getting-started/installation.md)
- [クイックスタート](./getting-started/quick-start.md)
- [チュートリアル](./getting-started/tutorial.md)
- [Getting Started（詳細）](../guide/getting-started.md)

# ガイド

- [エラー処理パターン](../guide/error-handling.md)
- [パフォーマンス最適化](../guides/performance.md)
- [コーディングスタイル](../guide/style-guide.md)
- [RustからVaisへの移行](../guides/migration-from-rust.md)
- [C/C++からVaisへの移行](../guides/migration-from-c.md)
- [Cookbook — 実践レシピ](../guides/cookbook.md)
- [テストガイド](../guides/testing.md)
- [エラー処理ベストプラクティス](../guides/error-handling.md)
- [エコシステムパッケージ](../guide/ecosystem-packages.md)
- [トラブルシューティング & FAQ](../troubleshooting.md)

# 言語リファレンス

- [言語仕様](./language/language-spec.md)
- [ジェネリクス](../language/generics.md)
- [型推論](../language/type-inference.md)
- [イテレータ型推論](../language/iterator-type-inference.md)
- [高度な型システム](../language/advanced-types.md)
- [スライス型](../language/slices.md)
- [ライフタイム & 借用チェック](../language/lifetimes.md)
- [非同期プログラミング](../language/async-tutorial.md)
- [クロージャ & ラムダ](../language/closures.md)
- [遅延評価（Lazy/Force）](../language/lazy-evaluation.md)
- [コンパイル時機能](../language/comptime-feature.md)

# 標準ライブラリ

- [標準ライブラリリファレンス](../stdlib/stdlib.md)
- [Vec](../stdlib/vec.md)
- [HashMap](../stdlib/hashmap.md)
- [File I/O](../stdlib/file_io.md)
- [ネットワーキング](../stdlib/net.md)
- [スレッド](../stdlib/thread.md)
- [チャンネル](../stdlib/channel.md)
- [同期](../stdlib/sync.md)
- [JSON](../stdlib/json.md)
- [Regex](../stdlib/regex.md)
- [暗号](../stdlib/crypto.md)

# コンパイラ

- [アーキテクチャ](../compiler/architecture.md)
- [技術仕様](../compiler/tech-spec.md)
- [JITコンパイル](../compiler/jit-compilation.md)
- [GPUコード生成](../compiler/gpu-codegen.md)
- [JavaScriptコード生成](../compiler/js-codegen.md)
- [コンパイラ内部構造](../compiler/internals.md)
- [Inkwell統合](../compiler/inkwell-integration.md)
- [単形化設計](../compiler/monomorphization-design.md)
- [ベンチマーク設計](../compiler/benchmark-design.md)

# 開発ツール

- [エディタ統合](../tools/editors.md)
- [LSPサーバー](../tools/lsp-features.md)
- [プレイグラウンド](../tools/playground/README.md)
  - [機能](../tools/playground/features.md)
  - [クイックスタート](../tools/playground/quickstart.md)
  - [チュートリアル](../tools/playground/tutorial.md)
  - [統合](../tools/playground/integration.md)
- [インタラクティブチュートリアル](../tools/vais-tutorial/README.md)
  - [使用法](../tools/vais-tutorial/usage.md)
  - [クイックスタート](../tools/vais-tutorial/quickstart.md)
- [ホットリロード](../tools/hot-reload.md)
- [コードカバレッジ](../tools/coverage.md)
- [パッケージマネージャー](../tools/package-manager.md)

# 高度なトピック

- [FFI（Foreign Function Interface）](../advanced/ffi/README.md)
  - [FFIガイド](../advanced/ffi/guide.md)
  - [FFI機能](../advanced/ffi/features.md)
- [言語バインディング](../advanced/language-bindings.md)
- [Bindgen](../advanced/bindgen/README.md)
  - [C++サポート](../advanced/bindgen/cpp-support.md)
  - [C++クイックスタート](../advanced/bindgen/cpp-quickstart.md)
  - [デザイン](../advanced/bindgen/design.md)
- [WASMコンポーネント](../advanced/wasm/README.md)
  - [Getting Started](../advanced/wasm/getting-started.md)
  - [コンポーネントモデル](../advanced/wasm/component-model.md)
  - [JS相互運用](../advanced/wasm/js-interop.md)
  - [WASI](../advanced/wasm/wasi.md)
- [非同期ランタイム](../advanced/async-runtime.md)
- [国際化（i18n）](../advanced/i18n-design.md)
- [IPv6実装](../advanced/ipv6-implementation.md)
- [パッケージマネージャー](../advanced/package-manager-design.md)
- [プラグインシステム](../advanced/plugin-system-design.md)
- [Range型実装](../advanced/range-type-implementation.md)
- [セルフホスティング](../advanced/self-hosting-design.md)

# セキュリティ

- [インポートパスセキュリティ](../security/import-path-security.md)
- [セキュリティ強化](../security/security-enhancement.md)

---

# APIリファレンス

- [インデックス](../api/index.md)

## コア型

- [Option](../api/option.md)
- [Result](../api/result.md)
- [String](../api/string.md)
- [OwnedString](../api/owned_string.md)
- [ByteBuffer](../api/bytebuffer.md)
- [Box](../api/box.md)
- [Rc](../api/rc.md)
- [Fmt](../api/fmt.md)

## コレクション

- [Vec](../api/vec.md)
- [HashMap](../api/hashmap.md)
- [StringMap](../api/stringmap.md)
- [BTreeMap](../api/btreemap.md)
- [Set](../api/set.md)
- [Deque](../api/deque.md)
- [PriorityQueue](../api/priority_queue.md)
- [Collections](../api/collections.md)

## ユーティリティ

- [Path](../api/path.md)
- [DateTime](../api/datetime.md)
- [Channel](../api/channel.md)
- [Args](../api/args.md)

## I/Oとファイルシステム

- [IO](../api/io.md)
- [File](../api/file.md)
- [Filesystem](../api/filesystem.md)

## ネットワークとWeb

- [Net](../api/net.md)
- [HTTP](../api/http.md)
- [HTTP Client](../api/http_client.md)
- [HTTP Server](../api/http_server.md)
- [WebSocket](../api/websocket.md)
- [TLS](../api/tls.md)
- [URL](../api/url.md)

## 並行性

- [Thread](../api/thread.md)
- [Sync](../api/sync.md)
- [Future](../api/future.md)
- [Async](../api/async.md)
- [Runtime](../api/runtime.md)
- [Async Reactor](../api/async_reactor.md)

## データ処理

- [JSON](../api/json.md)
- [Regex](../api/regex.md)
- [Base64](../api/base64.md)
- [Template](../api/template.md)
- [Compress](../api/compress.md)

## データベース

- [SQLite](../api/sqlite.md)
- [PostgreSQL](../api/postgres.md)
- [ORM](../api/orm.md)

## 数学とアルゴリズム

- [Math](../api/math.md)
- [Hash](../api/hash.md)
- [Random](../api/random.md)
- [UUID](../api/uuid.md)
- [CRC32](../api/crc32.md)

## セキュリティと暗号

- [Crypto](../api/crypto.md)
- [Log](../api/log.md)

## メモリ管理

- [Memory](../api/memory.md)
- [Allocator](../api/allocator.md)
- [Arena](../api/arena.md)
- [GC](../api/gc.md)

## システムとランタイム

- [Time](../api/time.md)
- [Profiler](../api/profiler.md)
- [Test](../api/test.md)
- [PropTest](../api/proptest.md)
- [Contract](../api/contract.md)
- [GPU](../api/gpu.md)
- [Hot Reload](../api/hot.md)
- [DynLoad](../api/dynload.md)

---

# コントリビューション

- [チームオンボーディングガイド](../onboarding.md)
- [コントリビューションガイド](../contributing/contributing.md)
- [実装サマリー](../contributing/implementation-summaries.md)
  - [全体実装サマリー](../contributing/summaries/implementation-summary.md)
  - [非同期型チェック](../contributing/summaries/async-type-checking.md)
  - [Bindgen実装](../contributing/summaries/bindgen-implementation.md)
  - [C++ Bindgen実装](../contributing/summaries/cpp-bindgen-implementation.md)
  - [FFI実装](../contributing/summaries/ffi-implementation.md)
  - [GC実装](../contributing/summaries/gc-implementation.md)
  - [ホットリロード実装](../contributing/summaries/hot-reload-implementation.md)
  - [プレイグラウンド実装](../contributing/summaries/playground-implementation.md)
  - [WASMコンポーネント実装](../contributing/summaries/wasm-component-implementation.md)
- [リファクタリングサマリー](../contributing/refactoring-summary.md)
- [ロードマップ](../contributing/roadmap.md)
- [プロダクションレディネスチェックリスト](../production-checklist.md)

---

[付録](../appendix.md)
