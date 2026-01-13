//! On-Stack Replacement (OSR)
//!
//! 실행 중인 함수를 더 최적화된 버전으로 교체합니다.
//! 주로 핫 루프에서 인터프리터 → JIT 코드로 전환할 때 사용됩니다.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, AtomicBool, Ordering};
use std::sync::Arc;

/// OSR 진입점
/// 루프 헤더에 삽입되어 OSR 전환 여부를 결정합니다.
#[derive(Debug, Clone)]
pub struct OsrPoint {
    /// OSR 포인트 ID
    pub id: u64,
    /// 함수 이름
    pub func_name: String,
    /// 바이트코드 오프셋 (루프 헤더 위치)
    pub bytecode_offset: usize,
    /// 루프 실행 횟수
    pub iteration_count: Arc<AtomicU64>,
    /// OSR 전환 임계값
    pub threshold: u64,
    /// 이미 전환됨
    pub transitioned: Arc<AtomicBool>,
    /// 컴파일된 OSR 진입 함수 포인터
    pub compiled_entry: Option<*const u8>,
}

impl OsrPoint {
    pub fn new(id: u64, func_name: String, bytecode_offset: usize, threshold: u64) -> Self {
        Self {
            id,
            func_name,
            bytecode_offset,
            iteration_count: Arc::new(AtomicU64::new(0)),
            threshold,
            transitioned: Arc::new(AtomicBool::new(false)),
            compiled_entry: None,
        }
    }

    /// 루프 반복 기록 및 OSR 필요 여부 반환
    pub fn record_iteration(&self) -> OsrDecision {
        if self.transitioned.load(Ordering::Relaxed) {
            // 이미 전환됨 - 컴파일된 코드로 점프
            if self.compiled_entry.is_some() {
                return OsrDecision::Jump;
            }
            return OsrDecision::Continue;
        }

        let count = self.iteration_count.fetch_add(1, Ordering::Relaxed) + 1;

        if count >= self.threshold {
            OsrDecision::Compile
        } else {
            OsrDecision::Continue
        }
    }

    /// OSR 전환 완료 표시
    pub fn mark_transitioned(&self, _entry: *const u8) {
        self.transitioned.store(true, Ordering::Relaxed);
    }

    /// 컴파일된 진입점 설정
    pub fn set_compiled_entry(&mut self, entry: *const u8) {
        self.compiled_entry = Some(entry);
        self.transitioned.store(true, Ordering::Relaxed);
    }

    /// 통계 문자열
    pub fn stats_string(&self) -> String {
        let count = self.iteration_count.load(Ordering::Relaxed);
        let transitioned = self.transitioned.load(Ordering::Relaxed);
        format!(
            "OSR[{}] {}@{} - iterations: {}, threshold: {}, transitioned: {}",
            self.id, self.func_name, self.bytecode_offset, count, self.threshold, transitioned
        )
    }
}

// 포인터 안전하게 공유
unsafe impl Send for OsrPoint {}
unsafe impl Sync for OsrPoint {}

/// OSR 결정
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OsrDecision {
    /// 계속 인터프리터에서 실행
    Continue,
    /// OSR 컴파일 시작
    Compile,
    /// 컴파일된 코드로 점프
    Jump,
}

/// OSR 상태 프레임
/// 인터프리터에서 JIT 코드로 전환할 때 필요한 상태
#[derive(Debug, Clone)]
pub struct OsrFrame {
    /// 함수 이름
    pub func_name: String,
    /// OSR 포인트 ID
    pub osr_point_id: u64,
    /// 로컬 변수 상태
    pub locals: HashMap<String, OsrValue>,
    /// 스택 상태
    pub stack: Vec<OsrValue>,
    /// 현재 루프 인덱스 (있는 경우)
    pub loop_index: Option<i64>,
    /// 루프 상한 (있는 경우)
    pub loop_limit: Option<i64>,
}

/// OSR용 값 표현
#[derive(Debug, Clone)]
pub enum OsrValue {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Array(Vec<OsrValue>),
    Null,
}

impl OsrFrame {
    pub fn new(func_name: String, osr_point_id: u64) -> Self {
        Self {
            func_name,
            osr_point_id,
            locals: HashMap::new(),
            stack: Vec::new(),
            loop_index: None,
            loop_limit: None,
        }
    }

    /// 로컬 변수 설정
    pub fn set_local(&mut self, name: String, value: OsrValue) {
        self.locals.insert(name, value);
    }

    /// 스택 값 추가
    pub fn push_stack(&mut self, value: OsrValue) {
        self.stack.push(value);
    }

    /// 루프 상태 설정
    pub fn set_loop_state(&mut self, index: i64, limit: i64) {
        self.loop_index = Some(index);
        self.loop_limit = Some(limit);
    }
}

/// OSR 버퍼
/// JIT 코드에서 인터프리터 상태를 읽기 위한 버퍼
#[repr(C)]
pub struct OsrBuffer {
    /// 로컬 변수 슬롯들
    pub locals: [i64; 64],
    /// 로컬 변수 타입 (0=int, 1=float, 2=bool, 3=string ptr)
    pub local_types: [u8; 64],
    /// 사용된 로컬 변수 수
    pub local_count: usize,
    /// 스택 슬롯들
    pub stack: [i64; 32],
    /// 스택 타입
    pub stack_types: [u8; 32],
    /// 스택 깊이
    pub stack_depth: usize,
    /// 루프 인덱스
    pub loop_index: i64,
    /// 루프 상한
    pub loop_limit: i64,
}

impl OsrBuffer {
    pub fn new() -> Self {
        Self {
            locals: [0; 64],
            local_types: [0; 64],
            local_count: 0,
            stack: [0; 32],
            stack_types: [0; 32],
            stack_depth: 0,
            loop_index: 0,
            loop_limit: 0,
        }
    }

    /// OsrFrame에서 버퍼 생성
    pub fn from_frame(frame: &OsrFrame, var_names: &[String]) -> Self {
        let mut buffer = Self::new();

        // 로컬 변수 복사
        for (i, name) in var_names.iter().enumerate() {
            if i >= 64 {
                break;
            }
            if let Some(value) = frame.locals.get(name) {
                match value {
                    OsrValue::Int(v) => {
                        buffer.locals[i] = *v;
                        buffer.local_types[i] = 0;
                    }
                    OsrValue::Float(v) => {
                        buffer.locals[i] = v.to_bits() as i64;
                        buffer.local_types[i] = 1;
                    }
                    OsrValue::Bool(v) => {
                        buffer.locals[i] = if *v { 1 } else { 0 };
                        buffer.local_types[i] = 2;
                    }
                    _ => {}
                }
                buffer.local_count = i + 1;
            }
        }

        // 스택 복사
        for (i, value) in frame.stack.iter().enumerate() {
            if i >= 32 {
                break;
            }
            match value {
                OsrValue::Int(v) => {
                    buffer.stack[i] = *v;
                    buffer.stack_types[i] = 0;
                }
                OsrValue::Float(v) => {
                    buffer.stack[i] = v.to_bits() as i64;
                    buffer.stack_types[i] = 1;
                }
                OsrValue::Bool(v) => {
                    buffer.stack[i] = if *v { 1 } else { 0 };
                    buffer.stack_types[i] = 2;
                }
                _ => {}
            }
            buffer.stack_depth = i + 1;
        }

        // 루프 상태
        buffer.loop_index = frame.loop_index.unwrap_or(0);
        buffer.loop_limit = frame.loop_limit.unwrap_or(0);

        buffer
    }
}

impl Default for OsrBuffer {
    fn default() -> Self {
        Self::new()
    }
}

/// OSR 매니저
pub struct OsrManager {
    /// OSR 포인트들
    points: HashMap<u64, OsrPoint>,
    /// 함수별 OSR 포인트 매핑
    func_points: HashMap<String, Vec<u64>>,
    /// 다음 포인트 ID
    next_id: AtomicU64,
    /// OSR 임계값
    threshold: u64,
    /// 총 OSR 전환 횟수
    total_transitions: AtomicU64,
}

impl OsrManager {
    pub fn new() -> Self {
        Self::with_threshold(1000) // 기본 1000번 반복 후 OSR
    }

    pub fn with_threshold(threshold: u64) -> Self {
        Self {
            points: HashMap::new(),
            func_points: HashMap::new(),
            next_id: AtomicU64::new(1),
            threshold,
            total_transitions: AtomicU64::new(0),
        }
    }

    /// OSR 포인트 생성
    pub fn create_point(&mut self, func_name: &str, bytecode_offset: usize) -> u64 {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let point = OsrPoint::new(id, func_name.to_string(), bytecode_offset, self.threshold);

        self.points.insert(id, point);
        self.func_points
            .entry(func_name.to_string())
            .or_default()
            .push(id);

        id
    }

    /// OSR 포인트 조회
    pub fn get_point(&self, id: u64) -> Option<&OsrPoint> {
        self.points.get(&id)
    }

    /// OSR 포인트 조회 (mutable)
    pub fn get_point_mut(&mut self, id: u64) -> Option<&mut OsrPoint> {
        self.points.get_mut(&id)
    }

    /// 함수의 모든 OSR 포인트 조회
    pub fn get_func_points(&self, func_name: &str) -> Vec<&OsrPoint> {
        self.func_points
            .get(func_name)
            .map(|ids| ids.iter().filter_map(|id| self.points.get(id)).collect())
            .unwrap_or_default()
    }

    /// 루프 반복 기록
    pub fn record_iteration(&self, point_id: u64) -> OsrDecision {
        if let Some(point) = self.points.get(&point_id) {
            point.record_iteration()
        } else {
            OsrDecision::Continue
        }
    }

    /// OSR 전환 완료
    pub fn complete_transition(&mut self, point_id: u64, entry: *const u8) {
        if let Some(point) = self.points.get_mut(&point_id) {
            point.set_compiled_entry(entry);
            self.total_transitions.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// 임계값 설정
    pub fn set_threshold(&mut self, threshold: u64) {
        self.threshold = threshold;
    }

    /// 통계 출력
    pub fn print_stats(&self) {
        println!("\n=== OSR Statistics ===");
        println!("Total OSR points: {}", self.points.len());
        println!(
            "Total transitions: {}",
            self.total_transitions.load(Ordering::Relaxed)
        );

        // 함수별 통계
        println!("\nPer-function OSR points:");
        for (func_name, point_ids) in &self.func_points {
            println!("  {}: {} points", func_name, point_ids.len());
            for id in point_ids {
                if let Some(point) = self.points.get(id) {
                    println!("    {}", point.stats_string());
                }
            }
        }

        // 핫 루프 (가장 많이 반복된)
        let mut hot_points: Vec<_> = self.points.values().collect();
        hot_points.sort_by(|a, b| {
            b.iteration_count
                .load(Ordering::Relaxed)
                .cmp(&a.iteration_count.load(Ordering::Relaxed))
        });

        println!("\nTop 10 hottest loops:");
        for point in hot_points.iter().take(10) {
            println!("  {}", point.stats_string());
        }
    }

    /// 모든 포인트 초기화
    pub fn clear(&mut self) {
        self.points.clear();
        self.func_points.clear();
    }
}

impl Default for OsrManager {
    fn default() -> Self {
        Self::new()
    }
}

/// OSR 컴파일 요청
#[derive(Debug, Clone)]
pub struct OsrCompileRequest {
    /// OSR 포인트 ID
    pub point_id: u64,
    /// 함수 이름
    pub func_name: String,
    /// 바이트코드 오프셋
    pub bytecode_offset: usize,
    /// 예상 변수 타입들
    pub expected_types: HashMap<String, OsrValueType>,
}

/// OSR 값 타입
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OsrValueType {
    Int,
    Float,
    Bool,
    String,
    Array,
    Unknown,
}

impl OsrCompileRequest {
    pub fn new(point_id: u64, func_name: String, bytecode_offset: usize) -> Self {
        Self {
            point_id,
            func_name,
            bytecode_offset,
            expected_types: HashMap::new(),
        }
    }

    /// 예상 타입 추가
    pub fn add_type(&mut self, var_name: String, typ: OsrValueType) {
        self.expected_types.insert(var_name, typ);
    }
}

/// OSR 진입 스텁 생성기
/// 인터프리터에서 JIT 코드로 안전하게 전환하기 위한 스텁 코드 생성
pub struct OsrStubGenerator {
    /// 생성된 스텁들
    stubs: HashMap<u64, OsrStub>,
}

/// OSR 스텁
#[derive(Debug)]
pub struct OsrStub {
    /// 스텁 코드 포인터
    pub code: *const u8,
    /// 코드 크기
    pub size: usize,
    /// 변수 오프셋 맵 (변수 이름 -> 스택 오프셋)
    pub var_offsets: HashMap<String, i32>,
}

unsafe impl Send for OsrStub {}
unsafe impl Sync for OsrStub {}

impl OsrStubGenerator {
    pub fn new() -> Self {
        Self {
            stubs: HashMap::new(),
        }
    }

    /// 스텁 조회
    pub fn get_stub(&self, point_id: u64) -> Option<&OsrStub> {
        self.stubs.get(&point_id)
    }

    /// 스텁 등록
    pub fn register_stub(&mut self, point_id: u64, stub: OsrStub) {
        self.stubs.insert(point_id, stub);
    }
}

impl Default for OsrStubGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_osr_point_basic() {
        let point = OsrPoint::new(1, "test_func".to_string(), 10, 100);

        assert_eq!(point.id, 1);
        assert_eq!(point.func_name, "test_func");
        assert_eq!(point.bytecode_offset, 10);
        assert_eq!(point.threshold, 100);

        // 임계값 이전
        for _ in 0..99 {
            assert_eq!(point.record_iteration(), OsrDecision::Continue);
        }

        // 임계값 도달
        assert_eq!(point.record_iteration(), OsrDecision::Compile);
    }

    #[test]
    fn test_osr_manager() {
        let mut manager = OsrManager::with_threshold(10);

        // OSR 포인트 생성
        let point_id = manager.create_point("loop_func", 5);
        assert!(manager.get_point(point_id).is_some());

        // 반복 기록
        for _ in 0..9 {
            assert_eq!(manager.record_iteration(point_id), OsrDecision::Continue);
        }
        assert_eq!(manager.record_iteration(point_id), OsrDecision::Compile);

        // 전환 완료
        let fake_entry = 0x1000 as *const u8;
        manager.complete_transition(point_id, fake_entry);

        let point = manager.get_point(point_id).unwrap();
        assert!(point.transitioned.load(Ordering::Relaxed));
        assert_eq!(point.compiled_entry, Some(fake_entry));
    }

    #[test]
    fn test_osr_frame() {
        let mut frame = OsrFrame::new("test".to_string(), 1);

        frame.set_local("i".to_string(), OsrValue::Int(42));
        frame.set_local("sum".to_string(), OsrValue::Int(100));
        frame.push_stack(OsrValue::Int(5));
        frame.set_loop_state(10, 100);

        assert_eq!(frame.locals.len(), 2);
        assert_eq!(frame.stack.len(), 1);
        assert_eq!(frame.loop_index, Some(10));
        assert_eq!(frame.loop_limit, Some(100));
    }

    #[test]
    fn test_osr_buffer_from_frame() {
        let mut frame = OsrFrame::new("test".to_string(), 1);
        frame.set_local("a".to_string(), OsrValue::Int(10));
        frame.set_local("b".to_string(), OsrValue::Float(3.14));
        frame.push_stack(OsrValue::Bool(true));
        frame.set_loop_state(5, 50);

        let var_names = vec!["a".to_string(), "b".to_string()];
        let buffer = OsrBuffer::from_frame(&frame, &var_names);

        assert_eq!(buffer.locals[0], 10);
        assert_eq!(buffer.local_types[0], 0); // Int
        assert_eq!(f64::from_bits(buffer.locals[1] as u64), 3.14);
        assert_eq!(buffer.local_types[1], 1); // Float
        assert_eq!(buffer.local_count, 2);

        assert_eq!(buffer.stack[0], 1); // true
        assert_eq!(buffer.stack_types[0], 2); // Bool
        assert_eq!(buffer.stack_depth, 1);

        assert_eq!(buffer.loop_index, 5);
        assert_eq!(buffer.loop_limit, 50);
    }

    #[test]
    fn test_osr_func_points() {
        let mut manager = OsrManager::with_threshold(100);

        // 같은 함수에 여러 OSR 포인트
        manager.create_point("func_a", 10);
        manager.create_point("func_a", 20);
        manager.create_point("func_a", 30);
        manager.create_point("func_b", 5);

        let func_a_points = manager.get_func_points("func_a");
        assert_eq!(func_a_points.len(), 3);

        let func_b_points = manager.get_func_points("func_b");
        assert_eq!(func_b_points.len(), 1);

        let func_c_points = manager.get_func_points("func_c");
        assert!(func_c_points.is_empty());
    }

    #[test]
    fn test_osr_after_transition() {
        let mut point = OsrPoint::new(1, "test".to_string(), 0, 5);

        // 임계값 도달
        for _ in 0..5 {
            point.record_iteration();
        }

        // 전환 완료
        let entry = 0x2000 as *const u8;
        point.set_compiled_entry(entry);

        // 전환 후에는 Jump 반환
        assert_eq!(point.record_iteration(), OsrDecision::Jump);
    }

    #[test]
    fn test_osr_compile_request() {
        let mut request = OsrCompileRequest::new(1, "hot_loop".to_string(), 15);

        request.add_type("i".to_string(), OsrValueType::Int);
        request.add_type("sum".to_string(), OsrValueType::Float);

        assert_eq!(request.expected_types.len(), 2);
        assert_eq!(
            request.expected_types.get("i"),
            Some(&OsrValueType::Int)
        );
        assert_eq!(
            request.expected_types.get("sum"),
            Some(&OsrValueType::Float)
        );
    }
}
