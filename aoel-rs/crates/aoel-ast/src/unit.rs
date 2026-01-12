//! AOEL Unit (top-level) definitions

use aoel_lexer::Span;
use serde::{Deserialize, Serialize};
use crate::{
    QualifiedName,
    MetaBlock, InputBlock, OutputBlock, IntentBlock,
    ConstraintBlock, FlowBlock, ExecutionBlock, VerifyBlock,
};

/// Unit types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnitKind {
    Function,
    Service,
    Pipeline,
    Module,
}

impl UnitKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            UnitKind::Function => "FUNCTION",
            UnitKind::Service => "SERVICE",
            UnitKind::Pipeline => "PIPELINE",
            UnitKind::Module => "MODULE",
        }
    }
}

/// Version string (e.g., "V1.0.0")
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub span: Span,
}

impl Version {
    pub fn new(major: u32, minor: u32, patch: u32, span: Span) -> Self {
        Self {
            major,
            minor,
            patch,
            span,
        }
    }

    pub fn parse(s: &str, span: Span) -> Option<Self> {
        let s = s.strip_prefix('V')?;
        let mut parts = s.split('.');

        let major = parts.next()?.parse().ok()?;
        let minor = parts.next()?.parse().ok()?;
        let patch = parts.next()?.parse().ok()?;

        Some(Self::new(major, minor, patch, span))
    }

    pub fn to_string(&self) -> String {
        format!("V{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// UNIT block header
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnitHeader {
    pub kind: UnitKind,
    pub name: QualifiedName,
    pub version: Option<Version>,
    pub span: Span,
}

/// Complete AOEL Unit (the top-level AST node)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Unit {
    pub header: UnitHeader,
    pub meta: MetaBlock,
    pub input: InputBlock,
    pub output: OutputBlock,
    pub intent: IntentBlock,
    pub constraint: ConstraintBlock,
    pub flow: FlowBlock,
    pub execution: ExecutionBlock,
    pub verify: VerifyBlock,
    pub span: Span,
}

impl Unit {
    /// Get the unit's full name
    pub fn full_name(&self) -> String {
        self.header.name.full_name()
    }

    /// Get the unit's kind
    pub fn kind(&self) -> UnitKind {
        self.header.kind
    }

    /// Get the unit's version, if specified
    pub fn version(&self) -> Option<&Version> {
        self.header.version.as_ref()
    }
}

impl std::fmt::Display for Unit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {}",
            self.header.kind.as_str(),
            self.full_name()
        )?;

        if let Some(version) = &self.header.version {
            write!(f, " {}", version.to_string())?;
        }

        Ok(())
    }
}
