//! NaN-boxing implementation for high-performance Value representation
//!
//! NaN-boxing encodes all values in 64 bits using IEEE 754 NaN space:
//! - Floats: Direct IEEE 754 representation
//! - Integers: 48-bit signed integers (fits most use cases)
//! - Pointers: 48-bit pointers (sufficient for current architectures)
//! - Small types: Bool, Void encoded in tag bits
//!
//! This eliminates heap allocation for small values and enables O(1) clone.

use std::collections::HashMap;
use std::rc::Rc;

/// NaN-boxing bit patterns
///
/// IEEE 754 double: sign(1) | exponent(11) | mantissa(52)
/// NaN condition: exponent = 0x7FF and mantissa != 0
///
/// Our encoding:
/// - Float: Any valid double that's not our special NaN patterns
/// - Tagged values: 0x7FF8_XXXX_XXXX_XXXX (quiet NaN with tag in upper bits)
///
/// Tag layout (bits 48-51):
/// 0000 = Float (not a tagged value)
/// 0001 = Int (48-bit signed)
/// 0010 = Bool (true/false in bit 0)
/// 0011 = Void
/// 0100 = Pointer to Array (Rc<Vec<NanBoxedValue>>)
/// 0101 = Pointer to Map (Rc<HashMap<String, NanBoxedValue>>)
/// 0110 = Pointer to String (Rc<String>)
/// 0111 = Pointer to Struct (same as Map)
/// 1000 = Pointer to Closure
/// 1001 = Pointer to Bytes (Rc<Vec<u8>>)
/// 1010 = Error (pointer to String)
/// 1011 = Optional (pointer or special None marker)

const QNAN: u64 = 0x7FF8_0000_0000_0000; // Quiet NaN base
const TAG_MASK: u64 = 0x000F_0000_0000_0000; // Tag bits (48-51)
const PAYLOAD_MASK: u64 = 0x0000_FFFF_FFFF_FFFF; // Lower 48 bits

// Tags
const TAG_INT: u64 = 0x0001_0000_0000_0000;
const TAG_BOOL: u64 = 0x0002_0000_0000_0000;
const TAG_VOID: u64 = 0x0003_0000_0000_0000;
const TAG_ARRAY: u64 = 0x0004_0000_0000_0000;
const TAG_MAP: u64 = 0x0005_0000_0000_0000;
const TAG_STRING: u64 = 0x0006_0000_0000_0000;
const TAG_STRUCT: u64 = 0x0007_0000_0000_0000;
const TAG_CLOSURE: u64 = 0x0008_0000_0000_0000;
const TAG_BYTES: u64 = 0x0009_0000_0000_0000;
const TAG_ERROR: u64 = 0x000A_0000_0000_0000;
const TAG_OPTIONAL: u64 = 0x000B_0000_0000_0000;

// Special values
const VOID_VALUE: u64 = QNAN | TAG_VOID;
const TRUE_VALUE: u64 = QNAN | TAG_BOOL | 1;
const FALSE_VALUE: u64 = QNAN | TAG_BOOL | 0;
const NONE_VALUE: u64 = QNAN | TAG_OPTIONAL; // None marker (payload = 0)

/// Closure data stored on heap
#[derive(Debug, Clone)]
pub struct ClosureData {
    pub params: Vec<String>,
    pub captured: Rc<HashMap<String, NanBoxedValue>>,
    pub body_id: usize,
}

/// NaN-boxed value - all values fit in 64 bits
///
/// This enables:
/// - O(1) clone (just copy 64 bits)
/// - No heap allocation for Int, Float, Bool, Void
/// - Cache-friendly stack operations
///
/// Note: Heap values (Array, Map, String) use Rc internally,
/// so cloning is still O(1) via reference count increment.
/// We don't implement Drop to allow Copy semantics - Rc handles cleanup.
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct NanBoxedValue(u64);

impl NanBoxedValue {
    // ========================================================================
    // Constructors
    // ========================================================================

    /// Create Void value
    #[inline(always)]
    pub const fn void() -> Self {
        Self(VOID_VALUE)
    }

    /// Create Bool value
    #[inline(always)]
    pub const fn bool(b: bool) -> Self {
        Self(if b { TRUE_VALUE } else { FALSE_VALUE })
    }

    /// Create Int value (48-bit signed)
    #[inline(always)]
    pub fn int(n: i64) -> Self {
        // Sign-extend 48-bit to 64-bit for storage
        let payload = (n as u64) & PAYLOAD_MASK;
        Self(QNAN | TAG_INT | payload)
    }

    /// Create Float value
    #[inline(always)]
    pub fn float(f: f64) -> Self {
        let bits = f.to_bits();
        // If it's a NaN, we need to handle it specially to avoid collision
        if (bits & 0x7FF0_0000_0000_0000) == 0x7FF0_0000_0000_0000 && (bits & PAYLOAD_MASK) != 0 {
            // It's a NaN - store as a special float NaN
            Self(f64::NAN.to_bits())
        } else {
            Self(bits)
        }
    }

    /// Create Array value
    #[inline]
    pub fn array(arr: Vec<NanBoxedValue>) -> Self {
        let rc = Rc::new(arr);
        let ptr = Rc::into_raw(rc) as u64;
        debug_assert!(ptr & !PAYLOAD_MASK == 0, "Pointer too large for NaN-boxing");
        Self(QNAN | TAG_ARRAY | (ptr & PAYLOAD_MASK))
    }

    /// Create Map value
    #[inline]
    pub fn map(map: HashMap<String, NanBoxedValue>) -> Self {
        let rc = Rc::new(map);
        let ptr = Rc::into_raw(rc) as u64;
        Self(QNAN | TAG_MAP | (ptr & PAYLOAD_MASK))
    }

    /// Create String value
    #[inline]
    pub fn string(s: String) -> Self {
        let rc = Rc::new(s);
        let ptr = Rc::into_raw(rc) as u64;
        Self(QNAN | TAG_STRING | (ptr & PAYLOAD_MASK))
    }

    /// Create Struct value (same representation as Map)
    #[inline]
    pub fn struct_val(fields: HashMap<String, NanBoxedValue>) -> Self {
        let rc = Rc::new(fields);
        let ptr = Rc::into_raw(rc) as u64;
        Self(QNAN | TAG_STRUCT | (ptr & PAYLOAD_MASK))
    }

    /// Create Closure value
    #[inline]
    pub fn closure(params: Vec<String>, captured: HashMap<String, NanBoxedValue>, body_id: usize) -> Self {
        let data = ClosureData {
            params,
            captured: Rc::new(captured),
            body_id,
        };
        let rc = Rc::new(data);
        let ptr = Rc::into_raw(rc) as u64;
        Self(QNAN | TAG_CLOSURE | (ptr & PAYLOAD_MASK))
    }

    /// Create Bytes value
    #[inline]
    pub fn bytes(b: Vec<u8>) -> Self {
        let rc = Rc::new(b);
        let ptr = Rc::into_raw(rc) as u64;
        Self(QNAN | TAG_BYTES | (ptr & PAYLOAD_MASK))
    }

    /// Create Error value
    #[inline]
    pub fn error(msg: String) -> Self {
        let rc = Rc::new(msg);
        let ptr = Rc::into_raw(rc) as u64;
        Self(QNAN | TAG_ERROR | (ptr & PAYLOAD_MASK))
    }

    /// Create Some(value)
    #[inline]
    pub fn some(value: NanBoxedValue) -> Self {
        let boxed = Box::new(value);
        let ptr = Box::into_raw(boxed) as u64;
        Self(QNAN | TAG_OPTIONAL | (ptr & PAYLOAD_MASK) | 1) // bit 0 = 1 means Some
    }

    /// Create None
    #[inline(always)]
    pub const fn none() -> Self {
        Self(NONE_VALUE)
    }

    // ========================================================================
    // Type checking
    // ========================================================================

    #[inline(always)]
    fn is_float_bits(bits: u64) -> bool {
        // Not a quiet NaN with our tag pattern
        (bits & 0x7FF8_0000_0000_0000) != QNAN
    }

    #[inline(always)]
    pub fn is_float(&self) -> bool {
        Self::is_float_bits(self.0)
    }

    #[inline(always)]
    pub fn is_int(&self) -> bool {
        (self.0 & (QNAN | TAG_MASK)) == (QNAN | TAG_INT)
    }

    #[inline(always)]
    pub fn is_bool(&self) -> bool {
        (self.0 & (QNAN | TAG_MASK)) == (QNAN | TAG_BOOL)
    }

    #[inline(always)]
    pub fn is_void(&self) -> bool {
        self.0 == VOID_VALUE
    }

    #[inline(always)]
    pub fn is_array(&self) -> bool {
        (self.0 & (QNAN | TAG_MASK)) == (QNAN | TAG_ARRAY)
    }

    #[inline(always)]
    pub fn is_map(&self) -> bool {
        (self.0 & (QNAN | TAG_MASK)) == (QNAN | TAG_MAP)
    }

    #[inline(always)]
    pub fn is_string(&self) -> bool {
        (self.0 & (QNAN | TAG_MASK)) == (QNAN | TAG_STRING)
    }

    #[inline(always)]
    pub fn is_struct(&self) -> bool {
        (self.0 & (QNAN | TAG_MASK)) == (QNAN | TAG_STRUCT)
    }

    #[inline(always)]
    pub fn is_closure(&self) -> bool {
        (self.0 & (QNAN | TAG_MASK)) == (QNAN | TAG_CLOSURE)
    }

    // ========================================================================
    // Value extraction
    // ========================================================================

    /// Get as i64 (unchecked - caller must verify is_int())
    #[inline(always)]
    pub fn as_int_unchecked(&self) -> i64 {
        // Sign-extend from 48 bits
        let payload = self.0 & PAYLOAD_MASK;
        if payload & 0x0000_8000_0000_0000 != 0 {
            // Negative - sign extend
            (payload | 0xFFFF_0000_0000_0000) as i64
        } else {
            payload as i64
        }
    }

    /// Get as i64 (checked)
    #[inline(always)]
    pub fn as_int(&self) -> Option<i64> {
        if self.is_int() {
            Some(self.as_int_unchecked())
        } else if self.is_float() {
            Some(self.as_float_unchecked() as i64)
        } else {
            None
        }
    }

    /// Get as f64 (unchecked)
    #[inline(always)]
    pub fn as_float_unchecked(&self) -> f64 {
        f64::from_bits(self.0)
    }

    /// Get as f64 (checked)
    #[inline(always)]
    pub fn as_float(&self) -> Option<f64> {
        if self.is_float() {
            Some(self.as_float_unchecked())
        } else if self.is_int() {
            Some(self.as_int_unchecked() as f64)
        } else {
            None
        }
    }

    /// Get as bool (unchecked)
    #[inline(always)]
    pub fn as_bool_unchecked(&self) -> bool {
        (self.0 & 1) != 0
    }

    /// Get as bool (checked)
    #[inline(always)]
    pub fn as_bool(&self) -> Option<bool> {
        if self.is_bool() {
            Some(self.as_bool_unchecked())
        } else {
            None
        }
    }

    /// Get pointer from tagged value (internal)
    #[inline(always)]
    fn get_ptr<T>(&self) -> *const T {
        (self.0 & PAYLOAD_MASK) as *const T
    }

    /// Get array reference
    #[inline]
    pub fn as_array(&self) -> Option<&Vec<NanBoxedValue>> {
        if self.is_array() {
            unsafe { Some(&*self.get_ptr::<Vec<NanBoxedValue>>()) }
        } else {
            None
        }
    }

    /// Get map reference
    #[inline]
    pub fn as_map(&self) -> Option<&HashMap<String, NanBoxedValue>> {
        if self.is_map() {
            unsafe { Some(&*self.get_ptr::<HashMap<String, NanBoxedValue>>()) }
        } else {
            None
        }
    }

    /// Get string reference
    #[inline]
    pub fn as_string(&self) -> Option<&str> {
        if self.is_string() {
            unsafe { Some(&*self.get_ptr::<String>()) }
        } else {
            None
        }
    }

    /// Get struct reference
    #[inline]
    pub fn as_struct(&self) -> Option<&HashMap<String, NanBoxedValue>> {
        if self.is_struct() {
            unsafe { Some(&*self.get_ptr::<HashMap<String, NanBoxedValue>>()) }
        } else {
            None
        }
    }

    /// Get closure data
    #[inline]
    pub fn as_closure(&self) -> Option<&ClosureData> {
        if self.is_closure() {
            unsafe { Some(&*self.get_ptr::<ClosureData>()) }
        } else {
            None
        }
    }

    // ========================================================================
    // Truthiness
    // ========================================================================

    #[inline]
    pub fn is_truthy(&self) -> bool {
        if self.is_void() {
            false
        } else if self.is_bool() {
            self.as_bool_unchecked()
        } else if self.is_int() {
            self.as_int_unchecked() != 0
        } else if self.is_float() {
            self.as_float_unchecked() != 0.0
        } else if self.is_string() {
            !self.as_string().unwrap().is_empty()
        } else if self.is_array() {
            !self.as_array().unwrap().is_empty()
        } else {
            true
        }
    }

    // ========================================================================
    // Reference counting for heap values
    // ========================================================================

    /// Increment reference count (for Clone semantics with heap values)
    #[inline]
    pub fn inc_ref(&self) {
        let tag = self.0 & TAG_MASK;
        if tag >= TAG_ARRAY && tag <= TAG_ERROR {
            // It's a pointer type - need to increment Rc
            match tag {
                TAG_ARRAY => unsafe {
                    Rc::increment_strong_count(self.get_ptr::<Vec<NanBoxedValue>>());
                },
                TAG_MAP | TAG_STRUCT => unsafe {
                    Rc::increment_strong_count(self.get_ptr::<HashMap<String, NanBoxedValue>>());
                },
                TAG_STRING | TAG_ERROR => unsafe {
                    Rc::increment_strong_count(self.get_ptr::<String>());
                },
                TAG_CLOSURE => unsafe {
                    Rc::increment_strong_count(self.get_ptr::<ClosureData>());
                },
                TAG_BYTES => unsafe {
                    Rc::increment_strong_count(self.get_ptr::<Vec<u8>>());
                },
                _ => {}
            }
        }
    }

    /// Decrement reference count (for Drop semantics)
    #[inline]
    pub fn dec_ref(&self) {
        let tag = self.0 & TAG_MASK;
        if tag >= TAG_ARRAY && tag <= TAG_ERROR {
            match tag {
                TAG_ARRAY => unsafe {
                    Rc::decrement_strong_count(self.get_ptr::<Vec<NanBoxedValue>>());
                },
                TAG_MAP | TAG_STRUCT => unsafe {
                    Rc::decrement_strong_count(self.get_ptr::<HashMap<String, NanBoxedValue>>());
                },
                TAG_STRING | TAG_ERROR => unsafe {
                    Rc::decrement_strong_count(self.get_ptr::<String>());
                },
                TAG_CLOSURE => unsafe {
                    Rc::decrement_strong_count(self.get_ptr::<ClosureData>());
                },
                TAG_BYTES => unsafe {
                    Rc::decrement_strong_count(self.get_ptr::<Vec<u8>>());
                },
                _ => {}
            }
        }
    }
}

// Note: We don't implement Drop because NanBoxedValue is Copy.
// Heap values (Array, Map, String) use Rc which handles its own reference counting.
// This means we intentionally "leak" the Rc's reference count increment from into_raw,
// but this is acceptable for a benchmark/prototype. In production, we would need
// a more sophisticated approach (arena allocation or explicit lifetime management).

impl std::fmt::Debug for NanBoxedValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_void() {
            write!(f, "Void")
        } else if self.is_bool() {
            write!(f, "Bool({})", self.as_bool_unchecked())
        } else if self.is_int() {
            write!(f, "Int({})", self.as_int_unchecked())
        } else if self.is_float() {
            write!(f, "Float({})", self.as_float_unchecked())
        } else if self.is_string() {
            write!(f, "String({:?})", self.as_string().unwrap())
        } else if self.is_array() {
            write!(f, "Array({:?})", self.as_array().unwrap())
        } else if self.is_map() {
            write!(f, "Map({:?})", self.as_map().unwrap())
        } else if self.is_closure() {
            write!(f, "Closure")
        } else {
            write!(f, "Unknown(0x{:016x})", self.0)
        }
    }
}

impl std::fmt::Display for NanBoxedValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_void() {
            write!(f, "VOID")
        } else if self.is_bool() {
            write!(f, "{}", self.as_bool_unchecked())
        } else if self.is_int() {
            write!(f, "{}", self.as_int_unchecked())
        } else if self.is_float() {
            write!(f, "{}", self.as_float_unchecked())
        } else if self.is_string() {
            write!(f, "{}", self.as_string().unwrap())
        } else if self.is_array() {
            let arr = self.as_array().unwrap();
            write!(f, "[")?;
            for (i, v) in arr.iter().enumerate() {
                if i > 0 { write!(f, ", ")?; }
                write!(f, "{}", v)?;
            }
            write!(f, "]")
        } else if self.is_map() {
            let m = self.as_map().unwrap();
            write!(f, "{{")?;
            for (i, (k, v)) in m.iter().enumerate() {
                if i > 0 { write!(f, ", ")?; }
                write!(f, "{}: {}", k, v)?;
            }
            write!(f, "}}")
        } else {
            write!(f, "<value>")
        }
    }
}

impl PartialEq for NanBoxedValue {
    fn eq(&self, other: &Self) -> bool {
        if self.is_int() && other.is_int() {
            self.as_int_unchecked() == other.as_int_unchecked()
        } else if self.is_float() && other.is_float() {
            self.as_float_unchecked() == other.as_float_unchecked()
        } else if self.is_bool() && other.is_bool() {
            self.as_bool_unchecked() == other.as_bool_unchecked()
        } else if self.is_void() && other.is_void() {
            true
        } else if self.is_string() && other.is_string() {
            self.as_string() == other.as_string()
        } else if self.is_array() && other.is_array() {
            self.as_array() == other.as_array()
        } else if self.is_map() && other.is_map() {
            self.as_map() == other.as_map()
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_int_roundtrip() {
        for n in [-1000000i64, -1, 0, 1, 42, 1000000] {
            let v = NanBoxedValue::int(n);
            assert!(v.is_int());
            assert_eq!(v.as_int_unchecked(), n);
        }
    }

    #[test]
    fn test_float_roundtrip() {
        for f in [0.0f64, 1.0, -1.0, 3.14159, f64::MAX, f64::MIN] {
            let v = NanBoxedValue::float(f);
            assert!(v.is_float());
            assert_eq!(v.as_float_unchecked(), f);
        }
    }

    #[test]
    fn test_bool() {
        let t = NanBoxedValue::bool(true);
        let f = NanBoxedValue::bool(false);
        assert!(t.is_bool());
        assert!(f.is_bool());
        assert!(t.as_bool_unchecked());
        assert!(!f.as_bool_unchecked());
    }

    #[test]
    fn test_void() {
        let v = NanBoxedValue::void();
        assert!(v.is_void());
    }

    #[test]
    fn test_string() {
        let s = NanBoxedValue::string("hello".to_string());
        assert!(s.is_string());
        assert_eq!(s.as_string(), Some("hello"));
    }

    #[test]
    fn test_array() {
        let arr = NanBoxedValue::array(vec![
            NanBoxedValue::int(1),
            NanBoxedValue::int(2),
            NanBoxedValue::int(3),
        ]);
        assert!(arr.is_array());
        let inner = arr.as_array().unwrap();
        assert_eq!(inner.len(), 3);
        assert_eq!(inner[0].as_int_unchecked(), 1);
    }

    #[test]
    fn test_clone_is_copy() {
        let v = NanBoxedValue::int(42);
        let v2 = v; // Copy
        assert_eq!(v.as_int_unchecked(), v2.as_int_unchecked());
    }
}
