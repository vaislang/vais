//! Test case and test suite generation from function signatures.

use rand::Rng;
use std::fmt;

use crate::properties::Property;

/// A generated test value for a specific type.
#[derive(Debug, Clone, PartialEq)]
pub enum TestValue {
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),
    Array(Vec<TestValue>),
    Tuple(Vec<TestValue>),
    None,
}

impl fmt::Display for TestValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TestValue::Int(v) => write!(f, "{}", v),
            TestValue::Float(v) => write!(f, "{:.6}", v),
            TestValue::Bool(v) => write!(f, "{}", v),
            TestValue::Str(v) => write!(f, "\"{}\"", v),
            TestValue::Array(items) => {
                write!(f, "[")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", item)?;
                }
                write!(f, "]")
            }
            TestValue::Tuple(items) => {
                write!(f, "(")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", item)?;
                }
                write!(f, ")")
            }
            TestValue::None => write!(f, "None"),
        }
    }
}

/// Vais type hint for test generation.
#[derive(Debug, Clone, PartialEq)]
pub enum TypeHint {
    I64,
    I32,
    I16,
    I8,
    U64,
    U32,
    U16,
    U8,
    F64,
    F32,
    Bool,
    Str,
    Array(Box<TypeHint>),
    Tuple(Vec<TypeHint>),
    Unknown,
}

impl TypeHint {
    /// Parse a Vais type string into a TypeHint.
    pub fn from_str(s: &str) -> Self {
        match s.trim() {
            "i64" => TypeHint::I64,
            "i32" => TypeHint::I32,
            "i16" => TypeHint::I16,
            "i8" => TypeHint::I8,
            "u64" => TypeHint::U64,
            "u32" => TypeHint::U32,
            "u16" => TypeHint::U16,
            "u8" => TypeHint::U8,
            "f64" => TypeHint::F64,
            "f32" => TypeHint::F32,
            "bool" => TypeHint::Bool,
            "str" => TypeHint::Str,
            _ => TypeHint::Unknown,
        }
    }
}

/// A single test case with input values and expected properties.
#[derive(Debug, Clone)]
pub struct TestCase {
    pub name: String,
    pub function_name: String,
    pub inputs: Vec<TestValue>,
    pub properties: Vec<Property>,
    pub category: TestCategory,
}

/// Category of test case for organization.
#[derive(Debug, Clone, PartialEq)]
pub enum TestCategory {
    /// Random inputs.
    Random,
    /// Boundary values (0, -1, MAX, MIN).
    Boundary,
    /// Edge cases (empty string, empty array).
    Edge,
    /// Property assertion (idempotent, commutative, etc.).
    Property,
}

impl TestCase {
    /// Generate Vais source code for this test case.
    pub fn to_vais_source(&self) -> String {
        let args: Vec<String> = self.inputs.iter().map(|v| format!("{}", v)).collect();
        let call = format!("{}({})", self.function_name, args.join(", "));

        let mut lines = vec![];
        lines.push(format!("# Test: {}", self.name));
        lines.push(format!("# Category: {:?}", self.category));

        for prop in &self.properties {
            match prop {
                Property::DoesNotCrash => {
                    lines.push(format!("{}", call));
                    lines.push("# Should not crash".to_string());
                }
                Property::ReturnsNonZero => {
                    lines.push(format!("result := {}", call));
                    lines.push("assert(result != 0)".to_string());
                }
                Property::ReturnsInRange(lo, hi) => {
                    lines.push(format!("result := {}", call));
                    lines.push(format!("assert(result >= {})", lo));
                    lines.push(format!("assert(result <= {})", hi));
                }
                Property::Idempotent => {
                    lines.push(format!("r1 := {}", call));
                    let args2: Vec<String> =
                        std::iter::once("r1".to_string())
                            .chain(self.inputs[1..].iter().map(|v| format!("{}", v)))
                            .collect();
                    lines.push(format!(
                        "r2 := {}({})",
                        self.function_name,
                        args2.join(", ")
                    ));
                    lines.push("assert(r1 == r2)".to_string());
                }
                Property::Commutative => {
                    if self.inputs.len() >= 2 {
                        lines.push(format!("r1 := {}", call));
                        let mut reversed = self.inputs.clone();
                        reversed.swap(0, 1);
                        let args2: Vec<String> =
                            reversed.iter().map(|v| format!("{}", v)).collect();
                        lines.push(format!(
                            "r2 := {}({})",
                            self.function_name,
                            args2.join(", ")
                        ));
                        lines.push("assert(r1 == r2)".to_string());
                    }
                }
                Property::Custom(assertion) => {
                    lines.push(format!("result := {}", call));
                    lines.push(assertion.clone());
                }
            }
        }

        lines.join("\n")
    }
}

/// A collection of test cases for a function.
#[derive(Debug, Clone)]
pub struct TestSuite {
    pub function_name: String,
    pub param_types: Vec<TypeHint>,
    pub return_type: TypeHint,
    pub test_cases: Vec<TestCase>,
}

impl TestSuite {
    /// Generate Vais test file source code.
    pub fn to_vais_source(&self) -> String {
        let mut lines = vec![];
        lines.push(format!(
            "# Auto-generated property-based tests for {}",
            self.function_name
        ));
        lines.push(format!("# Parameters: {:?}", self.param_types));
        lines.push(format!("# Returns: {:?}", self.return_type));
        lines.push(String::new());

        for case in &self.test_cases {
            lines.push(case.to_vais_source());
            lines.push(String::new());
        }

        lines.join("\n")
    }
}

/// Test generator that creates test suites from function signatures.
pub struct TestGenerator {
    num_random_cases: usize,
    seed: u64,
}

impl TestGenerator {
    pub fn new() -> Self {
        Self {
            num_random_cases: 10,
            seed: 42,
        }
    }

    pub fn with_num_cases(mut self, n: usize) -> Self {
        self.num_random_cases = n;
        self
    }

    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed = seed;
        self
    }

    /// Generate a test suite for a function.
    pub fn generate(
        &self,
        function_name: &str,
        param_types: &[TypeHint],
        return_type: &TypeHint,
    ) -> TestSuite {
        let mut cases = vec![];
        let mut rng = rand::thread_rng();

        // Boundary value tests
        cases.extend(self.generate_boundary_tests(function_name, param_types));

        // Random tests
        for i in 0..self.num_random_cases {
            let inputs: Vec<TestValue> = param_types
                .iter()
                .map(|t| self.random_value(t, &mut rng))
                .collect();

            cases.push(TestCase {
                name: format!("{}_random_{}", function_name, i),
                function_name: function_name.to_string(),
                inputs,
                properties: vec![Property::DoesNotCrash],
                category: TestCategory::Random,
            });
        }

        // Property tests based on function name heuristics
        cases.extend(self.generate_property_tests(function_name, param_types, return_type));

        TestSuite {
            function_name: function_name.to_string(),
            param_types: param_types.to_vec(),
            return_type: return_type.clone(),
            test_cases: cases,
        }
    }

    fn generate_boundary_tests(
        &self,
        function_name: &str,
        param_types: &[TypeHint],
    ) -> Vec<TestCase> {
        let mut cases = vec![];

        // All zeros
        let zeros: Vec<TestValue> = param_types.iter().map(|t| self.zero_value(t)).collect();
        cases.push(TestCase {
            name: format!("{}_boundary_zeros", function_name),
            function_name: function_name.to_string(),
            inputs: zeros,
            properties: vec![Property::DoesNotCrash],
            category: TestCategory::Boundary,
        });

        // Max values
        let maxes: Vec<TestValue> = param_types.iter().map(|t| self.max_value(t)).collect();
        cases.push(TestCase {
            name: format!("{}_boundary_max", function_name),
            function_name: function_name.to_string(),
            inputs: maxes,
            properties: vec![Property::DoesNotCrash],
            category: TestCategory::Boundary,
        });

        // Min values
        let mins: Vec<TestValue> = param_types.iter().map(|t| self.min_value(t)).collect();
        cases.push(TestCase {
            name: format!("{}_boundary_min", function_name),
            function_name: function_name.to_string(),
            inputs: mins,
            properties: vec![Property::DoesNotCrash],
            category: TestCategory::Boundary,
        });

        // Negative one (for signed integers)
        if param_types.iter().any(|t| matches!(t, TypeHint::I64 | TypeHint::I32)) {
            let neg_ones: Vec<TestValue> = param_types
                .iter()
                .map(|t| match t {
                    TypeHint::I64 | TypeHint::I32 | TypeHint::I16 | TypeHint::I8 => {
                        TestValue::Int(-1)
                    }
                    _ => self.zero_value(t),
                })
                .collect();
            cases.push(TestCase {
                name: format!("{}_boundary_neg_one", function_name),
                function_name: function_name.to_string(),
                inputs: neg_ones,
                properties: vec![Property::DoesNotCrash],
                category: TestCategory::Boundary,
            });
        }

        cases
    }

    fn generate_property_tests(
        &self,
        function_name: &str,
        param_types: &[TypeHint],
        return_type: &TypeHint,
    ) -> Vec<TestCase> {
        let mut cases = vec![];
        let name = function_name.to_lowercase();

        // Heuristic: "add", "sum", "max", "min" with 2 params → commutative
        if param_types.len() == 2
            && param_types[0] == param_types[1]
            && (name.contains("add")
                || name.contains("sum")
                || name.contains("max")
                || name.contains("min")
                || name.contains("mul"))
        {
            let inputs: Vec<TestValue> = param_types
                .iter()
                .map(|t| self.random_value(t, &mut rand::thread_rng()))
                .collect();
            cases.push(TestCase {
                name: format!("{}_property_commutative", function_name),
                function_name: function_name.to_string(),
                inputs,
                properties: vec![Property::Commutative],
                category: TestCategory::Property,
            });
        }

        // Heuristic: "abs", "normalize", "clamp" → idempotent
        if param_types.len() >= 1
            && (name.contains("abs")
                || name.contains("normalize")
                || name.contains("clamp")
                || name.contains("sort"))
        {
            let inputs: Vec<TestValue> = param_types
                .iter()
                .map(|t| self.random_value(t, &mut rand::thread_rng()))
                .collect();
            cases.push(TestCase {
                name: format!("{}_property_idempotent", function_name),
                function_name: function_name.to_string(),
                inputs,
                properties: vec![Property::Idempotent],
                category: TestCategory::Property,
            });
        }

        // Heuristic: functions returning bool → non-zero check not needed
        // Functions returning i64 with "count", "len", "size" → returns >= 0
        if matches!(return_type, TypeHint::I64 | TypeHint::I32 | TypeHint::U64 | TypeHint::U32)
            && (name.contains("count")
                || name.contains("len")
                || name.contains("size")
                || name.contains("abs"))
        {
            let inputs: Vec<TestValue> = param_types
                .iter()
                .map(|t| self.random_value(t, &mut rand::thread_rng()))
                .collect();
            cases.push(TestCase {
                name: format!("{}_property_non_negative", function_name),
                function_name: function_name.to_string(),
                inputs,
                properties: vec![Property::ReturnsInRange(0, i64::MAX)],
                category: TestCategory::Property,
            });
        }

        cases
    }

    fn random_value(&self, ty: &TypeHint, rng: &mut impl Rng) -> TestValue {
        match ty {
            TypeHint::I64 => TestValue::Int(rng.gen_range(-1000..=1000)),
            TypeHint::I32 => TestValue::Int(rng.gen_range(-1000..=1000)),
            TypeHint::I16 => TestValue::Int(rng.gen_range(-100..=100)),
            TypeHint::I8 => TestValue::Int(rng.gen_range(-128..=127)),
            TypeHint::U64 | TypeHint::U32 => TestValue::Int(rng.gen_range(0..=1000)),
            TypeHint::U16 => TestValue::Int(rng.gen_range(0..=100)),
            TypeHint::U8 => TestValue::Int(rng.gen_range(0..=255)),
            TypeHint::F64 | TypeHint::F32 => {
                TestValue::Float(rng.gen_range(-100.0..=100.0))
            }
            TypeHint::Bool => TestValue::Bool(rng.gen_bool(0.5)),
            TypeHint::Str => {
                let len = rng.gen_range(0..=10);
                let s: String = (0..len)
                    .map(|_| rng.gen_range(b'a'..=b'z') as char)
                    .collect();
                TestValue::Str(s)
            }
            TypeHint::Array(inner) => {
                let len = rng.gen_range(0..=5);
                TestValue::Array((0..len).map(|_| self.random_value(inner, rng)).collect())
            }
            TypeHint::Tuple(types) => {
                TestValue::Tuple(types.iter().map(|t| self.random_value(t, rng)).collect())
            }
            TypeHint::Unknown => TestValue::Int(0),
        }
    }

    fn zero_value(&self, ty: &TypeHint) -> TestValue {
        match ty {
            TypeHint::I64 | TypeHint::I32 | TypeHint::I16 | TypeHint::I8
            | TypeHint::U64 | TypeHint::U32 | TypeHint::U16 | TypeHint::U8 => TestValue::Int(0),
            TypeHint::F64 | TypeHint::F32 => TestValue::Float(0.0),
            TypeHint::Bool => TestValue::Bool(false),
            TypeHint::Str => TestValue::Str(String::new()),
            TypeHint::Array(_) => TestValue::Array(vec![]),
            TypeHint::Tuple(types) => {
                TestValue::Tuple(types.iter().map(|t| self.zero_value(t)).collect())
            }
            TypeHint::Unknown => TestValue::Int(0),
        }
    }

    fn max_value(&self, ty: &TypeHint) -> TestValue {
        match ty {
            TypeHint::I64 => TestValue::Int(i64::MAX),
            TypeHint::I32 => TestValue::Int(i32::MAX as i64),
            TypeHint::I16 => TestValue::Int(i16::MAX as i64),
            TypeHint::I8 => TestValue::Int(i8::MAX as i64),
            TypeHint::U64 => TestValue::Int(i64::MAX), // can't exceed i64
            TypeHint::U32 => TestValue::Int(u32::MAX as i64),
            TypeHint::U16 => TestValue::Int(u16::MAX as i64),
            TypeHint::U8 => TestValue::Int(u8::MAX as i64),
            TypeHint::F64 | TypeHint::F32 => TestValue::Float(f64::MAX),
            TypeHint::Bool => TestValue::Bool(true),
            TypeHint::Str => TestValue::Str("z".repeat(100)),
            _ => self.zero_value(ty),
        }
    }

    fn min_value(&self, ty: &TypeHint) -> TestValue {
        match ty {
            TypeHint::I64 => TestValue::Int(i64::MIN),
            TypeHint::I32 => TestValue::Int(i32::MIN as i64),
            TypeHint::I16 => TestValue::Int(i16::MIN as i64),
            TypeHint::I8 => TestValue::Int(i8::MIN as i64),
            TypeHint::U64 | TypeHint::U32 | TypeHint::U16 | TypeHint::U8 => TestValue::Int(0),
            TypeHint::F64 | TypeHint::F32 => TestValue::Float(f64::MIN),
            TypeHint::Bool => TestValue::Bool(false),
            TypeHint::Str => TestValue::Str(String::new()),
            _ => self.zero_value(ty),
        }
    }
}

impl Default for TestGenerator {
    fn default() -> Self {
        Self::new()
    }
}
