# 言語仕様

Vaisプログラミング言語の完全なリファレンスです。

## 概要

Vaisは以下のために設計されたシステムプログラミング言語です:

- **トークン効率** - 単一文字キーワードでAIトークン使用量を最小化
- **型安全性** - 完全な推論を備えた強力な静的型付け
- **ネイティブパフォーマンス** - LLVMベースのネイティブコードへのコンパイル
- **モダン機能** - ジェネリクス、トレイト、async/await、パターンマッチング

## キーワード

Vaisは最大効率のために単一文字キーワードを使用:

| キーワード | 意味 | 例 |
|---------|---------|---------|
| `F` | 関数 | `F add(a: i64, b: i64) -> i64 { a + b }` |
| `S` | 構造体 | `S Point { x: f64, y: f64 }` |
| `E` | 列挙型/Else | `E Color { Red, Green, Blue }` / `E { fallback }` |
| `I` | If | `I x > 0 { "positive" }` |
| `L` | ループ | `L i := 0; i < 10; i += 1 { ... }` |
| `M` | マッチ | `M x { 1 => "one", _ => "other" }` |
| `R` | リターン | `R 42` |
| `B` | Break | `B` |
| `C` | Continue | `C` |
| `W` | トレイト | `W Printable { F print(self) }` |
| `X` | 実装 | `X Point: Printable { ... }` |
| `U` | Use/インポート | `U std/io` |
| `P` | Public | `P F public_fn() {}` |
| `T` | 型エイリアス | `T Int = i64` |
| `A` | Async | `A F fetch() -> str { ... }` |
| `Y` | Await | `result := Y fetch()` |
| `N` | Extern | `N F malloc(size: i64) -> i64` |
| `G` | Global | `G counter: i64 = 0` |
| `D` | Defer | `D cleanup()` |
| `O` | Union | `O Data { i: i64, f: f64 }` |

## 演算子

### 特殊演算子

| 演算子 | 意味 | 例 |
|----------|---------|---------|
| `@` | 自己再帰 | `@(n-1) + @(n-2)` |
| `:=` | 変数束縛 | `x := 5` |
| `:= mut` | ミュータブル束縛 | `x := mut 0` |
| `?` | Try(エラー伝播) | `result?` |
| `!` | Unwrap | `result!` |
| `\|>` | パイプ | `x \|> f \|> g` |
| `..` | 範囲 | `1..10` |

### 算術演算子

```vais
+ - * / %        # 基本算術
+= -= *= /= %=   # 複合代入
```

### 比較演算子

```vais
== != < > <= >=
```

### 論理演算子

```vais
&& ||  # 論理AND、OR
!      # 論理NOT
```

### ビット演算子

```vais
& | ^ << >>      # AND、OR、XOR、左シフト、右シフト
```

## 型

### プリミティブ型

```vais
# 整数
i8 i16 i32 i64 i128
u8 u16 u32 u64 u128

# 浮動小数点
f32 f64

# ブール
bool

# 文字列
str
```

### 複合型

```vais
# 配列
[i64; 10]         # i64の固定サイズ配列(10個)

# スライス
&[i64]            # イミュータブルスライス
&mut [i64]        # ミュータブルスライス

# タプル
(i64, f64, str)

# ポインタ
*i64              # 生ポインタ
```

### ジェネリック型

```vais
Vec<T>            # ジェネリックベクター
HashMap<K, V>     # ジェネリックハッシュマップ
Option<T>         # オプショナル値
Result<T, E>      # エラー付き結果
```

## 変数宣言

```vais
# 型推論
x := 42                 # i64が推論される
y := 3.14               # f64が推論される

# 明示的な型
count: i64 = 100

# ミュータブル
counter := mut 0
counter = counter + 1

# 複数宣言
a := 1
b := 2
c := 3
```

## 関数

### 基本的な関数

```vais
F add(a: i64, b: i64) -> i64 {
    a + b
}
```

### 戻り値なし

```vais
F greet(name: str) {
    puts("Hello, ")
    puts(name)
}
```

### 自己再帰

```vais
F factorial(n: i64) -> i64 {
    I n <= 1 { R 1 }
    n * @(n - 1)
}
```

### ジェネリック関数

```vais
F identity<T>(x: T) -> T {
    x
}

F max<T>(a: T, b: T) -> T {
    I a > b { a } E { b }
}
```

## 制御フロー

### if式

```vais
# シンプルなif
I x > 0 {
    puts("positive")
}

# if-else
I x > 0 {
    puts("positive")
} E {
    puts("negative or zero")
}

# 式としてのif
sign := I x > 0 { 1 } E I x < 0 { -1 } E { 0 }
```

### match式

```vais
M x {
    0 => "zero",
    1 => "one",
    2 => "two",
    _ => "many"
}
```

### ループ

```vais
# Cスタイルループ
L i := 0; i < 10; i += 1 {
    print_i64(i)
}

# 無限ループ
L {
    I should_break { B }
}

# breakとcontinueを使ったループ
L i := 0; i < 20; i += 1 {
    I i % 2 == 0 { C }  # 偶数をスキップ
    I i > 15 { B }      # 15でbreak
    print_i64(i)
}
```

## 構造体

```vais
# 構造体を定義
S Point {
    x: f64,
    y: f64
}

# インスタンスを作成
p := Point { x: 3.0, y: 4.0 }

# フィールドにアクセス
x_coord := p.x
```

### メソッド

```vais
X Point {
    F distance(self) -> f64 {
        sqrt(self.x * self.x + self.y * self.y)
    }

    F translate(self, dx: f64, dy: f64) -> Point {
        Point { x: self.x + dx, y: self.y + dy }
    }
}
```

## 列挙型

```vais
# シンプルな列挙型
E Color {
    Red,
    Green,
    Blue
}

# データ付き列挙型
E Option<T> {
    Some(T),
    None
}

E Result<T, E> {
    Ok(T),
    Err(E)
}
```

## パターンマッチング

```vais
E Option<T> { Some(T), None }

F unwrap_or<T>(opt: Option<T>, default: T) -> T {
    M opt {
        Some(v) => v,
        None => default
    }
}
```

## トレイト

```vais
# トレイトを定義
W Printable {
    F print(self)
}

# トレイトを実装
S Person { name: str, age: i64 }

X Person: Printable {
    F print(self) {
        puts(self.name)
    }
}
```

## ジェネリクス

```vais
# ジェネリック構造体
S Box<T> {
    value: T
}

# ジェネリック関数
F swap<T>(a: T, b: T) -> (T, T) {
    (b, a)
}

# ジェネリックトレイト
W Container<T> {
    F get(self) -> T
}
```

## エラーハンドリング

### Option型

```vais
E Option<T> { Some(T), None }

F find(arr: &[i64], target: i64) -> Option<i64> {
    # ... 検索ロジック
    Some(index)  # またはNone
}
```

### Result型

```vais
E Result<T, E> { Ok(T), Err(E) }

F divide(a: i64, b: i64) -> Result<i64, str> {
    I b == 0 { R Err("division by zero") }
    Ok(a / b)
}
```

### Try演算子 `?`

```vais
F compute() -> Result<i64, str> {
    a := divide(10, 2)?   # エラーを伝播
    b := divide(a, 3)?
    Ok(b)
}
```

### Unwrap演算子 `!`

```vais
result := divide(10, 2)
value := result!  # Errの場合パニック
```

## モジュールシステム

```vais
# モジュールをインポート
U std/io
U std/vec

# モジュールからアイテムを使用
v := Vec::new()
content := read_file("data.txt")
```

## コメント

```vais
# 単一行コメント

F main() {
    x := 42  # インラインコメント
}
```

## 文字列補間

```vais
name := "Alice"
age := 30

# 変数補間(非サポート - putsを使用)
puts("Name: ")
puts(name)

# 連結
msg := "Hello, " + name
```

## 組み込み関数

```vais
# I/O
puts(s: str)              # 文字列を出力
print_i64(x: i64)         # 整数を出力
print_f64(x: f64)         # 浮動小数点を出力

# メモリ
malloc(size: i64) -> i64  # メモリを割り当て
free(ptr: i64)            # メモリを解放

# 型操作
sizeof(T) -> i64          # 型のサイズ
```

## ベストプラクティス

1. **型推論を使う** 型が明白な場合
2. **明示的な型を使う** 関数パラメータと戻り値
3. **式を優先** 文より(ifステートメントではなく`I`を使用)
4. **`@`を再帰に使う** 関数名ではなく
5. **エラーを処理** `Result`と`?`演算子で
6. **パターンマッチングを使う** `M`で複雑な条件分岐
7. **関数を小さく保つ** 単一責任に焦点を当てる

## 例

### フィボナッチ

```vais
F fib(n: i64) -> i64 {
    I n <= 1 { R n }
    @(n-1) + @(n-2)
}
```

### 連結リスト

```vais
S Node<T> {
    value: T,
    next: Option<Box<Node<T>>>
}

X Node<T> {
    F new(value: T) -> Node<T> {
        Node { value: value, next: None }
    }
}
```

### エラーハンドリング

```vais
F parse_number(s: str) -> Result<i64, str> {
    # パースロジック
    I is_valid {
        Ok(number)
    } E {
        Err("Invalid number")
    }
}
```

## さらに学ぶ

- [チュートリアル](../getting-started/tutorial.md) - ステップバイステップガイド
- [標準ライブラリ](https://github.com/vaislang/vais/tree/main/std) - 組み込みモジュール
- [サンプル](https://github.com/vaislang/vais/tree/main/examples) - コードサンプル
