# Vais Formal Grammar

This directory contains the complete EBNF grammar for the Vais programming language,
generated from the recursive-descent parser implementation in `crates/vais-parser/src/`.

## Files

- `vais.ebnf` -- Complete grammar in ISO/IEC 14977 EBNF notation (~320 lines)

## Notation Conventions

| Syntax      | Meaning                        |
|-------------|--------------------------------|
| `{ X }`     | Zero or more repetitions of X  |
| `[ X ]`     | Optional X (zero or one)       |
| `X \| Y`    | Alternative (X or Y)           |
| `'x'`       | Terminal literal token          |
| `(* ... *)` | Comment                        |

Single-character keywords (`F`, `S`, `E`, etc.) appear as quoted terminals.
Multi-character keywords (`spawn`, `yield`, `move`, `lazy`, `force`, `comptime`, `where`,
`dyn`, `linear`, `affine`, `const`, `mut`, `macro`, `as`, `fn`, `self`, `Self`) also
appear as quoted terminals.

Each production includes a comment mapping to the parser function and source location,
e.g. `(* Parser: parse_function (item/declarations.rs:8) *)`.

## Ambiguity Resolution Rules

The Vais grammar has several context-sensitive constructs resolved by the parser:

1. **`E` = enum vs else** -- At top level, `E` is the enum keyword. Inside an if-expression,
   `E` after `}` is the else keyword. Parser: `parse_if_expr` checks `Token::Enum` for else.

2. **`C` = continue vs const** -- Inside a block, `C` is the continue statement.
   At top level, `C` is a constant definition. Parser: `parse_item` vs `parse_stmt`.

3. **Struct literal vs block** -- `Name{...}` is a struct literal only when `Name`
   starts with an uppercase letter AND `allow_struct_literal` is true.
   In loop/if/match conditions, struct literals are disabled to avoid ambiguity with
   the block body `{`. Parser: `allow_struct_literal` flag.

4. **Map literal vs block** -- `{expr: expr, ...}` is a map literal. The parser uses
   backtracking: it saves position, tries to parse `expr :`, and restores on failure.
   Parser: `parse_primary_inner`, `Token::LBrace` branch.

5. **`?` postfix try vs ternary** -- `expr?` is postfix try (unwrap Result/Option).
   `expr ? then : else` is a ternary. Disambiguation: after `?`, if the next token
   can start an expression, it is ternary; otherwise it is postfix try.
   Parser: `parse_ternary` and `parse_postfix` both check the same token set.

6. **`>>` generic split** -- In `Vec<HashMap<K,V>>`, the lexer produces `>>` as a
   single `Token::Shr`. The parser splits it into two `>` tokens using `pending_gt`.
   `consume_gt()` handles this transparently.
   Parser: `check_gt()`, `consume_gt()`, `pending_gt` flag.

7. **`+`/`-` newline** -- `+` and `-` on a new line are NOT treated as binary
   operators. This prevents `f()\n-1` from being parsed as `f() - 1`.
   Parser: `parse_term` calls `has_newline_between()`.

8. **`move` keyword** -- `move |x| ...` is a move-capture lambda. `move` in a parameter
   list is an ownership modifier. Disambiguation: if `move` is followed by `|`, it is
   a lambda capture mode; otherwise it is an ownership annotation.
   Parser: `parse_let_stmt`, `parse_params`, `parse_primary_inner`.

9. **`T` = type alias vs trait alias** -- `T Name = Type` is a type alias.
   `T Name = Trait + Trait` is a trait alias. Disambiguation: after `=`, if the RHS
   matches `Ident +`, it is a trait alias. Parser: `parse_type_or_trait_alias`.

10. **`!` postfix unwrap vs macro invoke** -- `ident!(...)` is a macro invocation.
    `expr!` (not followed by `(`, `[`, or `{`) is postfix unwrap.
    Parser: `parse_postfix`, `Token::Bang` branch.
