# チュートリアル

Vaisへようこそ！このチュートリアルでは、インストールから最初のプログラムの作成まで、Vaisプログラミングの基本をガイドします。

## 目次

1. [インストール](#インストール)
2. [Hello World](#hello-world)
3. [変数と型](#変数と型)
4. [関数](#関数)
5. [制御フロー](#制御フロー)
6. [構造体と列挙型](#構造体と列挙型)
7. [パターンマッチング](#パターンマッチング)
8. [トレイトとメソッド](#トレイトとメソッド)
9. [ジェネリクス](#ジェネリクス)
10. [標準ライブラリの基本](#標準ライブラリの基本)
11. [非同期プログラミング](#非同期プログラミング)
12. [次のステップ](#次のステップ)

---

## インストール

### 前提条件

- Rustツールチェーン(コンパイラのビルド用)
- LLVM(コード生成用)
- Clang(生成されたLLVM IRのコンパイル用)

### ソースからビルド

```bash
# リポジトリをクローン
git clone https://github.com/vaislang/vais.git
cd vais

# コンパイラをビルド
cargo build --release

# コンパイラは以下の場所で利用可能になります:
./target/release/vaisc
```

### インストールの確認

```bash
./target/release/vaisc --version
# 出力されるべき内容: Vais 1.0.0
```

---

## Hello World

最初のVaisプログラムを書いてみましょう！

### ファイル`hello.vais`を作成:

```vais
# Hello World例
F main()->i64 {
    puts("Hello, Vais!")
    0
}
```

### コンパイルして実行:

```bash
./target/release/vaisc hello.vais
./hello
```

**出力:**
```
Hello, Vais!
```

### コードの理解:

- `F` - 関数定義のキーワード
- `main` - エントリーポイント関数名
- `()->i64` - 関数シグネチャ: パラメータなし、i64を返す
- `puts("Hello, Vais!")` - 文字列を出力
- `0` - 戻り値(慣例: 成功時は0)

---

## 変数と型

### 型推論された変数

`:=`を使用して自動型推論:

```vais
F main()->i64 {
    x := 10          # i64が推論される
    y := 3.14        # f64が推論される
    name := "Alice"  # strが推論される
    flag := true     # boolが推論される

    puts("変数を宣言しました!")
    0
}
```

### 明示的な型

`:`で型を明示的に指定:

```vais
F main()->i64 {
    x: i64 = 100
    y: f64 = 2.5
    count: i32 = 42

    puts("型付き変数を宣言しました!")
    0
}
```

### プリミティブ型

**整数型:**
```vais
a: i8 = 127          # 8ビット符号付き
b: i16 = 32000       # 16ビット符号付き
c: i32 = 1000000     # 32ビット符号付き
d: i64 = 999999999   # 64ビット符号付き

ua: u8 = 255         # 8ビット符号なし
ub: u32 = 4294967295 # 32ビット符号なし
```

**浮動小数点型:**
```vais
x: f32 = 3.14        # 32ビット浮動小数点
y: f64 = 2.718281828 # 64ビット浮動小数点
```

**ブール型:**
```vais
is_ready := true
is_done := false
```

### 変数の使用

```vais
F main()->i64 {
    x := 10
    y := 20
    sum := x + y

    puts("合計を計算しました!")
    0
}
```

### ミュータブル変数

再代入可能な変数には`mut`を使用:

```vais
F main()->i64 {
    x := mut 0
    x = 10  # OK: xはミュータブル
    x = 20  # OK
    0
}
```

---

## 関数

### シンプルな関数

**式形式**(単一の式):

```vais
F add(a:i64, b:i64)->i64 = a + b

F square(x:i64)->i64 = x * x

F max(a:i64, b:i64)->i64 = a > b ? a : b
```

**ブロック形式**(複数の文):

```vais
F greet(name: str)->i64 {
    puts("Hello, ")
    puts(name)
    puts("!")
    0
}
```

### 関数パラメータ

```vais
# 異なる型を持つ複数のパラメータ
F calculate(x: i64, y: f64, multiplier: i64) -> f64 {
    result := x * multiplier
    result * y
}
```

### 関数の呼び出し

```vais
F main()->i64 {
    sum := add(10, 20)
    squared := square(5)
    maximum := max(100, 200)

    puts("関数を呼び出しました!")
    0
}
```

### `@`による自己再帰

`@`演算子は現在の関数を再帰的に呼び出します:

```vais
# 自己再帰を使用したフィボナッチ
F fib(n:i64)->i64 = n<2 ? n : @(n-1) + @(n-2)

# 階乗
F factorial(n:i64)->i64 = n<2 ? 1 : n * @(n-1)

# カウントダウン
F countdown(n:i64)->i64 {
    I n <= 0 {
        puts("完了!")
        0
    } E {
        puts("カウント中...")
        @(n-1)
    }
}
```

**なぜ`@`を使用するか?**
- 関数名を書くよりも簡潔
- AIコード生成のトークン数が少ない
- 再帰の明確な指標

### 早期リターン

```vais
F validate(x: i64)->i64 {
    I x < 0 {
        puts("エラー: 負の値")
        R -1  # 早期リターン
    }
    I x == 0 {
        puts("エラー: ゼロ値")
        R -1
    }

    # 有効な値を処理
    puts("有効!")
    x * 2
}
```

---

## 制御フロー

### If-Else式

**三項形式**(単一式):

```vais
F abs(x:i64)->i64 = x < 0 ? -x : x

F sign(x:i64)->i64 = x < 0 ? -1 : x > 0 ? 1 : 0
```

**ブロック形式:**

```vais
F classify(x:i64)->str {
    I x < 0 {
        "negative"
    } E I x == 0 {
        "zero"
    } E {
        "positive"
    }
}
```

注: `E`は"else"に使用されます。コンテキストによって`E`が"enum"または"else"を意味するか決定されます。

### ループ

**無限ループ:**

```vais
F loop_forever()->i64 {
    L {
        puts("ループ中...")
        # breakの条件が必要
    }
    0
}
```

**範囲ループ:**

```vais
F count_to_ten()->i64 {
    L i: 0..10 {
        puts("数値: ")
        print_i64(i)
        putchar(10)
    }
    0
}
```

**breakとcontinue付き:**

```vais
F find_first_even()->i64 {
    L i: 0..100 {
        I i % 2 == 0 {
            puts("偶数を発見:")
            print_i64(i)
            B  # Break
        }
        C  # Continue
    }
    0
}
```

---

## 構造体と列挙型

### 構造体の定義

```vais
S Point {
    x: f64,
    y: f64
}

S Person {
    name: str,
    age: i64
}

S Rectangle {
    top_left: Point,
    bottom_right: Point
}
```

### 構造体インスタンスの作成

```vais
F main()->i64 {
    # Pointを作成
    p := Point { x: 10.0, y: 20.0 }

    # Personを作成
    person := Person { name: "Bob", age: 25 }

    # ネストした構造体
    rect := Rectangle {
        top_left: Point { x: 0.0, y: 10.0 },
        bottom_right: Point { x: 10.0, y: 0.0 }
    }

    0
}
```

### フィールドへのアクセス

```vais
F main()->i64 {
    p := Point { x: 5.0, y: 15.0 }

    x_coord := p.x
    y_coord := p.y

    puts("Point座標:")
    print_f64(x_coord)
    print_f64(y_coord)

    0
}
```

### 列挙型の定義

**シンプルな列挙型:**

```vais
E Color {
    Red,
    Green,
    Blue
}
```

**データを持つ列挙型:**

```vais
E Option {
    None,
    Some(i64)
}

E Result {
    Ok(i64),
    Err(str)
}

E Message {
    Quit,
    Move(i64, i64),
    Write(str)
}
```

### 列挙型の使用

```vais
F main()->i64 {
    color := Red
    opt := Some(42)
    result := Ok(100)
    msg := Move(10, 20)

    puts("列挙型を作成しました!")
    0
}
```

---

## パターンマッチング

`M`(match)によるパターンマッチングは、列挙型と値を扱う強力な機能です。

### 基本的なマッチ

```vais
F describe_number(n: i64)->str {
    M n {
        0 => "zero",
        1 => "one",
        2 => "two",
        _ => "other"  # ワイルドカード: それ以外すべてにマッチ
    }
}
```

### バインディング付きマッチ

マッチしたパターンから値を抽出:

```vais
E Option {
    None,
    Some(i64)
}

F unwrap_or(opt: Option, default: i64) -> i64 {
    M opt {
        Some(x) => x,        # 値を'x'にバインド
        None => default
    }
}

F main()->i64 {
    opt1 := Some(42)
    opt2 := None

    v1 := unwrap_or(opt1, 0)  # 42を返す
    v2 := unwrap_or(opt2, 99) # 99を返す

    print_i64(v1)
    print_i64(v2)
    0
}
```

### Result型とのマッチ

```vais
E Result {
    Ok(i64),
    Err(str)
}

F handle_result(res: Result) -> i64 {
    M res {
        Ok(value) => value,
        Err(msg) => {
            puts("エラー: ")
            puts(msg)
            0
        }
    }
}
```

### 完全な例

```vais
E Color {
    Red,
    Green,
    Blue
}

F color_to_code(c: Color) -> i64 {
    M c {
        Red => 0xFF0000,
        Green => 0x00FF00,
        Blue => 0x0000FF
    }
}

F main()->i64 {
    red_code := color_to_code(Red)
    green_code := color_to_code(Green)

    puts("カラーコードを計算しました!")
    0
}
```

---

## トレイトとメソッド

### トレイトの定義

トレイトは型が実装できるインターフェースを定義します:

```vais
W Printable {
    F print(&self) -> i64
}

W Comparable {
    F compare(&self, other: &Self) -> i64
}
```

### トレイトの実装

```vais
S Counter {
    value: i64
}

# CounterにPrintableトレイトを実装
X Counter: Printable {
    F print(&self) -> i64 {
        puts("Counter値: ")
        print_i64(self.value)
        putchar(10)
        0
    }
}
```

### メソッドの追加

`X`を使用してトレイトなしでメソッドを追加:

```vais
X Counter {
    F increment(&self) -> i64 {
        self.value + 1
    }

    F double(&self) -> i64 {
        self.value * 2
    }

    F reset() -> Counter {
        Counter { value: 0 }
    }
}
```

### メソッドの使用

```vais
F main()->i64 {
    c := Counter { value: 10 }

    # トレイトメソッドを呼び出す
    c.print()

    # implメソッドを呼び出す
    inc := c.increment()
    dbl := c.double()

    puts("Incremented: ")
    print_i64(inc)
    puts("Doubled: ")
    print_i64(dbl)

    0
}
```

### 完全な例

```vais
W Shape {
    F area(&self) -> f64
}

S Circle {
    radius: f64
}

S Rectangle {
    width: f64,
    height: f64
}

X Circle: Shape {
    F area(&self) -> f64 {
        pi := 3.14159
        pi * self.radius * self.radius
    }
}

X Rectangle: Shape {
    F area(&self) -> f64 {
        self.width * self.height
    }
}

F main()->i64 {
    circle := Circle { radius: 5.0 }
    rect := Rectangle { width: 4.0, height: 6.0 }

    circle_area := circle.area()
    rect_area := rect.area()

    puts("円の面積: ")
    print_f64(circle_area)

    puts("長方形の面積: ")
    print_f64(rect_area)

    0
}
```

---

## ジェネリクス

ジェネリクスを使用すると、複数の型で動作するコードを書くことができます。

### ジェネリック関数

```vais
F identity<T>(x: T) -> T = x

F first<T>(a: T, b: T) -> T = a

F swap<A, B>(a: A, b: B) -> (B, A) {
    (b, a)
}
```

### ジェネリック構造体

```vais
S Pair<T> {
    first: T,
    second: T
}

S Box<T> {
    value: T
}

S Container<K, V> {
    key: K,
    value: V
}
```

### ジェネリック構造体の使用

```vais
F main()->i64 {
    # 整数のペア
    int_pair := Pair { first: 10, second: 20 }

    # 浮動小数点のペア
    float_pair := Pair { first: 1.5, second: 2.5 }

    # 異なる型のコンテナ
    container := Container { key: 1, value: "hello" }

    0
}
```

### ジェネリック型のメソッド

```vais
S Pair<T> {
    first: T,
    second: T
}

X Pair {
    F sum(&self) -> i64 {
        self.first + self.second
    }

    F swap(&self) -> Pair {
        Pair { first: self.second, second: self.first }
    }
}

F main()->i64 {
    p := Pair { first: 10, second: 20 }
    total := p.sum()
    swapped := p.swap()

    print_i64(total)  # 30
    0
}
```

### ジェネリック列挙型

```vais
E Option<T> {
    None,
    Some(T)
}

E Result<T, E> {
    Ok(T),
    Err(E)
}

F main()->i64 {
    # i64のOption
    opt_int := Some(42)

    # strのOption
    opt_str := Some("hello")

    # i64値とstrエラーを持つResult
    result := Ok(100)

    0
}
```

---

## 標準ライブラリの基本

### 数学ライブラリの使用

```vais
U std/math

F main()->i64 {
    # 定数
    pi := PI
    e := E

    # 基本数学
    x := abs(-10.0)          # 絶対値
    min_val := min(5.0, 10.0)
    max_val := max(5.0, 10.0)

    # 高度な数学
    root := sqrt(16.0)       # 平方根: 4.0
    power := pow(2.0, 8.0)   # 2^8 = 256.0

    # 三角関数
    sine := sin(PI / 2.0)    # sin(90°) = 1.0
    cosine := cos(0.0)       # cos(0°) = 1.0

    # 対数
    natural_log := log(E)    # ln(e) = 1.0
    log_base_10 := log10(100.0)  # 2.0

    print_f64(root)
    0
}
```

### IOライブラリの使用

```vais
U std/io

F main()->i64 {
    # 整数を読み取る
    puts("数値を入力: ")
    num := read_i64()
    puts("入力された値: ")
    print_i64(num)
    putchar(10)

    # 浮動小数点を読み取る
    puts("小数を入力: ")
    decimal := read_f64()
    puts("入力された値: ")
    print_f64(decimal)
    putchar(10)

    # プロンプト関数
    age := prompt_i64("年齢を入力: ")
    height := prompt_f64("身長を入力: ")

    puts("年齢: ")
    print_i64(age)
    puts("身長: ")
    print_f64(height)

    0
}
```

### OptionとResultの使用

```vais
U std/option
U std/result

F divide(a: i64, b: i64) -> Option {
    I b == 0 {
        None
    } E {
        Some(a / b)
    }
}

F main()->i64 {
    result := divide(10, 2)
    value := result.unwrap_or(0)  # 5を返す

    error_result := divide(10, 0)
    default_value := error_result.unwrap_or(-1)  # -1を返す

    print_i64(value)
    print_i64(default_value)
    0
}
```

---

## 非同期プログラミング

Vaisは並行プログラミングのためにasync/awaitをサポートしています。

### 非同期関数の定義

```vais
# 'A'で関数を非同期としてマーク
A F compute(x: i64) -> i64 {
    x * 2
}

A F fetch_data(id: i64) -> str {
    # 非同期操作をシミュレート
    "データがロードされました"
}
```

### 非同期関数の待機

```vais
F main()->i64 {
    # 非同期関数を呼び出して結果を待機
    result := compute(21).await

    puts("結果: ")
    print_i64(result)  # 42

    # 非同期呼び出しをチェーン
    data := fetch_data(1).await
    puts(data)

    0
}
```

### 並行タスクの生成

```vais
A F task1() -> i64 {
    puts("タスク1実行中")
    100
}

A F task2() -> i64 {
    puts("タスク2実行中")
    200
}

F main()->i64 {
    # タスクを並行実行するため生成
    t1 := spawn task1()
    t2 := spawn task2()

    # 結果を待機
    r1 := t1.await
    r2 := t2.await

    total := r1 + r2
    print_i64(total)  # 300

    0
}
```

---

## 次のステップ

### 完全な例

より完全なプログラムは`examples/`ディレクトリを探索してください:

- `fib.vais` - 自己再帰によるフィボナッチ
- `pattern_match_test.vais` - パターンマッチングの例
- `trait_test.vais` - トレイトと実装
- `generic_struct_test.vais` - ジェネリック型
- `async_test.vais` - Async/awaitの例
- `io_test.vais` - 対話型I/Oの例

### さらに詳しく

- **言語仕様**: 完全な言語リファレンスについては[言語仕様](../language/language-spec.md)を参照
- **標準ライブラリ**: 利用可能なすべてのモジュールと関数については[標準ライブラリ](../stdlib/index.md)を参照
- **REPL**: `vaisc repl`で対話型REPLを試す

### 実践プロジェクト

1. **電卓**: IOライブラリを使用してシンプルな電卓を構築
2. **ファイルプロセッサ**: `std/file`を使用してファイルを読み取り処理
3. **データ構造**: 独自のVectorやHashMapを実装
4. **非同期Webサーバー**: async/awaitを使用してシンプルなサーバーを構築

### コミュニティ

- GitHub: [https://github.com/vaislang/vais](https://github.com/vaislang/vais)
- Issues: バグ報告や機能リクエスト
- Discussions: 質問や成果の共有

---

## クイックリファレンス

### 関数定義
```vais
F name(param: type)->return_type = expr
F name(param: type)->return_type { body }
```

### 変数
```vais
x := value        # 型推論
x: type = value   # 明示的な型
```

### 制御フロー
```vais
I condition { then } E { else }
L { loop_body }
L var: range { body }
M value { pattern => expr, ... }
```

### 自己再帰
```vais
F fib(n:i64)->i64 = n<2 ? n : @(n-1) + @(n-2)
```

### 構造体
```vais
S Name { field: type, ... }
X Name { F method(&self)->type { body } }
```

### 列挙型
```vais
E Name { Variant, Variant(type), ... }
```

### トレイト
```vais
W Trait { F method(&self)->type }
X Type: Trait { F method(&self)->type { body } }
```

### ジェネリクス
```vais
F name<T>(x: T) -> T { body }
S Name<T> { field: T }
```

### 非同期
```vais
A F name() -> type { body }
result := async_func().await
```

---
