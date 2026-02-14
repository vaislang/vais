# チュートリアル

このチュートリアルでは、基本概念から高度な機能まで、Vaisプログラミングをガイドします。

## Hello World

古典的な最初のプログラムから始めましょう:

```vais
F main() {
    puts("Hello, Vais!")
}
```

**ポイント:**
- `F`は関数を宣言
- `main`はエントリーポイント
- `puts`は文字列を標準出力に出力

## 変数と型

### 型推論

`:=`を使用して自動型推論:

```vais
F main() {
    x := 10          # i64が推論される
    y := 3.14        # f64が推論される
    name := "Alice"  # strが推論される
    flag := true     # boolが推論される
}
```

### 明示的な型

必要に応じて型を指定:

```vais
F main() {
    count: i64 = 100
    price: f64 = 19.99
    is_active: bool = true
}
```

### ミュータブル変数

再代入可能な変数には`mut`を使用:

```vais
F main() {
    x := mut 0
    x = 10  # OK: xはミュータブル
    x = 20  # OK
}
```

## 関数

### 基本的な関数

```vais
F add(a: i64, b: i64) -> i64 {
    a + b  # 最後の式が戻り値
}

F greet(name: str) {
    puts("Hello, ")
    puts(name)
}
```

### 早期リターン

`R`を使用して早期に戻る:

```vais
F abs(x: i64) -> i64 {
    I x < 0 { R -x }
    x
}
```

### 自己再帰

`@`を使用して現在の関数を呼び出す:

```vais
F factorial(n: i64) -> i64 {
    I n <= 1 { R 1 }
    n * @(n - 1)
}

F fibonacci(n: i64) -> i64 {
    I n <= 1 { R n }
    @(n-1) + @(n-2)
}
```

## 制御フロー

### if式

Vaisではすべてが式です:

```vais
F main() {
    x := 10

    # ifは値を返す
    result := I x > 5 { "big" } E { "small" }
    puts(result)  # 出力: big
}
```

### ループ

```vais
F main() {
    # Cスタイルのループ
    L i := 0; i < 10; i += 1 {
        print_i64(i)
    }

    # breakを使った無限ループ
    counter := mut 0
    L {
        counter = counter + 1
        I counter >= 5 { B }
    }
}
```

## 構造体

カスタムデータ型を定義:

```vais
S Point {
    x: f64,
    y: f64
}

F main() {
    p := Point { x: 3.0, y: 4.0 }
    puts("Point created")
}
```

### メソッド

構造体のメソッドを実装:

```vais
S Point { x: f64, y: f64 }

X Point {
    F distance(self) -> f64 {
        sqrt(self.x * self.x + self.y * self.y)
    }
}

F main() {
    p := Point { x: 3.0, y: 4.0 }
    d := p.distance()
    print_f64(d)  # 出力: 5.0
}
```

## 列挙型

バリアント型を定義:

```vais
E Color {
    Red,
    Green,
    Blue
}

E Option<T> {
    Some(T),
    None
}
```

## パターンマッチング

`M`を使用してパターンをマッチ:

```vais
E Color { Red, Green, Blue }

F color_name(c: Color) -> str {
    M c {
        Red => "red",
        Green => "green",
        Blue => "blue"
    }
}

F main() {
    c := Red
    puts(color_name(c))  # 出力: red
}
```

## エラーハンドリング

### ResultとOption型

```vais
E Result<T, E> {
    Ok(T),
    Err(E)
}

F divide(a: i64, b: i64) -> Result<i64, str> {
    I b == 0 { R Err("Division by zero") }
    Ok(a / b)
}
```

### Try演算子

`?`を使用してエラーを伝播:

```vais
F compute() -> Result<i64, str> {
    x := divide(10, 2)?  # Errの場合は伝播
    y := divide(x, 0)?   # ここでErrを返す
    Ok(y)
}
```

### Unwrap演算子

`!`を使用してアンラップまたはパニック:

```vais
F main() {
    result := divide(10, 2)
    value := result!  # Ok値をアンラップ、Errでパニック
    print_i64(value)
}
```

## ジェネリクス

任意の型で動作するコードを記述:

```vais
F identity<T>(x: T) -> T {
    x
}

S Box<T> {
    value: T
}

F main() {
    x := identity(42)      # T = i64
    y := identity(3.14)    # T = f64

    b := Box { value: 100 }
}
```

## トレイト

共有の振る舞いを定義:

```vais
W Printable {
    F print(self)
}

S Point { x: f64, y: f64 }

X Point: Printable {
    F print(self) {
        puts("Point(")
        print_f64(self.x)
        puts(", ")
        print_f64(self.y)
        puts(")")
    }
}
```

## 標準ライブラリ

### コレクション

```vais
U std/vec
U std/hashmap

F main() {
    # Vector
    v := Vec::new()
    v.push(1)
    v.push(2)
    v.push(3)

    # HashMap
    m := HashMap::new()
    m.insert("name", "Alice")
    m.insert("city", "Paris")
}
```

### ファイルI/O

```vais
U std/io

F main() {
    # ファイルを読む
    content := read_file("data.txt")
    puts(content)

    # ファイルに書く
    write_file("output.txt", "Hello, file!")
}
```

## 次のステップ

Vaisの基本を習得しました！学習を続けるには:

- [言語仕様](../language/language-spec.md) - 完全な構文リファレンス
- [標準ライブラリ](https://github.com/vaislang/vais/tree/main/std) - 組み込みモジュールを探索
- [サンプル](https://github.com/vaislang/vais/tree/main/examples) - 実世界のコードサンプル
