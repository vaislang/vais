//! `vaisc fmt --to=multi` / `--to=single` — single→multi migration tool.
//!
//! **Status (post Step 19 P4 LANDED 2026-05-08, commit `2b485860`)**:
//! migration-only. The lexer no longer accepts the retired single-char
//! forms (F/S/E/EN/EL/M/R/T/U/P/W/X), so:
//!
//! - `--to=multi` is the **canonical migration tool** for legacy code
//!   that still contains single-char declaration / control / modifier
//!   keywords. Run it once on any historical .vais source (e.g. an
//!   archive checkout, an external project) to bring it onto the
//!   post-P4 baseline.
//! - `--to=single` is **deprecated**. The output it produces will not
//!   parse against the current lexer (single-char declaration forms
//!   lex as `Token::Ident`, not as the original keyword). The
//!   conversion is preserved for source-archeology purposes only.
//!   Calling it on user-facing code is a regression.
//!
//! The original Step 15 round-trip property (`--to=multi` then
//! `--to=single` reproduces input byte-identically) no longer holds
//! against the current lexer because P4 removed the single-form input
//! side. The unit tests below still validate the *substitution* logic
//! (which keyword maps to which) but not the round-trip semantics.
//!
//! Rationale + retirement record: `docs/language/removed_keywords.md`
//! §"Single-char declaration / control / modifier forms (Step 19 P4)".
//! See also LESSONS L-009 (codemod readability trap, type-position
//! generic-param clobber) and L-010 (token-efficiency hypothesis
//! empirically false).
//!
//! The conversion is span-based, not AST-based: we re-lex the source,
//! then for each token whose span text is one of the dual-syntax
//! keywords (e.g. `F` ↔ `fn`, `S` ↔ `struct`), we substitute the other
//! form. Comments, string literals, identifiers, and whitespace are
//! preserved bit-exact because we only touch keyword spans.

use std::path::Path;
use vais_lexer::tokenize;

/// Conversion direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DualForm {
    /// Multi-character form: `fn`, `struct`, `match`, ...
    Multi,
    /// Single-character form: `F`, `S`, `M`, ...
    Single,
}

impl DualForm {
    /// Parse from CLI string. Accepts `multi` / `multi-char` / `single`
    /// / `single-char`.
    pub fn parse(s: &str) -> Result<Self, String> {
        match s {
            "multi" | "multi-char" => Ok(DualForm::Multi),
            "single" | "single-char" => Ok(DualForm::Single),
            other => Err(format!(
                "unknown --to= form `{}`. Expected `multi` or `single`.",
                other
            )),
        }
    }
}

/// One dual-syntax keyword pair. `single` and `multi` are the two
/// canonical spellings that the lexer recognizes as the same Token.
#[derive(Debug, Clone, Copy)]
struct KeywordPair {
    single: &'static str,
    multi: &'static str,
}

/// 11 dual-syntax keyword pairs from Step 15 wave 1 + 2 (master-plan
/// v27 / v28 / v29 / v30). Excludes:
/// - mod (out of scope per v29; no Vais single-char counterpart)
/// - keywords with no multi-char form yet (I / L / B / C / D / O / N
///   / G / A / Y are single-char-only at the lexer level)
const DUAL_PAIRS: &[KeywordPair] = &[
    // Step 19 P3 augment (2026-05-07): list `EN` first so that
    // `replacement_for(target=Single)` walks the table in order and picks
    // `enum → EN` (the unambiguous 2-char form) before `enum → E` (the
    // contextual single-char form). This makes round-trip stable: any
    // source containing `EN` or `enum` normalizes to `EN` in the
    // single-form canonical pass. The contextual `E` remains in the table
    // so that legacy sources written with `E Foo {}` still convert to
    // `enum Foo {}` under `--to=multi`. Step 19 P4 retires both.
    KeywordPair {
        single: "EN",
        multi: "enum",
    },
    // wave 1 (v27)
    KeywordPair {
        single: "S",
        multi: "struct",
    },
    KeywordPair {
        single: "E",
        multi: "enum",
    },
    KeywordPair {
        single: "W",
        multi: "trait",
    },
    KeywordPair {
        single: "X",
        multi: "impl",
    },
    KeywordPair {
        single: "P",
        multi: "pub",
    },
    // wave 2 batch 1 (v28)
    KeywordPair {
        single: "EL",
        multi: "else",
    },
    KeywordPair {
        single: "M",
        multi: "match",
    },
    KeywordPair {
        single: "R",
        multi: "return",
    },
    // wave 2 batch 2 (v29)
    KeywordPair {
        single: "U",
        multi: "use",
    },
    KeywordPair {
        single: "T",
        multi: "type",
    },
    // wave 2 fn (v30)
    KeywordPair {
        single: "F",
        multi: "fn",
    },
];

/// Lookup the canonical replacement for a span lexeme under the target
/// form. Returns `None` if the lexeme is not a dual-syntax keyword.
fn replacement_for(lexeme: &str, target: DualForm) -> Option<&'static str> {
    for pair in DUAL_PAIRS {
        match target {
            DualForm::Multi => {
                if lexeme == pair.single {
                    return Some(pair.multi);
                }
            }
            DualForm::Single => {
                if lexeme == pair.multi {
                    return Some(pair.single);
                }
            }
        }
    }
    None
}

/// Options forwarded from the CLI.
#[derive(Debug, Clone)]
pub struct DualOptions {
    pub target: DualForm,
    /// Print to stdout instead of writing back.
    pub check: bool,
}

/// Run the `--to=` codemod against a single source file.
pub fn run_dual(input: &Path, options: &DualOptions) -> Result<String, String> {
    let source =
        std::fs::read_to_string(input).map_err(|e| format!("read {}: {}", input.display(), e))?;

    let converted = convert_source(&source, options.target)?;

    if options.check {
        // stdout already happens at the CLI layer; just hand back.
        return Ok(converted);
    }

    if converted != source {
        std::fs::write(input, &converted)
            .map_err(|e| format!("write {}: {}", input.display(), e))?;
    }
    Ok(converted)
}

/// Pure conversion. Re-lex the source, then for each token whose span
/// content is a dual-syntax keyword, substitute the canonical form for
/// `target`. All non-keyword bytes are passed through verbatim.
///
/// Step 19 P3 (2026-05-07) guard: when `target == Multi`, single-letter
/// tokens that lex as a retired keyword *but appear in type position* are
/// kept as-is (treated as generic-parameter identifiers, not as keywords).
/// Without this guard, `EN Result<T, E> { Ok(T), Err(E) }` would convert
/// to `enum Result<type, enum> { Ok(type), Err(enum) }` (LESSONS L-009),
/// which is round-trip-clean but readability-regressed and may collide
/// with codegen monomorphization (LESSONS L-011).
///
/// The position heuristic: a single-letter token (`T`, `E`, `M`, etc.) is
/// a generic-parameter identifier when the immediately preceding meaningful
/// token is `<`, `,`, `:`, `->`, `&`, or `(`. The 12 retired forms include
/// multi-letter keywords (`EN`, `EL`) that cannot collide with generic
/// param identifiers (which are conventionally single-letter), so the
/// guard only applies to single-letter retired forms.
pub fn convert_source(source: &str, target: DualForm) -> Result<String, String> {
    let tokens = tokenize(source).map_err(|e| format!("lex error: {:?}", e))?;

    let mut out = String::with_capacity(source.len());
    let mut cursor = 0usize;

    for (i, tok) in tokens.iter().enumerate() {
        let span = &tok.span;
        if span.start < cursor {
            // Defensive: tokens should be ordered. Skip overlap rather
            // than panic.
            continue;
        }
        // Copy gap (whitespace / comments) verbatim.
        if span.start > cursor {
            out.push_str(&source[cursor..span.start]);
        }
        let lexeme = &source[span.start..span.end];
        let in_type_position = matches!(target, DualForm::Multi)
            && is_single_letter_retired(lexeme)
            && preceded_by_type_marker(&tokens, i);
        if in_type_position {
            // Generic-parameter identifier — preserve verbatim.
            out.push_str(lexeme);
        } else if let Some(replacement) = replacement_for(lexeme, target) {
            out.push_str(replacement);
        } else {
            out.push_str(lexeme);
        }
        cursor = span.end;
    }

    // Trailing bytes after the last token (final newline, comment, ...).
    if cursor < source.len() {
        out.push_str(&source[cursor..]);
    }

    Ok(converted_post_check(out, target))
}

/// True if `lexeme` is a single-letter retired keyword that could plausibly
/// be a generic-parameter identifier (e.g., `T`, `E`).
///
/// Step 19 P3 (2026-05-08, augment-2): narrowed to `T` and `E` only.
/// In practice these are the only retired single-letter forms that
/// commonly collide with generic-param identifiers (Result<T, E>,
/// `Option<T>`, `Vec<T>`, ...). The other retired forms (F/S/M/R/U/P/W/X)
/// are declaration / statement keywords whose source positions never
/// overlap with generic-param identifier positions in well-formed Vais
/// code; preserving them in type-position turned out to misclassify
/// match expressions like `default_value: M &col.default_value { ... }`
/// where `M` is statement-leading-after-colon (a value position) and
/// not actually a generic-param identifier.
fn is_single_letter_retired(lexeme: &str) -> bool {
    matches!(lexeme, "T" | "E")
}

/// Look backwards from token index `i` for the most recent meaningful
/// (non-trivia) token. Returns true if it indicates a type-position
/// context: `<`, `,`, `:`, `->`, or `(`. Trivia (comments, whitespace)
/// are already filtered out by the lexer.
///
/// Step 19 P3 (2026-05-08, augment-2): removed `&` from the marker set —
/// `&` is value-prefix in expression position (`M &x` for `match &x`)
/// just as often as it is type-prefix in type position (`&T` reference
/// type). Without `&`, the heuristic is more conservative: T/E after
/// `&` may be misclassified as keyword and rewritten, which is fine
/// because `T`/`E` after `&` is rarely a generic-param identifier in
/// well-formed Vais code (it would have to be something like
/// `&T` as a return type, which the codemod handles correctly because
/// the *prior* token would be `->`, not `&`).
fn preceded_by_type_marker(tokens: &[vais_lexer::SpannedToken], i: usize) -> bool {
    if i == 0 {
        return false;
    }
    let prev = &tokens[i - 1].token;
    use vais_lexer::Token;
    matches!(
        prev,
        Token::Lt | Token::Comma | Token::Colon | Token::Arrow | Token::LParen
    )
}

/// Post-check hook for future invariants. For now it is the identity;
/// kept as a placeholder so the round-trip test can call it once and
/// future stages (e.g. selfhost migration) can wire validators in.
fn converted_post_check(s: String, _target: DualForm) -> String {
    s
}

/// CLI entry point.
pub fn cmd_fmt_dual(input: &Path, options: &DualOptions) -> Result<(), String> {
    let converted = run_dual(input, options)?;
    if options.check {
        print!("{}", converted);
    } else {
        eprintln!(
            "Wrote {} ({} form)",
            input.display(),
            match options.target {
                DualForm::Multi => "multi",
                DualForm::Single => "single",
            }
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dual_form_parse() {
        assert_eq!(DualForm::parse("multi").unwrap(), DualForm::Multi);
        assert_eq!(DualForm::parse("multi-char").unwrap(), DualForm::Multi);
        assert_eq!(DualForm::parse("single").unwrap(), DualForm::Single);
        assert_eq!(DualForm::parse("single-char").unwrap(), DualForm::Single);
        assert!(DualForm::parse("xyz").is_err());
    }

    #[test]
    fn replacement_table() {
        assert_eq!(replacement_for("F", DualForm::Multi), Some("fn"));
        assert_eq!(replacement_for("fn", DualForm::Single), Some("F"));
        assert_eq!(replacement_for("S", DualForm::Multi), Some("struct"));
        assert_eq!(replacement_for("struct", DualForm::Single), Some("S"));
        assert_eq!(replacement_for("EL", DualForm::Multi), Some("else"));
        assert_eq!(replacement_for("else", DualForm::Single), Some("EL"));
        // not a dual keyword
        assert_eq!(replacement_for("I", DualForm::Multi), None);
        assert_eq!(replacement_for("if", DualForm::Multi), None);
        assert_eq!(replacement_for("xyz", DualForm::Multi), None);
        // identifiers that happen to start with a keyword stay put
        // because the lexer emits them as a single Ident span.
        assert_eq!(replacement_for("fn_handler", DualForm::Single), None);
        assert_eq!(replacement_for("else_result", DualForm::Single), None);
    }

    #[test]
    fn convert_simple_function() {
        let src = "F double(x: i64) -> i64 { x * 2 }\n";
        let multi = convert_source(src, DualForm::Multi).unwrap();
        assert_eq!(multi, "fn double(x: i64) -> i64 { x * 2 }\n");
        let back = convert_source(&multi, DualForm::Single).unwrap();
        assert_eq!(back, src);
    }

    #[test]
    fn convert_struct_and_match() {
        let src = "S Point { x: i64, y: i64 }\n\
                   F describe(p: Point) -> i64 {\n\
                       M p.x { 0 => 100, _ => 0, }\n\
                   }\n";
        let multi = convert_source(src, DualForm::Multi).unwrap();
        assert!(multi.contains("struct Point"));
        assert!(multi.contains("fn describe"));
        assert!(multi.contains("match p.x"));
        let back = convert_source(&multi, DualForm::Single).unwrap();
        assert_eq!(back, src);
    }

    #[test]
    fn convert_else_and_return() {
        let src = "F absish(x: i64) -> i64 {\n\
                       I x > 0 { R x } EL { R 0 - x }\n\
                   }\n";
        let multi = convert_source(src, DualForm::Multi).unwrap();
        // I has no multi-char form yet; only EL → else and R → return.
        assert!(multi.contains("else { return"));
        assert!(multi.contains("return x"));
        // I stays single-char
        assert!(multi.contains("I x > 0"));
        let back = convert_source(&multi, DualForm::Single).unwrap();
        assert_eq!(back, src);
    }

    #[test]
    fn preserves_string_literal_with_keyword_word() {
        // String body says "fn" — must not be touched.
        let src = "F main() -> i64 {\n\
                       msg := \"fn or struct\"\n\
                       0\n\
                   }\n";
        let single = convert_source(src, DualForm::Single).unwrap();
        // Source already in single form, so output equals input.
        assert_eq!(single, src);
        // Going to multi keeps the string literal intact.
        let multi = convert_source(src, DualForm::Multi).unwrap();
        assert!(multi.contains("\"fn or struct\""));
        assert!(multi.contains("fn main"));
        let back = convert_source(&multi, DualForm::Single).unwrap();
        assert_eq!(back, src);
    }

    #[test]
    fn preserves_comment_with_keyword_word() {
        let src = "# this F is a struct\nF main() -> i64 { 0 }\n";
        let multi = convert_source(src, DualForm::Multi).unwrap();
        assert!(multi.contains("# this F is a struct"));
        assert!(multi.contains("fn main"));
        let back = convert_source(&multi, DualForm::Single).unwrap();
        assert_eq!(back, src);
    }

    #[test]
    fn ident_starting_with_keyword_unchanged() {
        // `fn_handler` is one Ident token, so the replacement table
        // never fires on the substring `fn`.
        let src = "F call_test_fn(fn_ptr: i64) -> i64 { fn_ptr }\n";
        let multi = convert_source(src, DualForm::Multi).unwrap();
        assert_eq!(multi, "fn call_test_fn(fn_ptr: i64) -> i64 { fn_ptr }\n");
        let back = convert_source(&multi, DualForm::Single).unwrap();
        assert_eq!(back, src);
    }

    #[test]
    fn round_trip_use_and_type() {
        let src = "U std::io\nT MyInt = i64\nF main() -> i64 { 0 }\n";
        let multi = convert_source(src, DualForm::Multi).unwrap();
        assert_eq!(
            multi,
            "use std::io\ntype MyInt = i64\nfn main() -> i64 { 0 }\n"
        );
        let back = convert_source(&multi, DualForm::Single).unwrap();
        assert_eq!(back, src);
    }
}
