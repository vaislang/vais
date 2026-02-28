//! Semantic versioning support
//!
//! Implements SemVer 2.0.0 spec for version parsing and comparison,
//! as well as version requirements (ranges) for dependency resolution.

use super::error::{RegistryError, RegistryResult};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fmt;
use std::str::FromStr;

/// A semantic version (major.minor.patch\[-prerelease\]\[+build\])
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Version {
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
    pub pre: Option<String>,
    pub build: Option<String>,
}

impl Version {
    pub fn new(major: u64, minor: u64, patch: u64) -> Self {
        Self {
            major,
            minor,
            patch,
            pre: None,
            build: None,
        }
    }

    pub fn with_pre(mut self, pre: impl Into<String>) -> Self {
        self.pre = Some(pre.into());
        self
    }

    pub fn with_build(mut self, build: impl Into<String>) -> Self {
        self.build = Some(build.into());
        self
    }

    /// Parse a version string
    pub fn parse(s: &str) -> RegistryResult<Self> {
        let s = s.trim();
        if s.is_empty() {
            return Err(RegistryError::InvalidVersion("empty version".into()));
        }

        // Split off build metadata
        let (main, build) = if let Some(idx) = s.find('+') {
            (&s[..idx], Some(s[idx + 1..].to_string()))
        } else {
            (s, None)
        };

        // Split off prerelease
        let (version_part, pre) = if let Some(idx) = main.find('-') {
            (&main[..idx], Some(main[idx + 1..].to_string()))
        } else {
            (main, None)
        };

        // Parse major.minor.patch
        let parts: Vec<&str> = version_part.split('.').collect();
        if parts.is_empty() || parts.len() > 3 {
            return Err(RegistryError::InvalidVersion(format!(
                "expected 1-3 version components, got {}",
                parts.len()
            )));
        }

        let major = parts[0]
            .parse::<u64>()
            .map_err(|_| RegistryError::InvalidVersion(format!("invalid major: {}", parts[0])))?;

        let minor = if parts.len() > 1 {
            parts[1].parse::<u64>().map_err(|_| {
                RegistryError::InvalidVersion(format!("invalid minor: {}", parts[1]))
            })?
        } else {
            0
        };

        let patch = if parts.len() > 2 {
            parts[2].parse::<u64>().map_err(|_| {
                RegistryError::InvalidVersion(format!("invalid patch: {}", parts[2]))
            })?
        } else {
            0
        };

        Ok(Self {
            major,
            minor,
            patch,
            pre,
            build,
        })
    }

    /// Check if this is a prerelease version
    pub fn is_prerelease(&self) -> bool {
        self.pre.is_some()
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)?;
        if let Some(ref pre) = self.pre {
            write!(f, "-{}", pre)?;
        }
        if let Some(ref build) = self.build {
            write!(f, "+{}", build)?;
        }
        Ok(())
    }
}

impl FromStr for Version {
    type Err = RegistryError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Version::parse(s)
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        // Compare major.minor.patch
        match self.major.cmp(&other.major) {
            Ordering::Equal => {}
            ord => return ord,
        }
        match self.minor.cmp(&other.minor) {
            Ordering::Equal => {}
            ord => return ord,
        }
        match self.patch.cmp(&other.patch) {
            Ordering::Equal => {}
            ord => return ord,
        }

        // Prerelease has lower precedence
        match (&self.pre, &other.pre) {
            (None, None) => Ordering::Equal,
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (Some(a), Some(b)) => compare_prerelease(a, b),
        }
        // Build metadata is ignored for comparison
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Compare prerelease identifiers per SemVer spec
fn compare_prerelease(a: &str, b: &str) -> Ordering {
    let a_parts: Vec<&str> = a.split('.').collect();
    let b_parts: Vec<&str> = b.split('.').collect();

    for (a_part, b_part) in a_parts.iter().zip(b_parts.iter()) {
        let a_num = a_part.parse::<u64>().ok();
        let b_num = b_part.parse::<u64>().ok();

        let ord = match (a_num, b_num) {
            (Some(an), Some(bn)) => an.cmp(&bn),
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (None, None) => a_part.cmp(b_part),
        };

        if ord != Ordering::Equal {
            return ord;
        }
    }

    a_parts.len().cmp(&b_parts.len())
}

/// Version requirement (e.g., "^1.0", ">=2.0,<3.0", "1.2.3")
#[derive(Debug, Clone)]
pub struct VersionReq {
    predicates: Vec<Predicate>,
}

// Custom serialization for VersionReq - serialize as string
impl serde::Serialize for VersionReq {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> serde::Deserialize<'de> for VersionReq {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        VersionReq::parse(&s).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone)]
struct Predicate {
    op: Op,
    version: Version,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Op {
    Exact,     // =1.0.0
    Greater,   // >1.0.0
    GreaterEq, // >=1.0.0
    Less,      // <1.0.0
    LessEq,    // <=1.0.0
    Caret,     // ^1.0.0
    Tilde,     // ~1.0.0
    Wildcard,  // * or 1.* or 1.2.*
}

impl VersionReq {
    /// Create a requirement that matches any version
    pub fn any() -> Self {
        Self {
            predicates: vec![Predicate {
                op: Op::Wildcard,
                version: Version::new(0, 0, 0),
            }],
        }
    }

    /// Create an exact version requirement
    pub fn exact(version: Version) -> Self {
        Self {
            predicates: vec![Predicate {
                op: Op::Exact,
                version,
            }],
        }
    }

    /// Parse a version requirement string
    pub fn parse(s: &str) -> RegistryResult<Self> {
        let s = s.trim();
        if s.is_empty() || s == "*" {
            return Ok(Self::any());
        }

        let mut predicates = Vec::new();

        // Split by comma for multiple predicates
        for part in s.split(',') {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }

            let predicate = parse_predicate(part)?;
            predicates.push(predicate);
        }

        if predicates.is_empty() {
            return Ok(Self::any());
        }

        Ok(Self { predicates })
    }

    /// Check if a version matches this requirement
    pub fn matches(&self, version: &Version) -> bool {
        self.predicates.iter().all(|p| p.matches(version))
    }

    /// Find the best matching version from a list
    pub fn best_match<'a>(&self, versions: &'a [Version]) -> Option<&'a Version> {
        versions
            .iter()
            .filter(|v| self.matches(v) && !v.is_prerelease())
            .max()
    }
}

impl fmt::Display for VersionReq {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let strs: Vec<String> = self.predicates.iter().map(|p| p.to_string()).collect();
        write!(f, "{}", strs.join(", "))
    }
}

impl FromStr for VersionReq {
    type Err = RegistryError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        VersionReq::parse(s)
    }
}

fn parse_predicate(s: &str) -> RegistryResult<Predicate> {
    let s = s.trim();

    // Check for operator prefix
    let (op, version_str) = if let Some(rest) = s.strip_prefix(">=") {
        (Op::GreaterEq, rest)
    } else if let Some(rest) = s.strip_prefix("<=") {
        (Op::LessEq, rest)
    } else if let Some(rest) = s.strip_prefix('>') {
        (Op::Greater, rest)
    } else if let Some(rest) = s.strip_prefix('<') {
        (Op::Less, rest)
    } else if let Some(rest) = s.strip_prefix('=') {
        (Op::Exact, rest)
    } else if let Some(rest) = s.strip_prefix('^') {
        (Op::Caret, rest)
    } else if let Some(rest) = s.strip_prefix('~') {
        (Op::Tilde, rest)
    } else if s.contains('*') {
        (Op::Wildcard, s)
    } else {
        // Default to caret (^) behavior like Cargo
        (Op::Caret, s)
    };

    // Handle wildcard specially
    if op == Op::Wildcard {
        let parts: Vec<&str> = version_str.split('.').collect();
        let major = if parts.is_empty() || parts[0] == "*" {
            0
        } else {
            parts[0]
                .parse::<u64>()
                .map_err(|_| RegistryError::InvalidVersionReq(format!("invalid version: {}", s)))?
        };
        let minor = if parts.len() < 2 || parts[1] == "*" {
            0
        } else {
            parts[1]
                .parse::<u64>()
                .map_err(|_| RegistryError::InvalidVersionReq(format!("invalid version: {}", s)))?
        };
        let patch = if parts.len() < 3 || parts[2] == "*" {
            0
        } else {
            parts[2]
                .parse::<u64>()
                .map_err(|_| RegistryError::InvalidVersionReq(format!("invalid version: {}", s)))?
        };

        return Ok(Predicate {
            op: Op::Wildcard,
            version: Version::new(major, minor, patch),
        });
    }

    let version = Version::parse(version_str.trim())?;

    Ok(Predicate { op, version })
}

impl Predicate {
    fn matches(&self, v: &Version) -> bool {
        match self.op {
            Op::Exact => v == &self.version,
            Op::Greater => v > &self.version,
            Op::GreaterEq => v >= &self.version,
            Op::Less => v < &self.version,
            Op::LessEq => v <= &self.version,
            Op::Caret => self.matches_caret(v),
            Op::Tilde => self.matches_tilde(v),
            Op::Wildcard => self.matches_wildcard(v),
        }
    }

    /// Caret (^) - compatible updates
    /// ^1.2.3 := >=1.2.3, <2.0.0
    /// ^0.2.3 := >=0.2.3, <0.3.0
    /// ^0.0.3 := >=0.0.3, <0.0.4
    fn matches_caret(&self, v: &Version) -> bool {
        if v < &self.version {
            return false;
        }

        if self.version.major != 0 {
            v.major == self.version.major
        } else if self.version.minor != 0 {
            v.major == 0 && v.minor == self.version.minor
        } else {
            v.major == 0 && v.minor == 0 && v.patch == self.version.patch
        }
    }

    /// Tilde (~) - patch-level changes
    /// ~1.2.3 := >=1.2.3, <1.3.0
    /// ~1.2 := >=1.2.0, <1.3.0
    fn matches_tilde(&self, v: &Version) -> bool {
        if v < &self.version {
            return false;
        }
        v.major == self.version.major && v.minor == self.version.minor
    }

    /// Wildcard (*) - any matching
    /// 1.* := >=1.0.0, <2.0.0
    /// 1.2.* := >=1.2.0, <1.3.0
    fn matches_wildcard(&self, v: &Version) -> bool {
        if self.version.major == 0 && self.version.minor == 0 && self.version.patch == 0 {
            // Full wildcard (*)
            true
        } else if self.version.minor == 0 && self.version.patch == 0 {
            // Major wildcard (1.*)
            v.major == self.version.major
        } else if self.version.patch == 0 {
            // Minor wildcard (1.2.*)
            v.major == self.version.major && v.minor == self.version.minor
        } else {
            // Exact
            v == &self.version
        }
    }
}

impl fmt::Display for Predicate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.op {
            Op::Exact => write!(f, "={}", self.version),
            Op::Greater => write!(f, ">{}", self.version),
            Op::GreaterEq => write!(f, ">={}", self.version),
            Op::Less => write!(f, "<{}", self.version),
            Op::LessEq => write!(f, "<={}", self.version),
            Op::Caret => write!(f, "^{}", self.version),
            Op::Tilde => write!(f, "~{}", self.version),
            Op::Wildcard => write!(f, "*"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Version::new and builder tests ──

    #[test]
    fn test_version_new() {
        let v = Version::new(1, 2, 3);
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 3);
        assert!(v.pre.is_none());
        assert!(v.build.is_none());
    }

    #[test]
    fn test_version_with_pre() {
        let v = Version::new(1, 0, 0).with_pre("alpha");
        assert_eq!(v.pre, Some("alpha".to_string()));
        assert!(v.is_prerelease());
    }

    #[test]
    fn test_version_with_build() {
        let v = Version::new(1, 0, 0).with_build("20260101");
        assert_eq!(v.build, Some("20260101".to_string()));
        assert!(!v.is_prerelease());
    }

    #[test]
    fn test_version_with_pre_and_build() {
        let v = Version::new(1, 0, 0).with_pre("rc.1").with_build("build.42");
        assert!(v.is_prerelease());
        assert_eq!(v.pre, Some("rc.1".to_string()));
        assert_eq!(v.build, Some("build.42".to_string()));
    }

    #[test]
    fn test_version_not_prerelease() {
        let v = Version::new(1, 0, 0);
        assert!(!v.is_prerelease());
    }

    // ── Version::parse tests ──

    #[test]
    fn test_version_parse() {
        assert_eq!(Version::parse("1.2.3").unwrap(), Version::new(1, 2, 3));
        assert_eq!(
            Version::parse("1.2.3-alpha").unwrap(),
            Version::new(1, 2, 3).with_pre("alpha")
        );
        assert_eq!(
            Version::parse("1.2.3+build").unwrap(),
            Version::new(1, 2, 3).with_build("build")
        );
        assert_eq!(
            Version::parse("1.2.3-beta.1+sha.abc").unwrap(),
            Version::new(1, 2, 3)
                .with_pre("beta.1")
                .with_build("sha.abc")
        );
    }

    #[test]
    fn test_version_parse_major_only() {
        let v = Version::parse("5").unwrap();
        assert_eq!(v, Version::new(5, 0, 0));
    }

    #[test]
    fn test_version_parse_major_minor() {
        let v = Version::parse("3.7").unwrap();
        assert_eq!(v, Version::new(3, 7, 0));
    }

    #[test]
    fn test_version_parse_trims_whitespace() {
        let v = Version::parse("  1.2.3  ").unwrap();
        assert_eq!(v, Version::new(1, 2, 3));
    }

    #[test]
    fn test_version_parse_empty_fails() {
        assert!(Version::parse("").is_err());
    }

    #[test]
    fn test_version_parse_whitespace_only_fails() {
        assert!(Version::parse("   ").is_err());
    }

    #[test]
    fn test_version_parse_invalid_major() {
        assert!(Version::parse("abc").is_err());
    }

    #[test]
    fn test_version_parse_invalid_minor() {
        assert!(Version::parse("1.xyz").is_err());
    }

    #[test]
    fn test_version_parse_invalid_patch() {
        assert!(Version::parse("1.2.xyz").is_err());
    }

    #[test]
    fn test_version_parse_too_many_components() {
        assert!(Version::parse("1.2.3.4").is_err());
    }

    #[test]
    fn test_version_parse_zero() {
        let v = Version::parse("0.0.0").unwrap();
        assert_eq!(v, Version::new(0, 0, 0));
    }

    #[test]
    fn test_version_parse_large_numbers() {
        let v = Version::parse("100.200.300").unwrap();
        assert_eq!(v, Version::new(100, 200, 300));
    }

    #[test]
    fn test_version_parse_prerelease_with_dots() {
        let v = Version::parse("1.0.0-alpha.1.2").unwrap();
        assert_eq!(v.pre, Some("alpha.1.2".to_string()));
    }

    #[test]
    fn test_version_parse_build_metadata() {
        let v = Version::parse("1.0.0+sha.45678").unwrap();
        assert_eq!(v.build, Some("sha.45678".to_string()));
        assert!(!v.is_prerelease());
    }

    // ── Version Display tests ──

    #[test]
    fn test_version_display_simple() {
        assert_eq!(Version::new(1, 2, 3).to_string(), "1.2.3");
    }

    #[test]
    fn test_version_display_with_pre() {
        assert_eq!(
            Version::new(1, 0, 0).with_pre("alpha").to_string(),
            "1.0.0-alpha"
        );
    }

    #[test]
    fn test_version_display_with_build() {
        assert_eq!(
            Version::new(1, 0, 0).with_build("build.1").to_string(),
            "1.0.0+build.1"
        );
    }

    #[test]
    fn test_version_display_with_pre_and_build() {
        assert_eq!(
            Version::new(1, 0, 0)
                .with_pre("beta")
                .with_build("sha.abc")
                .to_string(),
            "1.0.0-beta+sha.abc"
        );
    }

    // ── Version FromStr tests ──

    #[test]
    fn test_version_from_str() {
        let v: Version = "2.5.1".parse().unwrap();
        assert_eq!(v, Version::new(2, 5, 1));
    }

    #[test]
    fn test_version_from_str_error() {
        let result: Result<Version, _> = "invalid".parse();
        assert!(result.is_err());
    }

    // ── Version comparison tests ──

    #[test]
    fn test_version_compare() {
        let v1 = Version::parse("1.0.0").unwrap();
        let v2 = Version::parse("2.0.0").unwrap();
        let v1_alpha = Version::parse("1.0.0-alpha").unwrap();

        assert!(v1 < v2);
        assert!(v1_alpha < v1); // prerelease < release
    }

    #[test]
    fn test_version_compare_major() {
        assert!(Version::new(1, 0, 0) < Version::new(2, 0, 0));
        assert!(Version::new(3, 0, 0) > Version::new(2, 0, 0));
    }

    #[test]
    fn test_version_compare_minor() {
        assert!(Version::new(1, 1, 0) < Version::new(1, 2, 0));
        assert!(Version::new(1, 5, 0) > Version::new(1, 3, 0));
    }

    #[test]
    fn test_version_compare_patch() {
        assert!(Version::new(1, 0, 1) < Version::new(1, 0, 2));
        assert!(Version::new(1, 0, 5) > Version::new(1, 0, 3));
    }

    #[test]
    fn test_version_compare_equal() {
        assert_eq!(
            Version::new(1, 2, 3).cmp(&Version::new(1, 2, 3)),
            Ordering::Equal
        );
    }

    #[test]
    fn test_version_compare_prerelease_less_than_release() {
        let pre = Version::new(1, 0, 0).with_pre("alpha");
        let rel = Version::new(1, 0, 0);
        assert!(pre < rel);
    }

    #[test]
    fn test_version_compare_prerelease_numeric() {
        let a = Version::new(1, 0, 0).with_pre("1");
        let b = Version::new(1, 0, 0).with_pre("2");
        assert!(a < b);
    }

    #[test]
    fn test_version_compare_prerelease_alpha_vs_numeric() {
        // Numeric identifiers have lower precedence than alphanumeric
        let num = Version::new(1, 0, 0).with_pre("1");
        let alpha = Version::new(1, 0, 0).with_pre("alpha");
        assert!(num < alpha);
    }

    #[test]
    fn test_version_compare_prerelease_alpha_ordering() {
        let a = Version::new(1, 0, 0).with_pre("alpha");
        let b = Version::new(1, 0, 0).with_pre("beta");
        assert!(a < b);
    }

    #[test]
    fn test_version_compare_prerelease_length() {
        let a = Version::new(1, 0, 0).with_pre("alpha");
        let b = Version::new(1, 0, 0).with_pre("alpha.1");
        assert!(a < b); // shorter set has lower precedence
    }

    #[test]
    fn test_version_compare_build_metadata_ignored() {
        let a = Version::new(1, 0, 0).with_build("build.1");
        let b = Version::new(1, 0, 0).with_build("build.2");
        assert_eq!(a.cmp(&b), Ordering::Equal);
    }

    #[test]
    fn test_version_partial_ord() {
        let a = Version::new(1, 0, 0);
        let b = Version::new(2, 0, 0);
        assert!(a.partial_cmp(&b) == Some(Ordering::Less));
    }

    // ── Version equality and hash tests ──

    #[test]
    fn test_version_equality() {
        let a = Version::new(1, 2, 3);
        let b = Version::new(1, 2, 3);
        assert_eq!(a, b);
    }

    #[test]
    fn test_version_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(Version::new(1, 0, 0));
        set.insert(Version::new(1, 0, 0)); // duplicate
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn test_version_serde_roundtrip() {
        let v = Version::new(1, 2, 3).with_pre("alpha").with_build("build");
        let json = serde_json::to_string(&v).unwrap();
        let parsed: Version = serde_json::from_str(&json).unwrap();
        assert_eq!(v, parsed);
    }

    // ── VersionReq tests ──

    #[test]
    fn test_version_req_any() {
        let req = VersionReq::any();
        assert!(req.matches(&Version::new(0, 0, 0)));
        assert!(req.matches(&Version::new(99, 99, 99)));
    }

    #[test]
    fn test_version_req_exact() {
        let req = VersionReq::exact(Version::new(1, 2, 3));
        assert!(req.matches(&Version::new(1, 2, 3)));
        assert!(!req.matches(&Version::new(1, 2, 4)));
    }

    #[test]
    fn test_version_req_parse_empty() {
        let req = VersionReq::parse("").unwrap();
        assert!(req.matches(&Version::new(5, 0, 0))); // any
    }

    #[test]
    fn test_version_req_parse_star() {
        let req = VersionReq::parse("*").unwrap();
        assert!(req.matches(&Version::new(1, 0, 0)));
        assert!(req.matches(&Version::new(99, 0, 0)));
    }

    #[test]
    fn test_version_req_caret() {
        let req = VersionReq::parse("^1.2.3").unwrap();
        assert!(req.matches(&Version::parse("1.2.3").unwrap()));
        assert!(req.matches(&Version::parse("1.2.4").unwrap()));
        assert!(req.matches(&Version::parse("1.9.0").unwrap()));
        assert!(!req.matches(&Version::parse("2.0.0").unwrap()));
        assert!(!req.matches(&Version::parse("1.2.2").unwrap()));
    }

    #[test]
    fn test_version_req_caret_zero_major() {
        // ^0.2.3 := >=0.2.3, <0.3.0
        let req = VersionReq::parse("^0.2.3").unwrap();
        assert!(req.matches(&Version::new(0, 2, 3)));
        assert!(req.matches(&Version::new(0, 2, 9)));
        assert!(!req.matches(&Version::new(0, 3, 0)));
        assert!(!req.matches(&Version::new(1, 0, 0)));
    }

    #[test]
    fn test_version_req_caret_zero_zero() {
        // ^0.0.3 := >=0.0.3, <0.0.4
        let req = VersionReq::parse("^0.0.3").unwrap();
        assert!(req.matches(&Version::new(0, 0, 3)));
        assert!(!req.matches(&Version::new(0, 0, 4)));
        assert!(!req.matches(&Version::new(0, 1, 0)));
    }

    #[test]
    fn test_version_req_tilde() {
        let req = VersionReq::parse("~1.2.3").unwrap();
        assert!(req.matches(&Version::parse("1.2.3").unwrap()));
        assert!(req.matches(&Version::parse("1.2.9").unwrap()));
        assert!(!req.matches(&Version::parse("1.3.0").unwrap()));
    }

    #[test]
    fn test_version_req_tilde_minor_range() {
        // ~1.2 := >=1.2.0, <1.3.0
        let req = VersionReq::parse("~1.2.0").unwrap();
        assert!(req.matches(&Version::new(1, 2, 0)));
        assert!(req.matches(&Version::new(1, 2, 5)));
        assert!(!req.matches(&Version::new(1, 3, 0)));
    }

    #[test]
    fn test_version_req_exact_operator() {
        let req = VersionReq::parse("=1.0.0").unwrap();
        assert!(req.matches(&Version::new(1, 0, 0)));
        assert!(!req.matches(&Version::new(1, 0, 1)));
    }

    #[test]
    fn test_version_req_greater() {
        let req = VersionReq::parse(">1.0.0").unwrap();
        assert!(!req.matches(&Version::new(1, 0, 0)));
        assert!(req.matches(&Version::new(1, 0, 1)));
        assert!(req.matches(&Version::new(2, 0, 0)));
    }

    #[test]
    fn test_version_req_greater_eq() {
        let req = VersionReq::parse(">=1.0.0").unwrap();
        assert!(req.matches(&Version::new(1, 0, 0)));
        assert!(req.matches(&Version::new(1, 0, 1)));
        assert!(!req.matches(&Version::new(0, 9, 0)));
    }

    #[test]
    fn test_version_req_less() {
        let req = VersionReq::parse("<2.0.0").unwrap();
        assert!(req.matches(&Version::new(1, 9, 9)));
        assert!(!req.matches(&Version::new(2, 0, 0)));
    }

    #[test]
    fn test_version_req_less_eq() {
        let req = VersionReq::parse("<=2.0.0").unwrap();
        assert!(req.matches(&Version::new(2, 0, 0)));
        assert!(req.matches(&Version::new(1, 0, 0)));
        assert!(!req.matches(&Version::new(2, 0, 1)));
    }

    #[test]
    fn test_version_req_range() {
        let req = VersionReq::parse(">=1.0.0, <2.0.0").unwrap();
        assert!(req.matches(&Version::parse("1.0.0").unwrap()));
        assert!(req.matches(&Version::parse("1.5.0").unwrap()));
        assert!(!req.matches(&Version::parse("0.9.0").unwrap()));
        assert!(!req.matches(&Version::parse("2.0.0").unwrap()));
    }

    #[test]
    fn test_version_req_wildcard_major() {
        // 1.* matches 1.x.x
        let req = VersionReq::parse("1.*").unwrap();
        assert!(req.matches(&Version::new(1, 0, 0)));
        assert!(req.matches(&Version::new(1, 9, 9)));
        assert!(!req.matches(&Version::new(2, 0, 0)));
    }

    #[test]
    fn test_version_req_wildcard_minor() {
        // 1.2.* matches 1.2.x
        let req = VersionReq::parse("1.2.*").unwrap();
        assert!(req.matches(&Version::new(1, 2, 0)));
        assert!(req.matches(&Version::new(1, 2, 9)));
        assert!(!req.matches(&Version::new(1, 3, 0)));
    }

    #[test]
    fn test_version_req_default_caret() {
        // No operator defaults to caret
        let req = VersionReq::parse("1.2.3").unwrap();
        assert!(req.matches(&Version::new(1, 2, 3)));
        assert!(req.matches(&Version::new(1, 9, 0)));
        assert!(!req.matches(&Version::new(2, 0, 0)));
    }

    #[test]
    fn test_version_req_display() {
        let req = VersionReq::parse("^1.0.0").unwrap();
        let s = req.to_string();
        assert!(s.contains("1.0.0"));
    }

    #[test]
    fn test_version_req_from_str() {
        let req: VersionReq = "^1.0.0".parse().unwrap();
        assert!(req.matches(&Version::new(1, 5, 0)));
    }

    #[test]
    fn test_version_req_serde_roundtrip() {
        let req = VersionReq::parse("^1.2.3").unwrap();
        let json = serde_json::to_string(&req).unwrap();
        let parsed: VersionReq = serde_json::from_str(&json).unwrap();
        assert!(parsed.matches(&Version::new(1, 2, 3)));
    }

    // ── best_match tests ──

    #[test]
    fn test_best_match_basic() {
        let req = VersionReq::parse("^1.0.0").unwrap();
        let versions = vec![
            Version::new(1, 0, 0),
            Version::new(1, 1, 0),
            Version::new(1, 2, 0),
            Version::new(2, 0, 0),
        ];
        let best = req.best_match(&versions).unwrap();
        assert_eq!(best, &Version::new(1, 2, 0));
    }

    #[test]
    fn test_best_match_no_match() {
        let req = VersionReq::parse("^3.0.0").unwrap();
        let versions = vec![Version::new(1, 0, 0), Version::new(2, 0, 0)];
        assert!(req.best_match(&versions).is_none());
    }

    #[test]
    fn test_best_match_skips_prerelease() {
        let req = VersionReq::parse("^1.0.0").unwrap();
        let versions = vec![
            Version::new(1, 0, 0),
            Version::new(1, 1, 0).with_pre("alpha"),
            Version::new(1, 2, 0),
        ];
        let best = req.best_match(&versions).unwrap();
        assert_eq!(best, &Version::new(1, 2, 0));
    }

    #[test]
    fn test_best_match_empty_versions() {
        let req = VersionReq::parse("^1.0.0").unwrap();
        let versions: Vec<Version> = vec![];
        assert!(req.best_match(&versions).is_none());
    }

    #[test]
    fn test_best_match_all_prerelease() {
        let req = VersionReq::parse("^1.0.0").unwrap();
        let versions = vec![
            Version::new(1, 0, 0).with_pre("alpha"),
            Version::new(1, 1, 0).with_pre("beta"),
        ];
        assert!(req.best_match(&versions).is_none());
    }
}
