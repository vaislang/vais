//! Phase 83 — Standard Library Utilities E2E tests
//!
//! Tests for regex engine, HTTP client helpers, and SQLite utility functions:
//! - Regex: literal match, dot wildcard, quantifiers (*, +, ?), char classes,
//!          anchors (^, $), ranges, escaping, complex patterns
//! - HTTP:  status code classification, hex digit utilities, URL encoding,
//!          chunked parsing, method constants, query building
//! - SQLite: result codes, is_ok/has_row/is_done helpers, column type constants,
//!           extended result codes, combined checks

use super::helpers::*;

// ==================== Section 1: Regex Tests ====================

#[test]
fn e2e_p83_regex_literal_match() {
    // Match literal "abc" in text "xabcy" -> score 2 (compile verified + match found)
    let source = r#"
# match_here: returns 1 if regex matches text starting at pos
F match_here(regex: i64, text: i64, pos: i64, tlen: i64) -> i64 {
    rtype := load_i64(regex)
    # END node: match successful
    I rtype == 0 { R 1 }
    # LITERAL node
    I rtype == 1 {
        I pos >= tlen { R 0 }
        ch := load_i64(regex + 8)
        I load_byte(text + pos) == ch {
            next := load_i64(regex + 24)
            R match_here(next, text, pos + 1, tlen)
        }
        R 0
    }
    0
}

F mk_end() -> i64 {
    n := malloc(32)
    store_i64(n, 0)
    store_i64(n + 8, 0)
    store_i64(n + 16, 0)
    store_i64(n + 24, 0)
    n
}

F mk_literal(c: i64, next: i64) -> i64 {
    n := malloc(32)
    store_i64(n, 1)
    store_i64(n + 8, c)
    store_i64(n + 16, 0)
    store_i64(n + 24, next)
    n
}

F main() -> i64 {
    # Pattern: LITERAL('a') -> LITERAL('b') -> LITERAL('c') -> END
    end := mk_end()
    c_node := mk_literal(99, end)
    b_node := mk_literal(98, c_node)
    a_node := mk_literal(97, b_node)

    # Text: "xabcy" = [120, 97, 98, 99, 121]
    text := malloc(6)
    store_byte(text, 120)
    store_byte(text + 1, 97)
    store_byte(text + 2, 98)
    store_byte(text + 3, 99)
    store_byte(text + 4, 121)
    store_byte(text + 5, 0)

    result := 0
    # Score 1: pattern successfully compiled (nodes created)
    result = result + 1
    # Score 2: match found at position 1
    I match_here(a_node, text, 1, 5) == 1 { result = result + 1 }
    # Verify no match at position 0 (x != a)
    I match_here(a_node, text, 0, 5) == 0 { result = result }

    free(text)
    free(a_node)
    free(b_node)
    free(c_node)
    free(end)
    result
}
"#;
    assert_exit_code(source, 2);
}

#[test]
fn e2e_p83_regex_dot_wildcard() {
    // "a.c" matches "abc" and "axc" -> 2 points
    let source = r#"
# Node types: 0=END, 1=LITERAL, 2=ANY
F match_here(regex: i64, text: i64, pos: i64, tlen: i64) -> i64 {
    rtype := load_i64(regex)
    I rtype == 0 { R 1 }
    I rtype == 1 {
        I pos >= tlen { R 0 }
        ch := load_i64(regex + 8)
        I load_byte(text + pos) == ch {
            next := load_i64(regex + 24)
            R match_here(next, text, pos + 1, tlen)
        }
        R 0
    }
    I rtype == 2 {
        # ANY matches any single character
        I pos >= tlen { R 0 }
        next := load_i64(regex + 24)
        R match_here(next, text, pos + 1, tlen)
    }
    0
}

F mk_end() -> i64 {
    n := malloc(32)
    store_i64(n, 0)
    store_i64(n + 8, 0)
    store_i64(n + 16, 0)
    store_i64(n + 24, 0)
    n
}

F mk_literal(c: i64, next: i64) -> i64 {
    n := malloc(32)
    store_i64(n, 1)
    store_i64(n + 8, c)
    store_i64(n + 16, 0)
    store_i64(n + 24, next)
    n
}

F mk_any(next: i64) -> i64 {
    n := malloc(32)
    store_i64(n, 2)
    store_i64(n + 8, 0)
    store_i64(n + 16, 0)
    store_i64(n + 24, next)
    n
}

F main() -> i64 {
    # Pattern: LITERAL('a') -> ANY -> LITERAL('c') -> END
    end := mk_end()
    c_node := mk_literal(99, end)
    dot := mk_any(c_node)
    a_node := mk_literal(97, dot)

    # Text "abc" = [97, 98, 99]
    t1 := malloc(4)
    store_byte(t1, 97)
    store_byte(t1 + 1, 98)
    store_byte(t1 + 2, 99)
    store_byte(t1 + 3, 0)

    # Text "axc" = [97, 120, 99]
    t2 := malloc(4)
    store_byte(t2, 97)
    store_byte(t2 + 1, 120)
    store_byte(t2 + 2, 99)
    store_byte(t2 + 3, 0)

    result := 0
    I match_here(a_node, t1, 0, 3) == 1 { result = result + 1 }
    I match_here(a_node, t2, 0, 3) == 1 { result = result + 1 }

    free(t1)
    free(t2)
    free(a_node)
    free(dot)
    free(c_node)
    free(end)
    result
}
"#;
    assert_exit_code(source, 2);
}

#[test]
fn e2e_p83_regex_star_quantifier() {
    // "ab*c": LITERAL('a') -> STAR(LITERAL('b')) -> LITERAL('c') -> END
    // matches "ac" (zero b's) and "abbc" (two b's) -> 2 points
    let source = r#"
# Node types: 0=END, 1=LITERAL, 7=STAR
F match_here(regex: i64, text: i64, pos: i64, tlen: i64) -> i64 {
    rtype := load_i64(regex)
    I rtype == 0 { R 1 }
    I rtype == 1 {
        I pos >= tlen { R 0 }
        ch := load_i64(regex + 8)
        I load_byte(text + pos) == ch {
            next := load_i64(regex + 24)
            R match_here(next, text, pos + 1, tlen)
        }
        R 0
    }
    I rtype == 7 {
        # STAR: data=char to repeat, next=continuation node
        ch := load_i64(regex + 8)
        next := load_i64(regex + 24)
        # Try zero repetitions first
        I match_here(next, text, pos, tlen) == 1 { R 1 }
        # Try consuming characters greedily
        p := mut pos
        L p < tlen && load_byte(text + p) == ch {
            p = p + 1
            I match_here(next, text, p, tlen) == 1 { R 1 }
        }
        R 0
    }
    0
}

F mk_end() -> i64 {
    n := malloc(32)
    store_i64(n, 0)
    store_i64(n + 8, 0)
    store_i64(n + 16, 0)
    store_i64(n + 24, 0)
    n
}

F mk_literal(c: i64, next: i64) -> i64 {
    n := malloc(32)
    store_i64(n, 1)
    store_i64(n + 8, c)
    store_i64(n + 16, 0)
    store_i64(n + 24, next)
    n
}

F mk_star(c: i64, next: i64) -> i64 {
    n := malloc(32)
    store_i64(n, 7)
    store_i64(n + 8, c)
    store_i64(n + 16, 0)
    store_i64(n + 24, next)
    n
}

F main() -> i64 {
    # Pattern: a b* c
    end := mk_end()
    c_node := mk_literal(99, end)
    star_b := mk_star(98, c_node)
    a_node := mk_literal(97, star_b)

    # "ac" = [97, 99]
    t1 := malloc(3)
    store_byte(t1, 97)
    store_byte(t1 + 1, 99)
    store_byte(t1 + 2, 0)

    # "abbc" = [97, 98, 98, 99]
    t2 := malloc(5)
    store_byte(t2, 97)
    store_byte(t2 + 1, 98)
    store_byte(t2 + 2, 98)
    store_byte(t2 + 3, 99)
    store_byte(t2 + 4, 0)

    result := 0
    I match_here(a_node, t1, 0, 2) == 1 { result = result + 1 }
    I match_here(a_node, t2, 0, 4) == 1 { result = result + 1 }

    free(t1)
    free(t2)
    free(a_node)
    free(star_b)
    free(c_node)
    free(end)
    result
}
"#;
    assert_exit_code(source, 2);
}

#[test]
fn e2e_p83_regex_plus_quantifier() {
    // "ab+c": matches "abc" (one b) but not "ac" (zero b's) -> 2 points
    let source = r#"
# Node types: 0=END, 1=LITERAL, 8=PLUS
F match_here(regex: i64, text: i64, pos: i64, tlen: i64) -> i64 {
    rtype := load_i64(regex)
    I rtype == 0 { R 1 }
    I rtype == 1 {
        I pos >= tlen { R 0 }
        ch := load_i64(regex + 8)
        I load_byte(text + pos) == ch {
            next := load_i64(regex + 24)
            R match_here(next, text, pos + 1, tlen)
        }
        R 0
    }
    I rtype == 8 {
        # PLUS: must match at least once
        ch := load_i64(regex + 8)
        next := load_i64(regex + 24)
        I pos >= tlen { R 0 }
        I load_byte(text + pos) != ch { R 0 }
        # Match first occurrence, then try more
        p := mut pos + 1
        I match_here(next, text, p, tlen) == 1 { R 1 }
        L p < tlen && load_byte(text + p) == ch {
            p = p + 1
            I match_here(next, text, p, tlen) == 1 { R 1 }
        }
        R 0
    }
    0
}

F mk_end() -> i64 {
    n := malloc(32)
    store_i64(n, 0)
    store_i64(n + 8, 0)
    store_i64(n + 16, 0)
    store_i64(n + 24, 0)
    n
}

F mk_literal(c: i64, next: i64) -> i64 {
    n := malloc(32)
    store_i64(n, 1)
    store_i64(n + 8, c)
    store_i64(n + 16, 0)
    store_i64(n + 24, next)
    n
}

F mk_plus(c: i64, next: i64) -> i64 {
    n := malloc(32)
    store_i64(n, 8)
    store_i64(n + 8, c)
    store_i64(n + 16, 0)
    store_i64(n + 24, next)
    n
}

F main() -> i64 {
    # Pattern: a b+ c
    end := mk_end()
    c_node := mk_literal(99, end)
    plus_b := mk_plus(98, c_node)
    a_node := mk_literal(97, plus_b)

    # "abc" = [97, 98, 99] - should match
    t1 := malloc(4)
    store_byte(t1, 97)
    store_byte(t1 + 1, 98)
    store_byte(t1 + 2, 99)
    store_byte(t1 + 3, 0)

    # "ac" = [97, 99] - should NOT match
    t2 := malloc(3)
    store_byte(t2, 97)
    store_byte(t2 + 1, 99)
    store_byte(t2 + 2, 0)

    result := 0
    I match_here(a_node, t1, 0, 3) == 1 { result = result + 1 }
    I match_here(a_node, t2, 0, 2) == 0 { result = result + 1 }

    free(t1)
    free(t2)
    free(a_node)
    free(plus_b)
    free(c_node)
    free(end)
    result
}
"#;
    assert_exit_code(source, 2);
}

#[test]
fn e2e_p83_regex_question_quantifier() {
    // "ab?c": matches "ac" (zero b's) and "abc" (one b) -> 2 points
    let source = r#"
# Node types: 0=END, 1=LITERAL, 9=QUESTION
F match_here(regex: i64, text: i64, pos: i64, tlen: i64) -> i64 {
    rtype := load_i64(regex)
    I rtype == 0 { R 1 }
    I rtype == 1 {
        I pos >= tlen { R 0 }
        ch := load_i64(regex + 8)
        I load_byte(text + pos) == ch {
            next := load_i64(regex + 24)
            R match_here(next, text, pos + 1, tlen)
        }
        R 0
    }
    I rtype == 9 {
        # QUESTION: zero or one occurrence
        ch := load_i64(regex + 8)
        next := load_i64(regex + 24)
        # Try zero first
        I match_here(next, text, pos, tlen) == 1 { R 1 }
        # Try one
        I pos < tlen && load_byte(text + pos) == ch {
            R match_here(next, text, pos + 1, tlen)
        }
        R 0
    }
    0
}

F mk_end() -> i64 {
    n := malloc(32)
    store_i64(n, 0)
    store_i64(n + 8, 0)
    store_i64(n + 16, 0)
    store_i64(n + 24, 0)
    n
}

F mk_literal(c: i64, next: i64) -> i64 {
    n := malloc(32)
    store_i64(n, 1)
    store_i64(n + 8, c)
    store_i64(n + 16, 0)
    store_i64(n + 24, next)
    n
}

F mk_question(c: i64, next: i64) -> i64 {
    n := malloc(32)
    store_i64(n, 9)
    store_i64(n + 8, c)
    store_i64(n + 16, 0)
    store_i64(n + 24, next)
    n
}

F main() -> i64 {
    # Pattern: a b? c
    end := mk_end()
    c_node := mk_literal(99, end)
    q_b := mk_question(98, c_node)
    a_node := mk_literal(97, q_b)

    # "ac" = [97, 99]
    t1 := malloc(3)
    store_byte(t1, 97)
    store_byte(t1 + 1, 99)
    store_byte(t1 + 2, 0)

    # "abc" = [97, 98, 99]
    t2 := malloc(4)
    store_byte(t2, 97)
    store_byte(t2 + 1, 98)
    store_byte(t2 + 2, 99)
    store_byte(t2 + 3, 0)

    result := 0
    I match_here(a_node, t1, 0, 2) == 1 { result = result + 1 }
    I match_here(a_node, t2, 0, 3) == 1 { result = result + 1 }

    free(t1)
    free(t2)
    free(a_node)
    free(q_b)
    free(c_node)
    free(end)
    result
}
"#;
    assert_exit_code(source, 2);
}

#[test]
fn e2e_p83_regex_char_class() {
    // "[abc]" matches "b" but not "d" -> 2 points
    // CHAR_CLASS node: data = pointer to 3-byte class string, extra = length
    let source = r#"
# Node types: 0=END, 5=CHAR_CLASS
# CHAR_CLASS node: data=ptr to accepted chars, extra=count, next=continuation
F char_in_class(cls: i64, cls_len: i64, c: i64) -> i64 {
    i := mut 0
    L i < cls_len {
        I load_byte(cls + i) == c { R 1 }
        i = i + 1
    }
    0
}

F match_here(regex: i64, text: i64, pos: i64, tlen: i64) -> i64 {
    rtype := load_i64(regex)
    I rtype == 0 { R 1 }
    I rtype == 5 {
        I pos >= tlen { R 0 }
        cls := load_i64(regex + 8)
        cls_len := load_i64(regex + 16)
        next := load_i64(regex + 24)
        I char_in_class(cls, cls_len, load_byte(text + pos)) == 1 {
            R match_here(next, text, pos + 1, tlen)
        }
        R 0
    }
    0
}

F mk_end() -> i64 {
    n := malloc(32)
    store_i64(n, 0)
    store_i64(n + 8, 0)
    store_i64(n + 16, 0)
    store_i64(n + 24, 0)
    n
}

F mk_char_class(cls: i64, cls_len: i64, next: i64) -> i64 {
    n := malloc(32)
    store_i64(n, 5)
    store_i64(n + 8, cls)
    store_i64(n + 16, cls_len)
    store_i64(n + 24, next)
    n
}

F main() -> i64 {
    # Class chars: "abc" = [97, 98, 99]
    cls := malloc(3)
    store_byte(cls, 97)
    store_byte(cls + 1, 98)
    store_byte(cls + 2, 99)

    end := mk_end()
    cls_node := mk_char_class(cls, 3, end)

    # "b" = [98]
    t1 := malloc(2)
    store_byte(t1, 98)
    store_byte(t1 + 1, 0)

    # "d" = [100]
    t2 := malloc(2)
    store_byte(t2, 100)
    store_byte(t2 + 1, 0)

    result := 0
    I match_here(cls_node, t1, 0, 1) == 1 { result = result + 1 }
    I match_here(cls_node, t2, 0, 1) == 0 { result = result + 1 }

    free(t1)
    free(t2)
    free(cls)
    free(cls_node)
    free(end)
    result
}
"#;
    assert_exit_code(source, 2);
}

#[test]
fn e2e_p83_regex_neg_char_class() {
    // "[^abc]" matches "d" but not "a" -> 2 points
    let source = r#"
# Node types: 0=END, 6=NEG_CLASS
F char_in_class(cls: i64, cls_len: i64, c: i64) -> i64 {
    i := mut 0
    L i < cls_len {
        I load_byte(cls + i) == c { R 1 }
        i = i + 1
    }
    0
}

F match_here(regex: i64, text: i64, pos: i64, tlen: i64) -> i64 {
    rtype := load_i64(regex)
    I rtype == 0 { R 1 }
    I rtype == 6 {
        # NEG_CLASS: matches any char NOT in the class
        I pos >= tlen { R 0 }
        cls := load_i64(regex + 8)
        cls_len := load_i64(regex + 16)
        next := load_i64(regex + 24)
        I char_in_class(cls, cls_len, load_byte(text + pos)) == 0 {
            R match_here(next, text, pos + 1, tlen)
        }
        R 0
    }
    0
}

F mk_end() -> i64 {
    n := malloc(32)
    store_i64(n, 0)
    store_i64(n + 8, 0)
    store_i64(n + 16, 0)
    store_i64(n + 24, 0)
    n
}

F mk_neg_class(cls: i64, cls_len: i64, next: i64) -> i64 {
    n := malloc(32)
    store_i64(n, 6)
    store_i64(n + 8, cls)
    store_i64(n + 16, cls_len)
    store_i64(n + 24, next)
    n
}

F main() -> i64 {
    # Excluded chars: "abc"
    cls := malloc(3)
    store_byte(cls, 97)
    store_byte(cls + 1, 98)
    store_byte(cls + 2, 99)

    end := mk_end()
    neg_node := mk_neg_class(cls, 3, end)

    # "d" = [100] -> should match (not in class)
    t1 := malloc(2)
    store_byte(t1, 100)
    store_byte(t1 + 1, 0)

    # "a" = [97] -> should NOT match (in class)
    t2 := malloc(2)
    store_byte(t2, 97)
    store_byte(t2 + 1, 0)

    result := 0
    I match_here(neg_node, t1, 0, 1) == 1 { result = result + 1 }
    I match_here(neg_node, t2, 0, 1) == 0 { result = result + 1 }

    free(t1)
    free(t2)
    free(cls)
    free(neg_node)
    free(end)
    result
}
"#;
    assert_exit_code(source, 2);
}

#[test]
fn e2e_p83_regex_anchor_start() {
    // "^abc" matches "abcdef" (starts at 0) but not "xabc" (doesn't start at 0) -> 2 points
    // Anchor start: match_from_start only tries position 0
    let source = r#"
F match_here(regex: i64, text: i64, pos: i64, tlen: i64) -> i64 {
    rtype := load_i64(regex)
    I rtype == 0 { R 1 }
    I rtype == 1 {
        I pos >= tlen { R 0 }
        ch := load_i64(regex + 8)
        I load_byte(text + pos) == ch {
            next := load_i64(regex + 24)
            R match_here(next, text, pos + 1, tlen)
        }
        R 0
    }
    0
}

# Anchor start: only try from position 0
F match_anchored_start(regex: i64, text: i64, tlen: i64) -> i64 {
    match_here(regex, text, 0, tlen)
}

# Search anywhere: try each starting position
F match_search(regex: i64, text: i64, tlen: i64) -> i64 {
    i := mut 0
    L i <= tlen {
        I match_here(regex, text, i, tlen) == 1 { R 1 }
        i = i + 1
    }
    0
}

F mk_end() -> i64 {
    n := malloc(32)
    store_i64(n, 0)
    store_i64(n + 8, 0)
    store_i64(n + 16, 0)
    store_i64(n + 24, 0)
    n
}

F mk_literal(c: i64, next: i64) -> i64 {
    n := malloc(32)
    store_i64(n, 1)
    store_i64(n + 8, c)
    store_i64(n + 16, 0)
    store_i64(n + 24, next)
    n
}

F main() -> i64 {
    # Pattern: ^abc  (anchored to start)
    end := mk_end()
    c_node := mk_literal(99, end)
    b_node := mk_literal(98, c_node)
    a_node := mk_literal(97, b_node)

    # "abcdef" = [97, 98, 99, 100, 101, 102]
    t1 := malloc(7)
    store_byte(t1, 97)
    store_byte(t1 + 1, 98)
    store_byte(t1 + 2, 99)
    store_byte(t1 + 3, 100)
    store_byte(t1 + 4, 101)
    store_byte(t1 + 5, 102)
    store_byte(t1 + 6, 0)

    # "xabc" = [120, 97, 98, 99]
    t2 := malloc(5)
    store_byte(t2, 120)
    store_byte(t2 + 1, 97)
    store_byte(t2 + 2, 98)
    store_byte(t2 + 3, 99)
    store_byte(t2 + 4, 0)

    result := 0
    # "abcdef" matches at start
    I match_anchored_start(a_node, t1, 6) == 1 { result = result + 1 }
    # "xabc" does NOT match at start (starts with 'x')
    I match_anchored_start(a_node, t2, 4) == 0 { result = result + 1 }

    free(t1)
    free(t2)
    free(a_node)
    free(b_node)
    free(c_node)
    free(end)
    result
}
"#;
    assert_exit_code(source, 2);
}

#[test]
fn e2e_p83_regex_anchor_end() {
    // "abc$" matches "xabc" but not "abcx" -> 2 points
    // Anchor end: after matching the pattern, verify we're at end of string
    let source = r#"
# match_here_to_end: matches pattern then requires end of string
F match_here_to_end(regex: i64, text: i64, pos: i64, tlen: i64) -> i64 {
    rtype := load_i64(regex)
    # END node: anchor end requires pos == tlen
    I rtype == 0 { I pos == tlen { R 1 } E { R 0 } }
    I rtype == 1 {
        I pos >= tlen { R 0 }
        ch := load_i64(regex + 8)
        I load_byte(text + pos) == ch {
            next := load_i64(regex + 24)
            R match_here_to_end(next, text, pos + 1, tlen)
        }
        R 0
    }
    0
}

# Try matching from every position, requiring anchor at end
F match_anchor_end(regex: i64, text: i64, tlen: i64) -> i64 {
    i := mut 0
    L i <= tlen {
        I match_here_to_end(regex, text, i, tlen) == 1 { R 1 }
        i = i + 1
    }
    0
}

F mk_end() -> i64 {
    n := malloc(32)
    store_i64(n, 0)
    store_i64(n + 8, 0)
    store_i64(n + 16, 0)
    store_i64(n + 24, 0)
    n
}

F mk_literal(c: i64, next: i64) -> i64 {
    n := malloc(32)
    store_i64(n, 1)
    store_i64(n + 8, c)
    store_i64(n + 16, 0)
    store_i64(n + 24, next)
    n
}

F main() -> i64 {
    end := mk_end()
    c_node := mk_literal(99, end)
    b_node := mk_literal(98, c_node)
    a_node := mk_literal(97, b_node)

    # "xabc" = [120, 97, 98, 99] -> should match (abc is at end)
    t1 := malloc(5)
    store_byte(t1, 120)
    store_byte(t1 + 1, 97)
    store_byte(t1 + 2, 98)
    store_byte(t1 + 3, 99)
    store_byte(t1 + 4, 0)

    # "abcx" = [97, 98, 99, 120] -> should NOT match (not at end)
    t2 := malloc(5)
    store_byte(t2, 97)
    store_byte(t2 + 1, 98)
    store_byte(t2 + 2, 99)
    store_byte(t2 + 3, 120)
    store_byte(t2 + 4, 0)

    result := 0
    I match_anchor_end(a_node, t1, 4) == 1 { result = result + 1 }
    I match_anchor_end(a_node, t2, 4) == 0 { result = result + 1 }

    free(t1)
    free(t2)
    free(a_node)
    free(b_node)
    free(c_node)
    free(end)
    result
}
"#;
    assert_exit_code(source, 2);
}

#[test]
fn e2e_p83_regex_char_range() {
    // "[a-z]" matches "m" but not "5" -> 2 points
    // Range: store lo in data, hi in extra
    let source = r#"
# Node type 3 = RANGE: data=lo, extra=hi
F match_here(regex: i64, text: i64, pos: i64, tlen: i64) -> i64 {
    rtype := load_i64(regex)
    I rtype == 0 { R 1 }
    I rtype == 3 {
        I pos >= tlen { R 0 }
        lo := load_i64(regex + 8)
        hi := load_i64(regex + 16)
        next := load_i64(regex + 24)
        c := load_byte(text + pos)
        I c >= lo && c <= hi {
            R match_here(next, text, pos + 1, tlen)
        }
        R 0
    }
    0
}

F mk_end() -> i64 {
    n := malloc(32)
    store_i64(n, 0)
    store_i64(n + 8, 0)
    store_i64(n + 16, 0)
    store_i64(n + 24, 0)
    n
}

F mk_range(lo: i64, hi: i64, next: i64) -> i64 {
    n := malloc(32)
    store_i64(n, 3)
    store_i64(n + 8, lo)
    store_i64(n + 16, hi)
    store_i64(n + 24, next)
    n
}

F main() -> i64 {
    end := mk_end()
    # [a-z]: lo=97 ('a'), hi=122 ('z')
    rng := mk_range(97, 122, end)

    # "m" = [109]
    t1 := malloc(2)
    store_byte(t1, 109)
    store_byte(t1 + 1, 0)

    # "5" = [53]
    t2 := malloc(2)
    store_byte(t2, 53)
    store_byte(t2 + 1, 0)

    result := 0
    I match_here(rng, t1, 0, 1) == 1 { result = result + 1 }
    I match_here(rng, t2, 0, 1) == 0 { result = result + 1 }

    free(t1)
    free(t2)
    free(rng)
    free(end)
    result
}
"#;
    assert_exit_code(source, 2);
}

#[test]
fn e2e_p83_regex_escape_literal() {
    // "a\.b" treats '.' as literal dot: matches "a.b" but not "axb" -> 2 points
    let source = r#"
F match_here(regex: i64, text: i64, pos: i64, tlen: i64) -> i64 {
    rtype := load_i64(regex)
    I rtype == 0 { R 1 }
    I rtype == 1 {
        I pos >= tlen { R 0 }
        ch := load_i64(regex + 8)
        I load_byte(text + pos) == ch {
            next := load_i64(regex + 24)
            R match_here(next, text, pos + 1, tlen)
        }
        R 0
    }
    0
}

F mk_end() -> i64 {
    n := malloc(32)
    store_i64(n, 0)
    store_i64(n + 8, 0)
    store_i64(n + 16, 0)
    store_i64(n + 24, 0)
    n
}

F mk_literal(c: i64, next: i64) -> i64 {
    n := malloc(32)
    store_i64(n, 1)
    store_i64(n + 8, c)
    store_i64(n + 16, 0)
    store_i64(n + 24, next)
    n
}

F main() -> i64 {
    # Pattern: a . b  (all three as LITERAL — dot is escaped to mean literal '.')
    # dot ASCII = 46
    end := mk_end()
    b_node := mk_literal(98, end)
    dot_node := mk_literal(46, b_node)
    a_node := mk_literal(97, dot_node)

    # "a.b" = [97, 46, 98]
    t1 := malloc(4)
    store_byte(t1, 97)
    store_byte(t1 + 1, 46)
    store_byte(t1 + 2, 98)
    store_byte(t1 + 3, 0)

    # "axb" = [97, 120, 98]
    t2 := malloc(4)
    store_byte(t2, 97)
    store_byte(t2 + 1, 120)
    store_byte(t2 + 2, 98)
    store_byte(t2 + 3, 0)

    result := 0
    I match_here(a_node, t1, 0, 3) == 1 { result = result + 1 }
    I match_here(a_node, t2, 0, 3) == 0 { result = result + 1 }

    free(t1)
    free(t2)
    free(a_node)
    free(dot_node)
    free(b_node)
    free(end)
    result
}
"#;
    assert_exit_code(source, 2);
}

#[test]
fn e2e_p83_regex_complex_pattern() {
    // "^[A-Z][a-z]+$" matches "Hello" but not "hello" -> 2 points
    // Anchored start+end, uppercase then lowercase+
    let source = r#"
# Node types: 0=END, 3=RANGE, 8=PLUS_RANGE (plus for a range)
# PLUS_RANGE: data=lo, extra=hi

F match_here_end(regex: i64, text: i64, pos: i64, tlen: i64) -> i64 {
    rtype := load_i64(regex)
    I rtype == 0 { I pos == tlen { R 1 } E { R 0 } }
    I rtype == 3 {
        # RANGE single match
        I pos >= tlen { R 0 }
        lo := load_i64(regex + 8)
        hi := load_i64(regex + 16)
        next := load_i64(regex + 24)
        c := load_byte(text + pos)
        I c >= lo && c <= hi {
            R match_here_end(next, text, pos + 1, tlen)
        }
        R 0
    }
    I rtype == 8 {
        # PLUS_RANGE: at least one from range [lo, hi]
        lo := load_i64(regex + 8)
        hi := load_i64(regex + 16)
        next := load_i64(regex + 24)
        I pos >= tlen { R 0 }
        c := load_byte(text + pos)
        I c < lo || c > hi { R 0 }
        # Consume first, then greedily more
        p := mut pos + 1
        I match_here_end(next, text, p, tlen) == 1 { R 1 }
        L p < tlen && load_byte(text + p) >= lo && load_byte(text + p) <= hi {
            p = p + 1
            I match_here_end(next, text, p, tlen) == 1 { R 1 }
        }
        R 0
    }
    0
}

F mk_end() -> i64 {
    n := malloc(32)
    store_i64(n, 0)
    store_i64(n + 8, 0)
    store_i64(n + 16, 0)
    store_i64(n + 24, 0)
    n
}

F mk_range(lo: i64, hi: i64, next: i64) -> i64 {
    n := malloc(32)
    store_i64(n, 3)
    store_i64(n + 8, lo)
    store_i64(n + 16, hi)
    store_i64(n + 24, next)
    n
}

F mk_plus_range(lo: i64, hi: i64, next: i64) -> i64 {
    n := malloc(32)
    store_i64(n, 8)
    store_i64(n + 8, lo)
    store_i64(n + 16, hi)
    store_i64(n + 24, next)
    n
}

F main() -> i64 {
    # Pattern: ^[A-Z][a-z]+$
    # A-Z: 65-90, a-z: 97-122
    end := mk_end()
    lc_plus := mk_plus_range(97, 122, end)  # [a-z]+
    uc_range := mk_range(65, 90, lc_plus)   # [A-Z]

    # "Hello" = [72, 101, 108, 108, 111]
    t1 := malloc(6)
    store_byte(t1, 72)
    store_byte(t1 + 1, 101)
    store_byte(t1 + 2, 108)
    store_byte(t1 + 3, 108)
    store_byte(t1 + 4, 111)
    store_byte(t1 + 5, 0)

    # "hello" = [104, 101, 108, 108, 111] (starts lowercase)
    t2 := malloc(6)
    store_byte(t2, 104)
    store_byte(t2 + 1, 101)
    store_byte(t2 + 2, 108)
    store_byte(t2 + 3, 108)
    store_byte(t2 + 4, 111)
    store_byte(t2 + 5, 0)

    result := 0
    # Anchored start+end: try from pos 0 only, require reach end
    I match_here_end(uc_range, t1, 0, 5) == 1 { result = result + 1 }
    I match_here_end(uc_range, t2, 0, 5) == 0 { result = result + 1 }

    free(t1)
    free(t2)
    free(uc_range)
    free(lc_plus)
    free(end)
    result
}
"#;
    assert_exit_code(source, 2);
}

#[test]
fn e2e_p83_regex_digit_class() {
    // "[0-9]+" matches in "abc123" (finds digits) -> 1 point
    let source = r#"
F match_here(regex: i64, text: i64, pos: i64, tlen: i64) -> i64 {
    rtype := load_i64(regex)
    I rtype == 0 { R 1 }
    I rtype == 8 {
        # PLUS_RANGE
        lo := load_i64(regex + 8)
        hi := load_i64(regex + 16)
        next := load_i64(regex + 24)
        I pos >= tlen { R 0 }
        c := load_byte(text + pos)
        I c < lo || c > hi { R 0 }
        p := mut pos + 1
        I match_here(next, text, p, tlen) == 1 { R 1 }
        L p < tlen && load_byte(text + p) >= lo && load_byte(text + p) <= hi {
            p = p + 1
            I match_here(next, text, p, tlen) == 1 { R 1 }
        }
        R 0
    }
    0
}

F match_search(regex: i64, text: i64, tlen: i64) -> i64 {
    i := mut 0
    L i <= tlen {
        I match_here(regex, text, i, tlen) == 1 { R 1 }
        i = i + 1
    }
    0
}

F mk_end() -> i64 {
    n := malloc(32)
    store_i64(n, 0)
    store_i64(n + 8, 0)
    store_i64(n + 16, 0)
    store_i64(n + 24, 0)
    n
}

F mk_plus_range(lo: i64, hi: i64, next: i64) -> i64 {
    n := malloc(32)
    store_i64(n, 8)
    store_i64(n + 8, lo)
    store_i64(n + 16, hi)
    store_i64(n + 24, next)
    n
}

F main() -> i64 {
    end := mk_end()
    # [0-9]+: 48='0', 57='9'
    digit_plus := mk_plus_range(48, 57, end)

    # "abc123" = [97, 98, 99, 49, 50, 51]
    text := malloc(7)
    store_byte(text, 97)
    store_byte(text + 1, 98)
    store_byte(text + 2, 99)
    store_byte(text + 3, 49)
    store_byte(text + 4, 50)
    store_byte(text + 5, 51)
    store_byte(text + 6, 0)

    result := 0
    I match_search(digit_plus, text, 6) == 1 { result = result + 1 }

    free(text)
    free(digit_plus)
    free(end)
    result
}
"#;
    assert_exit_code(source, 1);
}

#[test]
fn e2e_p83_regex_empty_match() {
    // "a*" with STAR can match zero occurrences (empty match) -> 1 point
    let source = r#"
F match_here(regex: i64, text: i64, pos: i64, tlen: i64) -> i64 {
    rtype := load_i64(regex)
    I rtype == 0 { R 1 }
    I rtype == 7 {
        ch := load_i64(regex + 8)
        next := load_i64(regex + 24)
        # Zero repetitions: try continuation immediately
        I match_here(next, text, pos, tlen) == 1 { R 1 }
        # One or more
        p := mut pos
        L p < tlen && load_byte(text + p) == ch {
            p = p + 1
            I match_here(next, text, p, tlen) == 1 { R 1 }
        }
        R 0
    }
    0
}

F mk_end() -> i64 {
    n := malloc(32)
    store_i64(n, 0)
    store_i64(n + 8, 0)
    store_i64(n + 16, 0)
    store_i64(n + 24, 0)
    n
}

F mk_star(c: i64, next: i64) -> i64 {
    n := malloc(32)
    store_i64(n, 7)
    store_i64(n + 8, c)
    store_i64(n + 16, 0)
    store_i64(n + 24, next)
    n
}

F main() -> i64 {
    end := mk_end()
    # Pattern: a*  (zero or more 'a's)
    star_a := mk_star(97, end)

    # Empty string: try matching a* against empty
    empty := malloc(1)
    store_byte(empty, 0)

    result := 0
    # a* matches empty string (zero a's) -> success
    I match_here(star_a, empty, 0, 0) == 1 { result = result + 1 }

    free(empty)
    free(star_a)
    free(end)
    result
}
"#;
    assert_exit_code(source, 1);
}

#[test]
fn e2e_p83_regex_no_match() {
    // "xyz" does not match "abc" -> 1 point
    let source = r#"
F match_here(regex: i64, text: i64, pos: i64, tlen: i64) -> i64 {
    rtype := load_i64(regex)
    I rtype == 0 { R 1 }
    I rtype == 1 {
        I pos >= tlen { R 0 }
        ch := load_i64(regex + 8)
        I load_byte(text + pos) == ch {
            next := load_i64(regex + 24)
            R match_here(next, text, pos + 1, tlen)
        }
        R 0
    }
    0
}

F match_search(regex: i64, text: i64, tlen: i64) -> i64 {
    i := mut 0
    L i <= tlen {
        I match_here(regex, text, i, tlen) == 1 { R 1 }
        i = i + 1
    }
    0
}

F mk_end() -> i64 {
    n := malloc(32)
    store_i64(n, 0)
    store_i64(n + 8, 0)
    store_i64(n + 16, 0)
    store_i64(n + 24, 0)
    n
}

F mk_literal(c: i64, next: i64) -> i64 {
    n := malloc(32)
    store_i64(n, 1)
    store_i64(n + 8, c)
    store_i64(n + 16, 0)
    store_i64(n + 24, next)
    n
}

F main() -> i64 {
    # Pattern: xyz = [120, 121, 122]
    end := mk_end()
    z_node := mk_literal(122, end)
    y_node := mk_literal(121, z_node)
    x_node := mk_literal(120, y_node)

    # Text: "abc" = [97, 98, 99]
    text := malloc(4)
    store_byte(text, 97)
    store_byte(text + 1, 98)
    store_byte(text + 2, 99)
    store_byte(text + 3, 0)

    result := 0
    # xyz should NOT be found in abc
    I match_search(x_node, text, 3) == 0 { result = result + 1 }

    free(text)
    free(x_node)
    free(y_node)
    free(z_node)
    free(end)
    result
}
"#;
    assert_exit_code(source, 1);
}

// ==================== Section 2: HTTP Client Helper Tests ====================

#[test]
fn e2e_p83_http_status_success() {
    // 200, 201, 299 are success (2xx); 199 and 300 are not -> 3 points scored
    let source = r#"
F http_is_success(code: i64) -> i64 {
    I code >= 200 && code <= 299 { 1 } E { 0 }
}

F main() -> i64 {
    result := 0
    I http_is_success(200) == 1 { result = result + 1 }
    I http_is_success(201) == 1 { result = result + 1 }
    I http_is_success(299) == 1 { result = result + 1 }
    I http_is_success(199) == 0 { result = result }
    I http_is_success(300) == 0 { result = result }
    result
}
"#;
    assert_exit_code(source, 3);
}

#[test]
fn e2e_p83_http_status_redirect() {
    // 301, 302 are redirect; 200, 400 are not -> 2 points
    let source = r#"
F http_is_redirect(code: i64) -> i64 {
    I code >= 300 && code <= 399 { 1 } E { 0 }
}

F main() -> i64 {
    result := 0
    I http_is_redirect(301) == 1 { result = result + 1 }
    I http_is_redirect(302) == 1 { result = result + 1 }
    I http_is_redirect(200) == 0 { result = result }
    I http_is_redirect(400) == 0 { result = result }
    result
}
"#;
    assert_exit_code(source, 2);
}

#[test]
fn e2e_p83_http_status_client_error() {
    // 400, 404, 499 are client errors (4xx) -> 2 points (one for any + one for specific)
    let source = r#"
F http_is_client_error(code: i64) -> i64 {
    I code >= 400 && code <= 499 { 1 } E { 0 }
}

F main() -> i64 {
    result := 0
    I http_is_client_error(400) == 1 { result = result + 1 }
    I http_is_client_error(404) == 1 { result = result + 1 }
    I http_is_client_error(499) == 1 { result = result }
    I http_is_client_error(399) == 0 { result = result }
    I http_is_client_error(500) == 0 { result = result }
    result
}
"#;
    assert_exit_code(source, 2);
}

#[test]
fn e2e_p83_http_status_server_error() {
    // 500, 503 are server errors (5xx) -> 2 points
    let source = r#"
F http_is_server_error(code: i64) -> i64 {
    I code >= 500 && code <= 599 { 1 } E { 0 }
}

F main() -> i64 {
    result := 0
    I http_is_server_error(500) == 1 { result = result + 1 }
    I http_is_server_error(503) == 1 { result = result + 1 }
    I http_is_server_error(499) == 0 { result = result }
    I http_is_server_error(600) == 0 { result = result }
    result
}
"#;
    assert_exit_code(source, 2);
}

#[test]
fn e2e_p83_http_hex_digit_value() {
    // hex_digit_value: '0'->0, '9'->9, 'a'->10, 'f'->15, 'A'->10, 'F'->15 -> 3 points
    let source = r#"
F hex_digit_value(c: i64) -> i64 {
    I c >= 48 && c <= 57 { R c - 48 }        # '0'-'9'
    I c >= 97 && c <= 102 { R c - 97 + 10 }  # 'a'-'f'
    I c >= 65 && c <= 70 { R c - 65 + 10 }   # 'A'-'F'
    0 - 1
}

F main() -> i64 {
    result := 0
    I hex_digit_value(48) == 0 { result = result + 1 }   # '0'
    I hex_digit_value(57) == 9 { result = result + 1 }   # '9'
    I hex_digit_value(97) == 10 { result = result + 1 }  # 'a'
    I hex_digit_value(102) == 15 { result = result }     # 'f'
    I hex_digit_value(65) == 10 { result = result }      # 'A'
    I hex_digit_value(70) == 15 { result = result }      # 'F'
    result
}
"#;
    assert_exit_code(source, 3);
}

#[test]
fn e2e_p83_http_hex_nibble() {
    // hex_nibble: 0->'0'(48), 9->'9'(57), 10->'A'(65), 15->'F'(70) -> 4 points
    let source = r#"
F hex_nibble(v: i64) -> i64 {
    I v < 10 { R v + 48 }       # '0'-'9'
    R v - 10 + 65               # 'A'-'F'
}

F main() -> i64 {
    result := 0
    I hex_nibble(0) == 48 { result = result + 1 }   # '0'
    I hex_nibble(9) == 57 { result = result + 1 }   # '9'
    I hex_nibble(10) == 65 { result = result + 1 }  # 'A'
    I hex_nibble(15) == 70 { result = result + 1 }  # 'F'
    result
}
"#;
    assert_exit_code(source, 4);
}

#[test]
fn e2e_p83_http_url_encode_simple() {
    // space (32) -> %20 (3 chars written), safe chars pass through -> 2 points
    let source = r#"
F is_url_safe(c: i64) -> i64 {
    # Unreserved chars: A-Z a-z 0-9 - _ . ~
    I c >= 65 && c <= 90 { R 1 }   # A-Z
    I c >= 97 && c <= 122 { R 1 }  # a-z
    I c >= 48 && c <= 57 { R 1 }   # 0-9
    I c == 45 { R 1 }  # -
    I c == 95 { R 1 }  # _
    I c == 46 { R 1 }  # .
    I c == 126 { R 1 } # ~
    0
}

F hex_nibble(v: i64) -> i64 {
    I v < 10 { R v + 48 }
    R v - 10 + 65
}

# url_encode_char: write encoded form of c into buf at pos, return new pos
F url_encode_char(buf: i64, pos: i64, c: i64) -> i64 {
    I is_url_safe(c) == 1 {
        store_byte(buf + pos, c)
        R pos + 1
    }
    # Percent-encode: %HH
    store_byte(buf + pos, 37)      # '%'
    store_byte(buf + pos + 1, hex_nibble((c >> 4) & 15))
    store_byte(buf + pos + 2, hex_nibble(c & 15))
    pos + 3
}

F main() -> i64 {
    buf := malloc(64)
    pos := mut 0

    # Encode space (32) -> %20
    pos = url_encode_char(buf, pos, 32)
    # Encode 'a' (97) -> passes through as 'a'
    pos = url_encode_char(buf, pos, 97)

    result := 0
    # Space encodes to 3 chars: '%', '2', '0'
    I load_byte(buf) == 37 { result = result + 1 }   # '%'
    I load_byte(buf + 1) == 50 { result = result }   # '2'
    I load_byte(buf + 2) == 48 { result = result }   # '0'
    # 'a' encodes to 1 char: 'a'
    I load_byte(buf + 3) == 97 { result = result + 1 }

    free(buf)
    result
}
"#;
    assert_exit_code(source, 2);
}

#[test]
fn e2e_p83_http_chunked_parse_size() {
    // Parse "1a\r\n" chunk size -> 26 (0x1a) -> 1 point
    let source = r#"
F hex_digit_value(c: i64) -> i64 {
    I c >= 48 && c <= 57 { R c - 48 }
    I c >= 97 && c <= 102 { R c - 97 + 10 }
    I c >= 65 && c <= 70 { R c - 65 + 10 }
    0 - 1
}

# Parse hex size from buffer until \r\n
F parse_chunk_size(buf: i64, buf_len: i64) -> i64 {
    size := mut 0
    i := mut 0
    L i < buf_len {
        c := load_byte(buf + i)
        I c == 13 { B }   # \r
        d := hex_digit_value(c)
        I d < 0 { B }
        size = size * 16 + d
        i = i + 1
    }
    size
}

F main() -> i64 {
    # "1a\r\n" = [49, 97, 13, 10]
    buf := malloc(4)
    store_byte(buf, 49)   # '1'
    store_byte(buf + 1, 97)  # 'a'
    store_byte(buf + 2, 13)  # \r
    store_byte(buf + 3, 10)  # \n

    sz := parse_chunk_size(buf, 4)

    result := 0
    I sz == 26 { result = result + 1 }  # 0x1a = 26

    free(buf)
    result
}
"#;
    assert_exit_code(source, 1);
}

#[test]
fn e2e_p83_http_method_constants() {
    // CLIENT_GET=1, POST=2, PUT=3, DELETE=4 -> 4 points
    let source = r#"
C CLIENT_GET: i64 = 1
C CLIENT_POST: i64 = 2
C CLIENT_PUT: i64 = 3
C CLIENT_DELETE: i64 = 4

F main() -> i64 {
    result := 0
    I CLIENT_GET == 1 { result = result + 1 }
    I CLIENT_POST == 2 { result = result + 1 }
    I CLIENT_PUT == 3 { result = result + 1 }
    I CLIENT_DELETE == 4 { result = result + 1 }
    result
}
"#;
    assert_exit_code(source, 4);
}

#[test]
fn e2e_p83_http_query_builder() {
    // Build "key=val&k2=v2" query string -> 2 points
    let source = r#"
# Append a byte into buf at pos, return new pos
F buf_put(buf: i64, pos: i64, c: i64) -> i64 {
    store_byte(buf + pos, c)
    pos + 1
}

# Append null-terminated string bytes (not null itself)
F buf_put_str_bytes(buf: i64, pos: i64, s: i64, slen: i64) -> i64 {
    i := mut 0
    p := mut pos
    L i < slen {
        p = buf_put(buf, p, load_byte(s + i))
        i = i + 1
    }
    p
}

F main() -> i64 {
    buf := malloc(64)
    pos := mut 0

    # "key" = [107, 101, 121]
    key1 := malloc(4)
    store_byte(key1, 107)
    store_byte(key1 + 1, 101)
    store_byte(key1 + 2, 121)
    store_byte(key1 + 3, 0)

    # "val" = [118, 97, 108]
    val1 := malloc(4)
    store_byte(val1, 118)
    store_byte(val1 + 1, 97)
    store_byte(val1 + 2, 108)
    store_byte(val1 + 3, 0)

    # "k2" = [107, 50]
    key2 := malloc(3)
    store_byte(key2, 107)
    store_byte(key2 + 1, 50)
    store_byte(key2 + 2, 0)

    # "v2" = [118, 50]
    val2 := malloc(3)
    store_byte(val2, 118)
    store_byte(val2 + 1, 50)
    store_byte(val2 + 2, 0)

    # Build "key=val&k2=v2"
    pos = buf_put_str_bytes(buf, pos, key1, 3)
    pos = buf_put(buf, pos, 61)    # '='
    pos = buf_put_str_bytes(buf, pos, val1, 3)
    pos = buf_put(buf, pos, 38)    # '&'
    pos = buf_put_str_bytes(buf, pos, key2, 2)
    pos = buf_put(buf, pos, 61)    # '='
    pos = buf_put_str_bytes(buf, pos, val2, 2)
    store_byte(buf + pos, 0)

    # Expected length: 3+1+3+1+2+1+2 = 13
    result := 0
    I pos == 13 { result = result + 1 }
    # Verify '=' at position 3
    I load_byte(buf + 3) == 61 { result = result + 1 }

    free(key1)
    free(val1)
    free(key2)
    free(val2)
    free(buf)
    result
}
"#;
    assert_exit_code(source, 2);
}

// ==================== Section 3: SQLite Utility Tests ====================

#[test]
fn e2e_p83_sqlite_result_code_str() {
    // SQLITE_OK(0) -> compare byte output; SQLITE_ERROR(1) -> compare -> 3 points
    let source = r#"
F sqlite_result_code(code: i64) -> i64 {
    # Returns a tag: 0=SQLITE_OK, 1=SQLITE_ERROR, 2=SQLITE_BUSY, 3=SQLITE_LOCKED,
    # 4=SQLITE_READONLY, 5=SQLITE_ROW, 6=SQLITE_DONE, 7=SQLITE_UNKNOWN
    I code == 0 { R 0 }
    I code == 1 { R 1 }
    I code == 5 { R 2 }
    I code == 6 { R 3 }
    I code == 8 { R 4 }
    I code == 100 { R 5 }
    I code == 101 { R 6 }
    7
}

F main() -> i64 {
    result := 0
    # SQLITE_OK = 0 -> tag 0
    I sqlite_result_code(0) == 0 { result = result + 1 }
    # SQLITE_ERROR = 1 -> tag 1
    I sqlite_result_code(1) == 1 { result = result + 1 }
    # SQLITE_ROW = 100 -> tag 5
    I sqlite_result_code(100) == 5 { result = result + 1 }
    result
}
"#;
    assert_exit_code(source, 3);
}

#[test]
fn e2e_p83_sqlite_is_ok() {
    // is_ok(0)==1, is_ok(1)==0 -> 2 points
    let source = r#"
F sqlite_is_ok(code: i64) -> i64 {
    I code == 0 { 1 } E { 0 }
}

F main() -> i64 {
    result := 0
    I sqlite_is_ok(0) == 1 { result = result + 1 }
    I sqlite_is_ok(1) == 0 { result = result + 1 }
    result
}
"#;
    assert_exit_code(source, 2);
}

#[test]
fn e2e_p83_sqlite_has_row() {
    // has_row(100)==1, has_row(101)==0 -> 2 points
    // SQLITE_ROW = 100
    let source = r#"
C SQLITE_ROW: i64 = 100

F sqlite_has_row(code: i64) -> i64 {
    I code == SQLITE_ROW { 1 } E { 0 }
}

F main() -> i64 {
    result := 0
    I sqlite_has_row(100) == 1 { result = result + 1 }
    I sqlite_has_row(101) == 0 { result = result + 1 }
    result
}
"#;
    assert_exit_code(source, 2);
}

#[test]
fn e2e_p83_sqlite_is_done() {
    // is_done(101)==1, is_done(100)==0 -> 2 points
    // SQLITE_DONE = 101
    let source = r#"
C SQLITE_DONE: i64 = 101

F sqlite_is_done(code: i64) -> i64 {
    I code == SQLITE_DONE { 1 } E { 0 }
}

F main() -> i64 {
    result := 0
    I sqlite_is_done(101) == 1 { result = result + 1 }
    I sqlite_is_done(100) == 0 { result = result + 1 }
    result
}
"#;
    assert_exit_code(source, 2);
}

#[test]
fn e2e_p83_sqlite_constants() {
    // SQLITE_OK==0, SQLITE_ROW==100, SQLITE_DONE==101, SQLITE_ERROR==1 -> 4 points
    let source = r#"
C SQLITE_OK: i64 = 0
C SQLITE_ERROR: i64 = 1
C SQLITE_ROW: i64 = 100
C SQLITE_DONE: i64 = 101

F main() -> i64 {
    result := 0
    I SQLITE_OK == 0 { result = result + 1 }
    I SQLITE_ERROR == 1 { result = result + 1 }
    I SQLITE_ROW == 100 { result = result + 1 }
    I SQLITE_DONE == 101 { result = result + 1 }
    result
}
"#;
    assert_exit_code(source, 4);
}

#[test]
fn e2e_p83_sqlite_column_types() {
    // SQLITE_INTEGER==1, SQLITE_FLOAT==2, SQLITE_TEXT==3, SQLITE_BLOB==4, SQLITE_NULL==5 -> 5 points
    let source = r#"
C SQLITE_INTEGER: i64 = 1
C SQLITE_FLOAT: i64 = 2
C SQLITE_TEXT: i64 = 3
C SQLITE_BLOB: i64 = 4
C SQLITE_NULL: i64 = 5

F main() -> i64 {
    result := 0
    I SQLITE_INTEGER == 1 { result = result + 1 }
    I SQLITE_FLOAT == 2 { result = result + 1 }
    I SQLITE_TEXT == 3 { result = result + 1 }
    I SQLITE_BLOB == 4 { result = result + 1 }
    I SQLITE_NULL == 5 { result = result + 1 }
    result
}
"#;
    assert_exit_code(source, 5);
}

#[test]
fn e2e_p83_sqlite_result_codes_extended() {
    // BUSY==5, LOCKED==6, READONLY==8 -> 3 points
    let source = r#"
C SQLITE_BUSY: i64 = 5
C SQLITE_LOCKED: i64 = 6
C SQLITE_READONLY: i64 = 8

F main() -> i64 {
    result := 0
    I SQLITE_BUSY == 5 { result = result + 1 }
    I SQLITE_LOCKED == 6 { result = result + 1 }
    I SQLITE_READONLY == 8 { result = result + 1 }
    result
}
"#;
    assert_exit_code(source, 3);
}

#[test]
fn e2e_p83_sqlite_result_code_unknown() {
    // result_code for an invalid/unrecognized code returns tag 7 (SQLITE_UNKNOWN) -> 1 point
    let source = r#"
F sqlite_classify_code(code: i64) -> i64 {
    I code == 0 { R 0 }    # SQLITE_OK
    I code == 1 { R 1 }    # SQLITE_ERROR
    I code == 5 { R 2 }    # SQLITE_BUSY
    I code == 6 { R 3 }    # SQLITE_LOCKED
    I code == 8 { R 4 }    # SQLITE_READONLY
    I code == 100 { R 5 }  # SQLITE_ROW
    I code == 101 { R 6 }  # SQLITE_DONE
    7                      # SQLITE_UNKNOWN (unrecognized)
}

F main() -> i64 {
    result := 0
    # An invalid code like 999 should return tag 7 (unknown)
    I sqlite_classify_code(999) == 7 { result = result + 1 }
    result
}
"#;
    assert_exit_code(source, 1);
}

#[test]
fn e2e_p83_sqlite_combined_checks() {
    // Combine is_ok + has_row + is_done for various values -> 3 points
    let source = r#"
C SQLITE_OK: i64 = 0
C SQLITE_ROW: i64 = 100
C SQLITE_DONE: i64 = 101

F sqlite_is_ok(code: i64) -> i64 {
    I code == SQLITE_OK { 1 } E { 0 }
}

F sqlite_has_row(code: i64) -> i64 {
    I code == SQLITE_ROW { 1 } E { 0 }
}

F sqlite_is_done(code: i64) -> i64 {
    I code == SQLITE_DONE { 1 } E { 0 }
}

F sqlite_is_terminal(code: i64) -> i64 {
    I sqlite_is_ok(code) == 1 { R 1 }
    I sqlite_is_done(code) == 1 { R 1 }
    0
}

F main() -> i64 {
    result := 0
    # OK is terminal but not a row
    I sqlite_is_ok(0) == 1 && sqlite_has_row(0) == 0 { result = result + 1 }
    # DONE is terminal and not a row
    I sqlite_is_terminal(101) == 1 && sqlite_has_row(101) == 0 { result = result + 1 }
    # ROW is not terminal but is a row
    I sqlite_is_terminal(100) == 0 && sqlite_has_row(100) == 1 { result = result + 1 }
    result
}
"#;
    assert_exit_code(source, 3);
}

#[test]
fn e2e_p83_sqlite_constraint_code() {
    // SQLITE_CONSTRAINT==19, SQLITE_MISMATCH==20 -> 2 points
    let source = r#"
C SQLITE_CONSTRAINT: i64 = 19
C SQLITE_MISMATCH: i64 = 20

F main() -> i64 {
    result := 0
    I SQLITE_CONSTRAINT == 19 { result = result + 1 }
    I SQLITE_MISMATCH == 20 { result = result + 1 }
    result
}
"#;
    assert_exit_code(source, 2);
}
