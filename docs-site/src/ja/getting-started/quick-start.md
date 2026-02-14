# クイックスタート

数分でVaisを起動して実行できます。

## インストール

```bash
# macOS / Linux (Homebrew)
brew tap vaislang/tap && brew install vais

# またはCargoで
cargo install vaisc
```

> ソースからビルドする場合は、[インストールガイド](./installation.md)を参照してください。

## 最初のプログラム

`hello.vais`という名前のファイルを作成:

```vais
F main() {
    puts("Hello, Vais!")
}
```

## コンパイルと実行

```bash
# コンパイル
vaisc build hello.vais -o hello
./hello

# または直接実行
vaisc run hello.vais
```

**出力:**
```
Hello, Vais!
```

## 基本構文

### 変数

```vais
F main() {
    x := 42              # i64として型推論
    y := 3.14            # f64として型推論
    name := "Alice"      # strとして型推論
    flag := true         # boolとして型推論

    puts("Variables declared!")
}
```

### 関数

```vais
F add(a: i64, b: i64) -> i64 {
    a + b  # 最後の式が戻り値
}

F main() {
    result := add(10, 20)
    print_i64(result)  # 出力: 30
}
```

### 制御フロー

```vais
F main() {
    x := 10

    # if式
    msg := I x > 5 { "big" } E { "small" }
    puts(msg)

    # ループ
    L i := 0; i < 5; i += 1 {
        print_i64(i)
    }
}
```

### 自己再帰

`@`を使用して現在の関数を呼び出す:

```vais
F factorial(n: i64) -> i64 {
    I n <= 1 { R 1 }
    n * @(n - 1)
}

F main() {
    print_i64(factorial(5))  # 出力: 120
}
```

## 次のステップ

- [チュートリアル](./tutorial.md) - Vaisを深く学ぶ
- [言語仕様](../language/language-spec.md) - 完全な構文リファレンス
- [サンプルプログラム](https://github.com/vaislang/vais/tree/main/examples) - コードサンプルを閲覧
