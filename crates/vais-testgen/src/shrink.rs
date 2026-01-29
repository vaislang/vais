//! Shrinking of test values to find minimal failing inputs.

use crate::generator::TestValue;

/// Shrinks test values to find minimal failing cases.
pub struct Shrinker;

impl Shrinker {
    /// Generate smaller variants of a test value.
    pub fn shrink(value: &TestValue) -> Vec<TestValue> {
        match value {
            TestValue::Int(n) => Self::shrink_int(*n),
            TestValue::Float(f) => Self::shrink_float(*f),
            TestValue::Bool(_) => vec![TestValue::Bool(false), TestValue::Bool(true)],
            TestValue::Str(s) => Self::shrink_str(s),
            TestValue::Array(items) => Self::shrink_array(items),
            TestValue::Tuple(items) => Self::shrink_tuple(items),
            TestValue::None => vec![],
        }
    }

    fn shrink_int(n: i64) -> Vec<TestValue> {
        if n == 0 {
            return vec![];
        }
        let mut results = vec![TestValue::Int(0)];
        if n > 0 {
            results.push(TestValue::Int(n / 2));
            results.push(TestValue::Int(n - 1));
        } else {
            results.push(TestValue::Int(n / 2));
            results.push(TestValue::Int(n + 1));
            results.push(TestValue::Int(-n)); // try positive
        }
        results
    }

    fn shrink_float(f: f64) -> Vec<TestValue> {
        if f == 0.0 {
            return vec![];
        }
        vec![
            TestValue::Float(0.0),
            TestValue::Float(f / 2.0),
            TestValue::Float(f.floor()),
        ]
    }

    fn shrink_str(s: &str) -> Vec<TestValue> {
        if s.is_empty() {
            return vec![];
        }
        let mut results = vec![TestValue::Str(String::new())];
        if s.len() > 1 {
            // Remove first/last char
            results.push(TestValue::Str(s[1..].to_string()));
            results.push(TestValue::Str(s[..s.len() - 1].to_string()));
            // Half the string
            results.push(TestValue::Str(s[..s.len() / 2].to_string()));
        }
        results
    }

    fn shrink_array(items: &[TestValue]) -> Vec<TestValue> {
        if items.is_empty() {
            return vec![];
        }
        let mut results = vec![TestValue::Array(vec![])];
        // Remove one element at a time
        for i in 0..items.len() {
            let mut smaller = items.to_vec();
            smaller.remove(i);
            results.push(TestValue::Array(smaller));
        }
        // Shrink individual elements
        for (i, item) in items.iter().enumerate() {
            for shrunk in Self::shrink(item) {
                let mut modified = items.to_vec();
                modified[i] = shrunk;
                results.push(TestValue::Array(modified));
            }
        }
        results
    }

    fn shrink_tuple(items: &[TestValue]) -> Vec<TestValue> {
        let mut results = vec![];
        // Shrink individual elements
        for (i, item) in items.iter().enumerate() {
            for shrunk in Self::shrink(item) {
                let mut modified = items.to_vec();
                modified[i] = shrunk;
                results.push(TestValue::Tuple(modified));
            }
        }
        results
    }
}
