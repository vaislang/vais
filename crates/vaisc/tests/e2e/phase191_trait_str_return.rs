use super::helpers::*;

/// Trait method returns a str literal. The vtable dispatch should
/// correctly pass the { i8*, i64 } fat pointer through.
#[test]
fn e2e_phase191_trait_str_return_literal() {
    assert_exit_code(
        r#"
trait Describable {
    fn describe(&self) -> str
}

struct Cat {
    age: i64
}

impl Cat: Describable {
    fn describe(&self) -> str {
        "meow"
    }
}

fn use_desc(c: Cat) -> i64 {
    s := c.describe()
    0
}

fn main() -> i64 {
    c := Cat { age: 3 }
    use_desc(c)
}
"#,
        0,
    );
}

/// Trait method returns a concat result (heap-owned). The caller
/// should take ownership and free it at scope exit.
#[test]
fn e2e_phase191_trait_str_return_concat() {
    assert_exit_code(
        r#"
trait Named {
    fn name(&self) -> str
}

struct Dog {
    prefix: str
}

impl Dog: Named {
    fn name(&self) -> str {
        "good-" + "dog"
    }
}

fn get_name(d: Dog) -> i64 {
    n := d.name()
    0
}

fn main() -> i64 {
    d := Dog { prefix: "Rex" }
    get_name(d)
}
"#,
        0,
    );
}
