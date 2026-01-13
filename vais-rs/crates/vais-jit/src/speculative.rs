//! 추측적 최적화 (Speculative Optimization)
//!
//! 타입 프로파일링 기반으로 특화된 코드를 생성하고,
//! 가정이 틀리면 deoptimization으로 인터프리터로 폴백합니다.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, AtomicBool, Ordering};
use std::sync::Arc;

/// 타입 가드 결과
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GuardResult {
    /// 가드 통과 - 최적화된 경로 사용
    Pass,
    /// 가드 실패 - deoptimization 필요
    Fail,
}

/// 타입 가드
#[derive(Debug, Clone)]
pub struct TypeGuard {
    /// 가드 ID
    pub id: u64,
    /// 기대하는 타입
    pub expected_type: SpecType,
    /// 가드 실패 횟수
    pub fail_count: Arc<AtomicU64>,
    /// 가드 비활성화 여부 (너무 많이 실패하면)
    pub disabled: Arc<AtomicBool>,
}

/// 추측적 타입
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SpecType {
    Int,
    Float,
    Bool,
    String,
    ArrayInt,
    ArrayFloat,
    ArrayMixed,
    Unknown,
}

impl SpecType {
    /// Value에서 타입 추출
    pub fn from_value(value: &vais_ir::Value) -> Self {
        match value {
            vais_ir::Value::Int(_) => SpecType::Int,
            vais_ir::Value::Float(_) => SpecType::Float,
            vais_ir::Value::Bool(_) => SpecType::Bool,
            vais_ir::Value::String(_) => SpecType::String,
            vais_ir::Value::Array(arr) => {
                if arr.is_empty() {
                    SpecType::ArrayMixed
                } else {
                    let first_type = Self::from_value(&arr[0]);
                    if arr.iter().all(|v| Self::from_value(v) == first_type) {
                        match first_type {
                            SpecType::Int => SpecType::ArrayInt,
                            SpecType::Float => SpecType::ArrayFloat,
                            _ => SpecType::ArrayMixed,
                        }
                    } else {
                        SpecType::ArrayMixed
                    }
                }
            }
            _ => SpecType::Unknown,
        }
    }

    /// 타입 이름
    pub fn name(&self) -> &'static str {
        match self {
            SpecType::Int => "Int",
            SpecType::Float => "Float",
            SpecType::Bool => "Bool",
            SpecType::String => "String",
            SpecType::ArrayInt => "Array<Int>",
            SpecType::ArrayFloat => "Array<Float>",
            SpecType::ArrayMixed => "Array<Mixed>",
            SpecType::Unknown => "Unknown",
        }
    }
}

impl TypeGuard {
    pub fn new(id: u64, expected_type: SpecType) -> Self {
        Self {
            id,
            expected_type,
            fail_count: Arc::new(AtomicU64::new(0)),
            disabled: Arc::new(AtomicBool::new(false)),
        }
    }

    /// 가드 체크
    pub fn check(&self, value: &vais_ir::Value) -> GuardResult {
        if self.disabled.load(Ordering::Relaxed) {
            return GuardResult::Pass; // 비활성화되면 항상 통과 (느린 경로로 폴백됨)
        }

        let actual_type = SpecType::from_value(value);
        if actual_type == self.expected_type {
            GuardResult::Pass
        } else {
            self.record_failure();
            GuardResult::Fail
        }
    }

    /// 가드 실패 기록
    fn record_failure(&self) {
        let count = self.fail_count.fetch_add(1, Ordering::Relaxed) + 1;

        // 10번 이상 실패하면 가드 비활성화
        if count >= 10 {
            self.disabled.store(true, Ordering::Relaxed);
        }
    }

    /// 가드가 유효한지 확인
    pub fn is_valid(&self) -> bool {
        !self.disabled.load(Ordering::Relaxed)
    }
}

/// 추측적 최적화 컨텍스트
pub struct SpeculativeContext {
    /// 함수별 타입 프로파일
    type_profiles: HashMap<String, TypeProfile>,
    /// 활성 가드들
    guards: HashMap<u64, TypeGuard>,
    /// 다음 가드 ID
    next_guard_id: AtomicU64,
    /// Deoptimization 카운터
    deopt_count: AtomicU64,
}

/// 함수의 타입 프로파일
#[derive(Debug, Clone)]
pub struct TypeProfile {
    /// 함수 이름
    pub name: String,
    /// 파라미터별 타입 히스토그램
    pub param_types: Vec<HashMap<SpecType, u64>>,
    /// 반환 타입 히스토그램
    pub return_types: HashMap<SpecType, u64>,
    /// 총 호출 횟수
    pub call_count: u64,
    /// 추측 특화 여부
    pub is_specialized: bool,
}

impl TypeProfile {
    pub fn new(name: String, param_count: usize) -> Self {
        Self {
            name,
            param_types: vec![HashMap::new(); param_count],
            return_types: HashMap::new(),
            call_count: 0,
            is_specialized: false,
        }
    }

    /// 호출 기록
    pub fn record_call(&mut self, args: &[vais_ir::Value], result: Option<&vais_ir::Value>) {
        self.call_count += 1;

        for (i, arg) in args.iter().enumerate() {
            if i < self.param_types.len() {
                let spec_type = SpecType::from_value(arg);
                *self.param_types[i].entry(spec_type).or_insert(0) += 1;
            }
        }

        if let Some(ret) = result {
            let spec_type = SpecType::from_value(ret);
            *self.return_types.entry(spec_type).or_insert(0) += 1;
        }
    }

    /// 지배적인 파라미터 타입 반환
    pub fn dominant_param_types(&self) -> Vec<SpecType> {
        self.param_types
            .iter()
            .map(|hist| {
                hist.iter()
                    .max_by_key(|(_, count)| *count)
                    .map(|(t, _)| *t)
                    .unwrap_or(SpecType::Unknown)
            })
            .collect()
    }

    /// 지배적인 반환 타입 반환
    pub fn dominant_return_type(&self) -> SpecType {
        self.return_types
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(t, _)| *t)
            .unwrap_or(SpecType::Unknown)
    }

    /// 특화 가능 여부 (타입이 안정적인지)
    pub fn is_stable(&self) -> bool {
        if self.call_count < 100 {
            return false; // 충분한 데이터 없음
        }

        // 각 파라미터에서 지배적 타입이 90% 이상이면 안정적
        for hist in &self.param_types {
            let total: u64 = hist.values().sum();
            if total == 0 {
                continue;
            }

            let max_count = hist.values().max().copied().unwrap_or(0);
            let ratio = max_count as f64 / total as f64;

            if ratio < 0.9 {
                return false;
            }
        }

        true
    }
}

impl SpeculativeContext {
    pub fn new() -> Self {
        Self {
            type_profiles: HashMap::new(),
            guards: HashMap::new(),
            next_guard_id: AtomicU64::new(1),
            deopt_count: AtomicU64::new(0),
        }
    }

    /// 타입 프로파일 가져오기 또는 생성
    pub fn get_or_create_profile(&mut self, name: &str, param_count: usize) -> &mut TypeProfile {
        self.type_profiles
            .entry(name.to_string())
            .or_insert_with(|| TypeProfile::new(name.to_string(), param_count))
    }

    /// 타입 프로파일 조회
    pub fn get_profile(&self, name: &str) -> Option<&TypeProfile> {
        self.type_profiles.get(name)
    }

    /// 새 타입 가드 생성
    pub fn create_guard(&mut self, expected_type: SpecType) -> TypeGuard {
        let id = self.next_guard_id.fetch_add(1, Ordering::Relaxed);
        let guard = TypeGuard::new(id, expected_type);
        self.guards.insert(id, guard.clone());
        guard
    }

    /// 가드 체크 및 deoptimization 처리
    pub fn check_guard(&self, guard_id: u64, value: &vais_ir::Value) -> GuardResult {
        if let Some(guard) = self.guards.get(&guard_id) {
            let result = guard.check(value);
            if result == GuardResult::Fail {
                self.deopt_count.fetch_add(1, Ordering::Relaxed);
            }
            result
        } else {
            GuardResult::Pass // 가드 없으면 통과
        }
    }

    /// Deoptimization 횟수 조회
    pub fn deopt_count(&self) -> u64 {
        self.deopt_count.load(Ordering::Relaxed)
    }

    /// 특화 추천 함수들 반환
    pub fn get_specialization_candidates(&self) -> Vec<&TypeProfile> {
        self.type_profiles
            .values()
            .filter(|p| p.is_stable() && !p.is_specialized)
            .collect()
    }

    /// 함수 특화 완료 표시
    pub fn mark_specialized(&mut self, name: &str) {
        if let Some(profile) = self.type_profiles.get_mut(name) {
            profile.is_specialized = true;
        }
    }

    /// 통계 출력
    pub fn print_stats(&self) {
        println!("\n=== Speculative Optimization Stats ===");
        println!("Total deoptimizations: {}", self.deopt_count());
        println!("Active guards: {}", self.guards.len());

        let valid_guards = self.guards.values().filter(|g| g.is_valid()).count();
        let disabled_guards = self.guards.len() - valid_guards;
        println!("Valid guards: {}, Disabled: {}", valid_guards, disabled_guards);

        println!("\nType Profiles:");
        for profile in self.type_profiles.values() {
            let stable_marker = if profile.is_stable() { " [stable]" } else { "" };
            let spec_marker = if profile.is_specialized { " [specialized]" } else { "" };
            println!(
                "  {} - {} calls{}{}",
                profile.name, profile.call_count, stable_marker, spec_marker
            );

            if !profile.param_types.is_empty() {
                let types: Vec<_> = profile.dominant_param_types()
                    .iter()
                    .map(|t| t.name())
                    .collect();
                println!("    params: ({})", types.join(", "));
            }
        }
    }
}

impl Default for SpeculativeContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Deoptimization 프레임
/// 인터프리터로 폴백할 때 필요한 상태
#[derive(Debug, Clone)]
pub struct DeoptFrame {
    /// 함수 이름
    pub func_name: String,
    /// 명령어 포인터 (폴백 위치)
    pub ip: usize,
    /// 로컬 변수들
    pub locals: HashMap<String, vais_ir::Value>,
    /// 스택 상태
    pub stack: Vec<vais_ir::Value>,
    /// Deopt 이유
    pub reason: DeoptReason,
}

/// Deoptimization 이유
#[derive(Debug, Clone)]
pub enum DeoptReason {
    /// 타입 가드 실패
    TypeGuardFailed { guard_id: u64, expected: SpecType, actual: SpecType },
    /// 오버플로우
    Overflow,
    /// 배열 범위 초과
    BoundsCheck { index: i64, length: usize },
    /// 알 수 없는 이유
    Unknown(String),
}

impl DeoptFrame {
    pub fn new(func_name: String, ip: usize, reason: DeoptReason) -> Self {
        Self {
            func_name,
            ip,
            locals: HashMap::new(),
            stack: Vec::new(),
            reason,
        }
    }

    pub fn with_state(
        mut self,
        locals: HashMap<String, vais_ir::Value>,
        stack: Vec<vais_ir::Value>,
    ) -> Self {
        self.locals = locals;
        self.stack = stack;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vais_ir::Value;

    #[test]
    fn test_spec_type_from_value() {
        assert_eq!(SpecType::from_value(&Value::Int(42)), SpecType::Int);
        assert_eq!(SpecType::from_value(&Value::Float(3.14)), SpecType::Float);
        assert_eq!(SpecType::from_value(&Value::Bool(true)), SpecType::Bool);

        let int_array = Value::Array(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
        assert_eq!(SpecType::from_value(&int_array), SpecType::ArrayInt);

        let float_array = Value::Array(vec![Value::Float(1.0), Value::Float(2.0)]);
        assert_eq!(SpecType::from_value(&float_array), SpecType::ArrayFloat);

        let mixed_array = Value::Array(vec![Value::Int(1), Value::Float(2.0)]);
        assert_eq!(SpecType::from_value(&mixed_array), SpecType::ArrayMixed);
    }

    #[test]
    fn test_type_guard() {
        let guard = TypeGuard::new(1, SpecType::Int);

        assert_eq!(guard.check(&Value::Int(42)), GuardResult::Pass);
        assert_eq!(guard.check(&Value::Float(3.14)), GuardResult::Fail);
        assert!(guard.is_valid());

        // 10번 실패하면 비활성화
        for _ in 0..10 {
            guard.check(&Value::Float(1.0));
        }
        assert!(!guard.is_valid());
    }

    #[test]
    fn test_type_profile() {
        let mut profile = TypeProfile::new("add".to_string(), 2);

        // 대부분 Int로 호출
        for _ in 0..95 {
            profile.record_call(&[Value::Int(1), Value::Int(2)], Some(&Value::Int(3)));
        }

        // 일부 Float로 호출
        for _ in 0..5 {
            profile.record_call(&[Value::Float(1.0), Value::Float(2.0)], Some(&Value::Float(3.0)));
        }

        assert_eq!(profile.call_count, 100);
        assert_eq!(profile.dominant_param_types(), vec![SpecType::Int, SpecType::Int]);
        assert!(profile.is_stable());
    }

    #[test]
    fn test_speculative_context() {
        let mut ctx = SpeculativeContext::new();

        let profile = ctx.get_or_create_profile("test_func", 2);
        for _ in 0..150 {
            profile.record_call(&[Value::Int(1), Value::Int(2)], Some(&Value::Int(3)));
        }

        let candidates = ctx.get_specialization_candidates();
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].name, "test_func");

        ctx.mark_specialized("test_func");
        let candidates_after = ctx.get_specialization_candidates();
        assert!(candidates_after.is_empty());
    }
}
