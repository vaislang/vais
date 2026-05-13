# 言語仕様

バージョン: 0.1.0

## 目次

1. [概要](#概要)
2. [字句構造](#字句構造)
3. [キーワード](#キーワード)
4. [型](#型)
5. [演算子](#演算子)
6. [式](#式)
7. [文](#文)
8. [関数](#関数)
9. [構造体](#構造体)
10. [列挙型](#列挙型)
11. [エラーハンドリング](#エラーハンドリング)
12. [トレイトと実装](#トレイトと実装)
13. [パターンマッチング](#パターンマッチング)
14. [ジェネリクス](#ジェネリクス)
15. [モジュールシステム](#モジュールシステム)
16. [Async/Await](#asyncawait)
17. [イテレータとジェネレータ](#イテレータとジェネレータ)
18. [クロージャとラムダ](#クロージャとラムダ)
19. [メモリ管理](#メモリ管理)
20. [組み込み関数](#組み込み関数)

---

## 概要

Vaisは、AIコード生成で明確なコードを出力し、ネイティブ実行と gate-backed public claims を両立するよう設計されたシステムプログラミング言語です。以下の特徴があります:

- **Canonical keywords** - `fn`, `struct`, `enum`, `else`, `match`, `return`, `use`, `pub` が現在の標準
- **式指向構文** - すべてが値を返す
- **自己再帰演算子 `@`** - 簡潔な再帰関数のため
- **LLVMベースのコンパイル** - ネイティブパフォーマンスのため
- **型推論** - 最小限の注釈で
- **高度な機能**: ジェネリクス、トレイト、Async/Await、パターンマッチング

---

## 字句構造

### コメント

コメントは`#`で始まり、行末まで続きます:

```vais
# これはコメントです
fn add(a:i64, b:i64)->i64 = a + b  # インラインコメント
```

### 空白文字

空白文字(スペース、タブ、改行)は、トークンを分離する場合を除いて無視されます。

### 識別子

識別子は文字またはアンダースコアで始まり、その後に文字、数字、またはアンダースコアが続きます:

```
[a-zA-Z_][a-zA-Z0-9_]*
```

例: `x`, `my_var`, `Counter`, `_private`

### リテラル

**整数リテラル:**
```vais
42
1_000_000    # 可読性のためのアンダースコア
-42          # 負数(単項マイナス演算子を使用)
```

**浮動小数点リテラル:**
```vais
3.14
1.0e10
2.5e-3
1_000.5_00
```

**文字列リテラル:**
```vais
"Hello, World!"
"引用符付き行: \"quotes\""
```

**文字列補間:**
```vais
name := "Vais"
println("Hello, {name}!")           # 変数補間
println("結果: {2 + 3}")            # 式補間
println("エスケープ: {{補間なし}}")  # エスケープされた括弧
```

**ブールリテラル:**
```vais
true
false
```

---

## キーワード

Vais は Step 19 で初期の single-character form を retired にしました。新しいコードは canonical keyword を使います:

| キーワード | 意味 | 使用法 |
|---------|---------|-------|
| `fn` | Function | 関数を定義 |
| `struct` | Struct | 構造体型を定義 |
| `enum` / `else` | Enum / Else | 列挙型を定義、またはifのelse分岐 |
| `I` | If | 条件式 |
| `L` | Loop | ループ構造 |
| `LF` | Range loop | range / foreach loop |
| `match` | Match | パターンマッチング |
| `trait` | Trait | トレイト(インターフェース)を定義 |
| `impl` | Impl | メソッドまたはトレイトを実装 |
| `type` | Type | 型エイリアス定義 |
| `use` | Use | モジュールのインポート/使用 |
| `pub` | Pub | 公開可視性 |
| `A` | Async | 非同期関数マーカー |
| `return` | Return | 関数から早期リターン |
| `B` | Break | ループから抜ける |
| `C` | Continue/Const | 次のループ反復に進む、または定数用 |
| `D` | Defer | 遅延実行 |
| `N` | Extern | 外部関数宣言 |
| `G` | Global | グローバル変数宣言 |
| `O` | Union | Cスタイルのタグなしunion |
| `Y` | Yield/Await | 値をyield (awaitの省略形) |

注: `C`キーワードには二重の意味があります - ループでのcontinueと、定数のための`C`(定数を参照)。コンテキストが使用法を決定します。

### 複数文字キーワード

- `mut` - ミュータブル変数/参照
- `self` - インスタンス参照
- `Self` - impl内での型参照
- `true`, `false` - ブールリテラル
- `spawn` - 非同期タスクを生成
- `await` - 非同期結果を待機(`Y`省略形も利用可能)
- `yield` - イテレータ/コルーチンで値をyield(簡略化実装)

### 省略形キーワード (Phase 29)

| 省略形 | 置き換え | 例 |
|-----------|----------|---------|
| `Y` | `await` | `result.Y` (後置await) |

---

## 演算子

### 算術演算子

| 演算子 | 説明 | 例 |
|----------|-------------|---------|
| `+` | 加算 | `a + b` |
| `-` | 減算または単項否定 | `a - b`, `-x` |
| `*` | 乗算 | `a * b` |
| `/` | 除算 | `a / b` |
| `%` | 剰余 | `a % b` |

### 比較演算子

| 演算子 | 説明 | 例 |
|----------|-------------|---------|
| `==` | 等しい | `a == b` |
| `!=` | 等しくない | `a != b` |
| `<` | より小さい | `a < b` |
| `>` | より大きい | `a > b` |
| `<=` | 以下 | `a <= b` |
| `>=` | 以上 | `a >= b` |

### 論理演算子

| 演算子 | 説明 | 例 |
|----------|-------------|---------|
| `&` | ビット単位AND | `a & b` |
| `\|` | ビット単位OR | `a \| b` |
| `^` | ビット単位XOR | `a ^ b` |
| `!` | 論理NOT | `!x` |
| `~` | ビット単位NOT | `~x` |
| `<<` | 左シフト | `a << 2` |
| `>>` | 右シフト | `a >> 2` |

### 代入演算子

| 演算子 | 説明 | 例 |
|----------|-------------|---------|
| `=` | 代入 | `x = 10` |
| `:=` | 型推論代入 | `x := 10` |
| `+=` | 加算して代入 | `x += 5` |
| `-=` | 減算して代入 | `x -= 5` |
| `*=` | 乗算して代入 | `x *= 2` |
| `/=` | 除算して代入 | `x /= 2` |

### 特殊演算子

| 演算子 | 説明 | 例 |
|----------|-------------|---------|
| `@` | 自己再帰 | `@(n-1)` |
| `?` | 三項条件またはTry演算子 | `x > 0 ? 1 : -1` または `file.read()?` |
| `!` | 論理NOTまたはUnwrap演算子 | `!x` または `option!` |
| `.` | メンバーアクセス | `point.x` |
| `::` | パス区切り | `std::math::PI` |
| `->` | 関数戻り値型 | `F add()->i64` |
| `=>` | マッチアーム区切り | `0 => "zero"` |
| `..` | 範囲(排他的)/スプレッド | `0..10`, `[..arr]` |
| `..=` | 範囲(包括的) | `0..=10` |
| `\|>` | パイプ演算子 | `x \|> f \|> g` (`g(f(x))`と等価) |

**注: `?`演算子について:** `?`演算子には2つの使用法があります:
- **三項条件**: `condition ? true_val : false_val`
- **Try演算子**: `result?` - エラーを呼び出し元に伝播([エラーハンドリング](#エラーハンドリング)を参照)

### 演算子優先順位

演算子は優先順位の高いものから低いものへリストされています:

| 優先順位 | 演算子 | 説明 |
|------------|-----------|-------------|
| 1 (最高) | `.`, `[]`, `()` | メンバーアクセス、インデックス、呼び出し |
| 2 | `-`, `!`, `~`, `@` | 単項演算子 |
| 3 | `*`, `/`, `%` | 乗算、除算、剰余 |
| 4 | `+`, `-` | 加算、減算 |
| 5 | `<<`, `>>` | ビットシフト |
| 6 | `&` | ビット単位AND |
| 7 | `^` | ビット単位XOR |
| 8 | `\|` | ビット単位OR |
| 9 | `==`, `!=`, `<`, `>`, `<=`, `>=` | 比較 |
| 10 | `&&` | 論理AND |
| 11 | `\|\|` | 論理OR |
| 12 | `?:`, `\|>` | 三項条件、パイプ |
| 13 (最低) | `=`, `:=`, `+=`, `-=`, `*=`, `/=` | 代入 |

**注:** ビット単位`&`は`==`のような比較演算子より優先順位が高いです。明確にするために括弧を使用してください: `(a == b) & (c == d)`。

---

## 型

### プリミティブ型

**整数型:**
- `i8`, `i16`, `i32`, `i64`, `i128` - 符号付き整数
- `u8`, `u16`, `u32`, `u64`, `u128` - 符号なし整数

**浮動小数点型:**
- `f32` - 32ビット浮動小数点
- `f64` - 64ビット浮動小数点

**その他の型:**
- `bool` - ブール型 (`true`または`false`)
- `str` - 文字列型

### ポインタ型

```vais
*i64        # i64へのポインタ
*type          # 型Tへのポインタ
```

### 配列型

```vais
[i64]       # i64の配列
[type]         # 型Tの配列
```

### ジェネリック型

```vais
Option<T>   # ジェネリックOption型
Vec<T>      # ジェネリックベクター型
Pair<A, B>  # 複数の型パラメータ
```

---

## 式

Vaisのすべては値を返す式です。

### リテラル

```vais
42          # 整数
3.14        # 浮動小数点
"hello"     # 文字列
true        # ブール
```

### 変数参照

```vais
x
my_variable
```

### 二項式

```vais
a + b
x * y - z
a == b
```

### 単項式

```vais
-x
!flag
~bits
```

### 関数呼び出し

```vais
add(1, 2)
compute(x, y, z)
obj.method()
```

### 三項条件

```vais
condition ? true_value : false_value
x > 0 ? x : -x    # 絶対値
```

### 配列/インデックスアクセス

```vais
arr[0]
data[i * 2 + 1]
```

### メンバーアクセス

```vais
point.x
counter.value
obj.method()
```

### 自己再帰

`@`演算子は現在の関数を再帰的に呼び出します:

```vais
fn fib(n:i64)->i64 = n<2 ? n : @(n-1) + @(n-2)
```

次と等価:
```vais
fn fib(n:i64)->i64 = n<2 ? n : fib(n-1) + fib(n-2)
```

### パイプ演算子

`|>`演算子は左辺の値を右辺の関数の最初の引数として渡します:

```vais
# x |> f は f(x) と等価
result := 5 |> double |> add_one

# 複数の変換をチェーン
fn double(x: i64) -> i64 = x * 2
fn add_one(x: i64) -> i64 = x + 1

fn main() -> i64 = 5 |> double |> add_one  # 11
```

### 文字列補間

`{expr}`で文字列リテラル内に式を埋め込み:

```vais
name := "World"
println("Hello, {name}!")          # 変数
println("合計: {2 + 3}")           # 式
println("エスケープ: {{括弧}}")    # リテラル括弧は {{ }}
```

### タプル分解

タプル値を複数の変数に展開:

```vais
(a, b) := get_pair()
(x, y, z) := (1, 2, 3)
```

### ブロック式

ブロックは最後の式の値を返す式です:

```vais
{
    x := 10
    y := 20
    x + y    # 30を返す
}
```

### 自動リターン

Vaisの関数は自動的に最後の式の値を返します。早期リターンが必要な場合を除き、明示的な`R`(return)は不要です:

```vais
fn add(a: i64, b: i64) -> i64 {
    a + b    # 自動的に返される
}

fn max(a: i64, b: i64) -> i64 {
    I a > b {
        a    # 各分岐は最後の式を返す
    } else {
        b
    }
}

# 明示的なRは早期リターンにのみ必要
fn safe_divide(a: i64, b: i64) -> i64 {
    I b == 0 {
        return 0    # 早期リターン
    }
    a / b      # 自動リターン
}
```

これは`I`/`E`、`M`、`L`を含むすべてのブロック式に適用されます。

---

## 文

### 変数宣言

```vais
# 型推論(イミュータブル)
x := 10

# 明示的な型
y: i64 = 20

# ミュータブル
z := mut 30
```

### If-Else式

```vais
# 単一行三項
result := x > 0 ? 1 : -1

# ブロック形式
I x < 0 {
    -1
} else {
    0
}

# Else-ifチェーン
I x < 0 {
    -1
} else I x == 0 {
    0
} else {
    1
}
```

注: `E`はif式の"else"に使用されます。

### ループ式

```vais
# 無限ループ
L {
    # ... 本体
    B  # Break
}

# 範囲ループ
L i: 0..10 {
    puts("反復")
}

# 配列の反復(概念的)
L item: array {
    # ... itemを処理
}
```

### マッチ式

```vais
match value {
    0 => "zero",
    1 => "one",
    2 => "two",
    _ => "other"    # ワイルドカード/デフォルト
}

# 変数バインディング付き
match option {
    Some(x) => x,
    None => 0
}
```

### BreakとContinue

```vais
L i: 0..100 {
    I i == 50 { B }      # Break
    I i % 2 == 0 { C }   # Continue
    process(i)
}
```

### Return文

```vais
fn compute(x: i64) -> i64 {
    I x < 0 {
        return 0    # 早期リターン
    }
    x * 2
}
```

---

## 関数

### 関数定義

**式形式(単一式):**
```vais
fn add(a:i64, b:i64)->i64 = a + b
```

**ブロック形式:**
```vais
fn factorial(n:i64)->i64 {
    I n < 2 {
        1
    } else {
        n * @(n-1)
    }
}
```

### パラメータ

```vais
fn example(x: i64, y: f64, name: str) -> i64 { ... }
```

### パラメータ型推論

パラメータ型は呼び出し元から推論できる場合、省略できます:

```vais
# 使用法から型が推論される
fn add(a, b) = a + b

# 混合: 一部明示的、一部推論
fn scale(x, factor: f64) -> f64 = x * factor

# コンパイラは関数の呼び出し方から型を推論
fn main() -> i64 {
    add(1, 2)       # a: i64, b: i64が推論される
    scale(3.0, 2.0)  # x: f64が推論される
    0
}
```

### 戻り値型

```vais
fn returns_int() -> i64 { 42 }
fn returns_nothing() -> i64 { 0 }  # 慣例: voidには0
```

### ジェネリック関数

```vais
fn identity<T>(x: T) -> T = x

fn swap<A, B>(a: A, b: B) -> (B, A) {
    (b, a)
}
```

### 自己再帰

```vais
fn fib(n:i64)->i64 = n<2 ? n : @(n-1) + @(n-2)
fn countdown(n:i64)->i64 = n<1 ? 0 : @(n-1)
```

### 外部関数

`N F`でC関数を宣言:

```vais
N fn puts(s: i64) -> i64
N fn malloc(size: i64) -> i64
N fn sqrt(x: f64) -> f64
```

---

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
} else {
    puts("negative or zero")
}

# 式としてのif
sign := I x > 0 { 1 } else I x < 0 { -1 } else { 0 }
```

### match式

```vais
match x {
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

### 構造体定義

```vais
struct Point {
    x: f64,
    y: f64
}

struct Person {
    name: str,
    age: i64
}
```

### ジェネリック構造体

```vais
struct Pair<T> {
    first: T,
    second: T
}

struct Container<K, V> {
    key: K,
    value: V
}
```

### 構造体のインスタンス化

```vais
p := Point { x: 10.0, y: 20.0 }
person := Person { name: "Alice", age: 30 }
pair := Pair { first: 1, second: 2 }
```

### フィールドアクセス

```vais
x_coord := p.x
person_age := person.age
```

### メソッド

```vais
impl Point {
    fn distance(self) -> f64 {
        sqrt(self.x * self.x + self.y * self.y)
    }

    fn translate(self, dx: f64, dy: f64) -> Point {
        Point { x: self.x + dx, y: self.y + dy }
    }
}
```

---

## 列挙型

### 列挙型定義

**シンプルな列挙型:**
```vais
enum Color {
    Red,
    Green,
    Blue
}
```

**データ付き列挙型:**
```vais
enum Option {
    None,
    Some(i64)
}

enum Result {
    Ok(i64),
    Err(str)
}
```

### 列挙型の使用

```vais
color := Red
opt := Some(42)
err := Err("file not found")
```

### 列挙型実装ブロック

列挙型は構造体と同様にメソッドを持つことができます:

```vais
enum Color {
    Red,
    Green,
    Blue
}

impl Color {
    fn is_warm(&self) -> bool {
        match self {
            Red => true,
            Green => false,
            Blue => false,
            _ => false
        }
    }

    fn to_hex(&self) -> str {
        match self {
            Red => "#FF0000",
            Green => "#00FF00",
            Blue => "#0000FF",
            _ => "#000000"
        }
    }
}

# 使用法
fn main() -> i64 {
    c := Red
    I c.is_warm() {
        puts("これは暖色です")
    }
    puts(c.to_hex())
    0
}
```

---

## パターンマッチング

```vais
enum Option<T> { Some(T), None }

fn unwrap_or<T>(opt: Option<T>, default: T) -> T {
    match opt {
        Some(v) => v,
        None => default
    }
}
```

## トレイト

```vais
# トレイトを定義
trait Printable {
    fn print(self)
}

# トレイトを実装
struct Person { name: str, age: i64 }

impl Person: Printable {
    fn print(self) {
        puts(self.name)
    }
}
```

## ジェネリクス

```vais
# ジェネリック構造体
struct Box<T> {
    value: T
}

# ジェネリック関数
fn swap<T>(a: T, b: T) -> (T, T) {
    (b, a)
}

# ジェネリックトレイト
trait Container<T> {
    fn get(self) -> T
}
```

## エラーハンドリング

Vaisは従来のtry-catchブロックなしで、Result/Optionベースのエラーハンドリングシステムを使用します。エラーハンドリングは`?`(try)と`!`(unwrap)演算子を通じて行われます。

### `?`演算子(エラー伝播)

`?`演算子はエラーを呼び出し元に伝播するために使用されます。`Result<T, E>`または`Option<T>`に適用すると:
- `Ok(value)`または`Some(value)`の場合、内部の値を返す
- `Err(e)`または`None`の場合、呼び出し元関数にエラー/Noneを早期リターンする

```vais
enum Result<T, E> {
    Ok(T),
    Err(E)
}

fn read_file(path: str) -> Result<str, str> {
    # openが失敗した場合、すぐにエラーを伝播
    file := open(path)?

    # readが失敗した場合、エラーを伝播
    data := file.read()?

    # 成功の場合
    Ok(data)
}

fn process() -> Result<i64, str> {
    # ?演算子は自動的にエラーを伝播
    content := read_file("config.txt")?

    # エラーがない場合、処理を続行
    Ok(parse(content))
}
```

### `!`演算子(Unwrap)

`!`演算子は`Result`または`Option`から値を強制的に抽出します。値が`Err`または`None`の場合、プログラムはパニックします:

```vais
# Optionをunwrap - Noneの場合パニック
value := some_option!

# Resultをunwrap - Errの場合パニック
data := some_result!

# 使用例
fn main() -> i64 {
    config := read_file("config.txt")!  # 失敗時パニック
    process(config)
    0
}
```

### Option型

```vais
enum Option<T> { Some(T), None }

fn find(arr: &[i64], target: i64) -> Option<i64> {
    # ... 検索ロジック
    Some(index)  # またはNone
}
```

### Result型

```vais
enum Result<T, E> { Ok(T), Err(E) }

fn divide(a: i64, b: i64) -> Result<i64, str> {
    I b == 0 { return Err("division by zero") }
    Ok(a / b)
}
```

### エラーハンドリングパターン

**パターンマッチによるエラー処理:**

```vais
fn handle_result(r: Result<i64, str>) -> i64 {
    match r {
        Ok(value) => value,
        Err(msg) => {
            puts("エラー: ")
            puts(msg)
            0
        }
    }
}
```

---

## トレイトと実装

### トレイト定義

トレイトは型が実装できるインターフェースを定義します:

```vais
trait Printable {
    fn print(&self) -> i64
}

trait Comparable {
    fn compare(&self, other: &Self) -> i64
}
```

### トレイト実装

```vais
struct Counter {
    value: i64
}

# CounterにPrintableトレイトを実装
impl Counter: Printable {
    fn print(&self) -> i64 {
        puts("Counter値: ")
        print_i64(self.value)
        0
    }
}
```

### メソッド追加

トレイトなしでメソッドを追加:

```vais
impl Counter {
    fn increment(&self) -> i64 {
        self.value + 1
    }

    fn reset() -> Counter {
        Counter { value: 0 }
    }
}
```

---

## Async/Await

Vaisは非同期プログラミングのためにasync/awaitをサポートしています。

### 非同期関数

`A`で関数を非同期としてマーク:

```vais
A fn fetch_data(url: str) -> str {
    # 非同期操作
    "データ"
}

A fn process(id: i64) -> i64 {
    id * 2
}
```

### 非同期関数の待機

`await`または`Y`で非同期結果を待機:

```vais
fn main() -> i64 {
    # awaitを使用
    data := fetch_data("http://example.com").await

    # または省略形Yを使用
    result := process(42).Y

    0
}
```

### 並行タスクの生成

`spawn`で並行タスクを生成:

```vais
A fn task1() -> i64 {
    puts("タスク1")
    100
}

A fn task2() -> i64 {
    puts("タスク2")
    200
}

fn main() -> i64 {
    t1 := spawn task1()
    t2 := spawn task2()

    r1 := t1.await
    r2 := t2.await

    r1 + r2  # 300
}
```

---

## モジュールシステム

### モジュールのインポート

`U`キーワードでモジュールをインポート:

```vais
# 標準ライブラリモジュール
use std/io
use std/vec
use std/hashmap

# モジュールからアイテムを使用
v := Vec::new()
content := read_file("data.txt")
```

### モジュールパス

```vais
# ネストされたモジュール
use std/collections/vec
use std/fs/file

# 複数のインポート
use std/io
use std/net
use std/thread
```

---

## コメント

```vais
# 単一行コメント

fn main() {
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
fn fib(n: i64) -> i64 {
    I n <= 1 { return n }
    @(n-1) + @(n-2)
}
```

### 連結リスト

```vais
struct Node<T> {
    value: T,
    next: Option<Box<Node<T>>>
}

impl Node<T> {
    fn new(value: T) -> Node<T> {
        Node { value: value, next: None }
    }
}
```

### エラーハンドリング

```vais
fn parse_number(s: str) -> Result<i64, str> {
    # パースロジック
    I is_valid {
        Ok(number)
    } else {
        Err("Invalid number")
    }
}
```

## さらに学ぶ

- [チュートリアル](../getting-started/tutorial.md) - ステップバイステップガイド
- [標準ライブラリ](https://github.com/vaislang/vais/tree/main/std) - 組み込みモジュール
- [サンプル](https://github.com/vaislang/vais/tree/main/examples) - コードサンプル
