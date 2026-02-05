//! Security findings and severity levels

use vais_ast::Span;

/// Severity level for security findings
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    /// Informational finding (best practices, style, etc.)
    Info,
    /// Low severity - minor security concerns
    Low,
    /// Medium severity - notable security issues
    Medium,
    /// High severity - serious security vulnerabilities
    High,
    /// Critical severity - severe vulnerabilities requiring immediate attention
    Critical,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Info => write!(f, "INFO"),
            Severity::Low => write!(f, "LOW"),
            Severity::Medium => write!(f, "MEDIUM"),
            Severity::High => write!(f, "HIGH"),
            Severity::Critical => write!(f, "CRITICAL"),
        }
    }
}

/// Category of security finding
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FindingCategory {
    /// Buffer overflow risks
    BufferOverflow,
    /// Unsafe pointer arithmetic
    UnsafePointer,
    /// Injection vulnerabilities (SQL, command, etc.)
    Injection,
    /// Hardcoded secrets
    HardcodedSecret,
    /// Integer overflow/underflow
    IntegerOverflow,
    /// Use-after-free
    UseAfterFree,
    /// Memory leak
    MemoryLeak,
    /// Uninitialized memory access
    UninitializedMemory,
    /// Unchecked error handling
    UncheckedError,
}

impl std::fmt::Display for FindingCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FindingCategory::BufferOverflow => write!(f, "Buffer Overflow"),
            FindingCategory::UnsafePointer => write!(f, "Unsafe Pointer"),
            FindingCategory::Injection => write!(f, "Injection"),
            FindingCategory::HardcodedSecret => write!(f, "Hardcoded Secret"),
            FindingCategory::IntegerOverflow => write!(f, "Integer Overflow"),
            FindingCategory::UseAfterFree => write!(f, "Use After Free"),
            FindingCategory::MemoryLeak => write!(f, "Memory Leak"),
            FindingCategory::UninitializedMemory => write!(f, "Uninitialized Memory"),
            FindingCategory::UncheckedError => write!(f, "Unchecked Error"),
        }
    }
}

/// A security finding discovered during analysis
#[derive(Debug, Clone, PartialEq)]
pub struct SecurityFinding {
    /// Severity level of this finding
    pub severity: Severity,
    /// Category of the security issue
    pub category: FindingCategory,
    /// Human-readable description of the issue
    pub description: String,
    /// Source location where the issue was found
    pub location: Span,
    /// Recommendation for fixing the issue
    pub recommendation: String,
}

impl SecurityFinding {
    /// Creates a new security finding
    pub fn new(
        severity: Severity,
        category: FindingCategory,
        description: impl Into<String>,
        location: Span,
        recommendation: impl Into<String>,
    ) -> Self {
        Self {
            severity,
            category,
            description: description.into(),
            location,
            recommendation: recommendation.into(),
        }
    }

    /// Creates a buffer overflow finding
    pub fn buffer_overflow(description: impl Into<String>, location: Span) -> Self {
        Self::new(
            Severity::Critical,
            FindingCategory::BufferOverflow,
            description,
            location,
            "Add bounds checking before memory operations. Consider using safe wrappers or array types with automatic bounds checking.",
        )
    }

    /// Creates an unsafe pointer arithmetic finding
    pub fn unsafe_pointer(description: impl Into<String>, location: Span) -> Self {
        Self::new(
            Severity::High,
            FindingCategory::UnsafePointer,
            description,
            location,
            "Avoid raw pointer arithmetic. Use safe abstractions like slices or verify pointer validity before operations.",
        )
    }

    /// Creates an injection vulnerability finding
    pub fn injection(description: impl Into<String>, location: Span) -> Self {
        Self::new(
            Severity::Critical,
            FindingCategory::Injection,
            description,
            location,
            "Never concatenate user input into commands or queries. Use parameterized queries or validated input.",
        )
    }

    /// Creates a hardcoded secret finding
    pub fn hardcoded_secret(
        description: impl Into<String>,
        location: Span,
        severity: Severity,
    ) -> Self {
        Self::new(
            severity,
            FindingCategory::HardcodedSecret,
            description,
            location,
            "Store secrets in environment variables or secure configuration. Never commit secrets to source code.",
        )
    }

    /// Creates an integer overflow finding
    pub fn integer_overflow(description: impl Into<String>, location: Span) -> Self {
        Self::new(
            Severity::Medium,
            FindingCategory::IntegerOverflow,
            description,
            location,
            "Add overflow checking or use checked arithmetic operations. Validate input ranges before arithmetic.",
        )
    }

    /// Creates a use-after-free finding
    pub fn use_after_free(description: impl Into<String>, location: Span) -> Self {
        Self::new(
            Severity::Critical,
            FindingCategory::UseAfterFree,
            description,
            location,
            "Ensure memory is not accessed after being freed. Consider using RAII patterns or ownership tracking.",
        )
    }

    /// Creates a memory leak finding
    pub fn memory_leak(description: impl Into<String>, location: Span) -> Self {
        Self::new(
            Severity::Medium,
            FindingCategory::MemoryLeak,
            description,
            location,
            "Ensure all allocated memory is freed. Use RAII patterns or defer statements for cleanup.",
        )
    }

    /// Creates an uninitialized memory finding
    pub fn uninitialized_memory(description: impl Into<String>, location: Span) -> Self {
        Self::new(
            Severity::High,
            FindingCategory::UninitializedMemory,
            description,
            location,
            "Initialize all memory before use. Consider using safe initialization patterns.",
        )
    }

    /// Creates an unchecked error finding
    pub fn unchecked_error(description: impl Into<String>, location: Span) -> Self {
        Self::new(
            Severity::Low,
            FindingCategory::UncheckedError,
            description,
            location,
            "Check the result of operations that can fail. Use proper error handling patterns.",
        )
    }
}

impl std::fmt::Display for SecurityFinding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "[{}] {} at {}..{}",
            self.severity, self.category, self.location.start, self.location.end
        )?;
        writeln!(f, "  Description: {}", self.description)?;
        writeln!(f, "  Recommendation: {}", self.recommendation)?;
        Ok(())
    }
}
