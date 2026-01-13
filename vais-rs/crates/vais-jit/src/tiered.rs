//! Tiered JIT (다단계 JIT 컴파일)
//!
//! - Tier 0: 인터프리터 (즉시 실행, 프로파일링)
//! - Tier 1: 기본 JIT (빠른 컴파일, 기본 최적화)
//! - Tier 2: 최적화 JIT (느린 컴파일, 고급 최적화)

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

/// 컴파일 티어
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CompilationTier {
    /// 인터프리터 (프로파일링 수집)
    Interpreter = 0,
    /// 기본 JIT (빠른 컴파일)
    Baseline = 1,
    /// 최적화 JIT (고급 최적화)
    Optimized = 2,
}

impl CompilationTier {
    pub fn name(&self) -> &'static str {
        match self {
            CompilationTier::Interpreter => "Interpreter",
            CompilationTier::Baseline => "Baseline JIT",
            CompilationTier::Optimized => "Optimized JIT",
        }
    }

    /// 다음 티어
    pub fn next(&self) -> Option<CompilationTier> {
        match self {
            CompilationTier::Interpreter => Some(CompilationTier::Baseline),
            CompilationTier::Baseline => Some(CompilationTier::Optimized),
            CompilationTier::Optimized => None,
        }
    }
}

/// 티어 승격 임계값
#[derive(Debug, Clone)]
pub struct TierThresholds {
    /// Tier 0 -> Tier 1 (호출 횟수)
    pub baseline_threshold: u64,
    /// Tier 1 -> Tier 2 (호출 횟수)
    pub optimized_threshold: u64,
    /// Tier 1 -> Tier 2 (실행 시간 기준, 밀리초)
    pub optimized_time_threshold: Duration,
}

impl Default for TierThresholds {
    fn default() -> Self {
        Self {
            baseline_threshold: 50,       // 50회 호출 후 Baseline JIT
            optimized_threshold: 1000,    // 1000회 호출 후 Optimized JIT
            optimized_time_threshold: Duration::from_millis(100), // 100ms 이상 소요 시
        }
    }
}

/// 함수별 티어 정보
#[derive(Debug)]
pub struct FunctionTierInfo {
    /// 함수 이름
    pub name: String,
    /// 현재 티어
    pub current_tier: CompilationTier,
    /// 호출 횟수
    pub call_count: AtomicU64,
    /// 총 실행 시간
    pub total_time: Duration,
    /// 컴파일된 코드 포인터 (티어별)
    pub compiled_code: HashMap<CompilationTier, *const u8>,
    /// 마지막 티어 업그레이드 시간
    pub last_upgrade: Option<Instant>,
    /// 티어업 보류 중 여부
    pub upgrade_pending: bool,
}

impl FunctionTierInfo {
    pub fn new(name: String) -> Self {
        Self {
            name,
            current_tier: CompilationTier::Interpreter,
            call_count: AtomicU64::new(0),
            total_time: Duration::ZERO,
            compiled_code: HashMap::new(),
            last_upgrade: None,
            upgrade_pending: false,
        }
    }

    /// 호출 기록
    pub fn record_call(&self) {
        self.call_count.fetch_add(1, Ordering::Relaxed);
    }

    /// 실행 시간 추가
    pub fn add_time(&mut self, duration: Duration) {
        self.total_time += duration;
    }

    /// 평균 실행 시간
    pub fn avg_time(&self) -> Duration {
        let count = self.call_count.load(Ordering::Relaxed);
        if count > 0 {
            self.total_time / count as u32
        } else {
            Duration::ZERO
        }
    }

    /// 티어 업그레이드 필요 여부
    pub fn should_upgrade(&self, thresholds: &TierThresholds) -> bool {
        if self.upgrade_pending {
            return false;
        }

        let call_count = self.call_count.load(Ordering::Relaxed);

        match self.current_tier {
            CompilationTier::Interpreter => {
                call_count >= thresholds.baseline_threshold
            }
            CompilationTier::Baseline => {
                call_count >= thresholds.optimized_threshold
                    || self.total_time >= thresholds.optimized_time_threshold
            }
            CompilationTier::Optimized => false,
        }
    }

    /// 티어 업그레이드
    pub fn upgrade(&mut self, new_tier: CompilationTier, code_ptr: *const u8) {
        self.current_tier = new_tier;
        self.compiled_code.insert(new_tier, code_ptr);
        self.last_upgrade = Some(Instant::now());
        self.upgrade_pending = false;
    }

    /// 현재 코드 포인터
    pub fn current_code(&self) -> Option<*const u8> {
        self.compiled_code.get(&self.current_tier).copied()
    }
}

// 포인터를 안전하게 공유
unsafe impl Send for FunctionTierInfo {}
unsafe impl Sync for FunctionTierInfo {}

/// Tiered JIT 매니저
pub struct TieredManager {
    /// 함수별 티어 정보
    functions: HashMap<String, FunctionTierInfo>,
    /// 티어 임계값
    thresholds: TierThresholds,
    /// 컴파일 큐 (백그라운드 컴파일용)
    compile_queue: Vec<CompileRequest>,
    /// 통계
    stats: TieredStats,
}

/// 컴파일 요청
#[derive(Debug, Clone)]
pub struct CompileRequest {
    pub func_name: String,
    pub target_tier: CompilationTier,
    pub priority: u32,
    pub requested_at: Instant,
}

/// 티어드 JIT 통계
#[derive(Debug, Default)]
pub struct TieredStats {
    /// 티어별 함수 수
    pub functions_per_tier: HashMap<CompilationTier, usize>,
    /// 티어별 총 컴파일 시간
    pub compile_time_per_tier: HashMap<CompilationTier, Duration>,
    /// 총 티어 업그레이드 횟수
    pub total_upgrades: u64,
    /// 백그라운드 컴파일 횟수
    pub background_compiles: u64,
}

impl TieredManager {
    pub fn new() -> Self {
        Self::with_thresholds(TierThresholds::default())
    }

    pub fn with_thresholds(thresholds: TierThresholds) -> Self {
        Self {
            functions: HashMap::new(),
            thresholds,
            compile_queue: Vec::new(),
            stats: TieredStats::default(),
        }
    }

    /// 함수 정보 가져오기 또는 생성
    pub fn get_or_create(&mut self, name: &str) -> &mut FunctionTierInfo {
        if !self.functions.contains_key(name) {
            self.functions.insert(
                name.to_string(),
                FunctionTierInfo::new(name.to_string()),
            );
        }
        self.functions.get_mut(name).unwrap()
    }

    /// 함수 정보 조회
    pub fn get(&self, name: &str) -> Option<&FunctionTierInfo> {
        self.functions.get(name)
    }

    /// 호출 기록 및 티어업 체크
    pub fn record_call(&mut self, name: &str, duration: Duration) -> Option<CompileRequest> {
        // 먼저 함수 정보를 가져오거나 생성
        if !self.functions.contains_key(name) {
            self.functions.insert(
                name.to_string(),
                FunctionTierInfo::new(name.to_string()),
            );
        }

        let info = self.functions.get_mut(name).unwrap();
        info.record_call();
        info.add_time(duration);

        // 티어업 필요 여부 확인
        let should_upgrade = info.should_upgrade(&self.thresholds);
        let next_tier = info.current_tier.next();
        let call_count = info.call_count.load(Ordering::Relaxed);
        let total_time = info.total_time;

        if should_upgrade {
            if let Some(target_tier) = next_tier {
                info.upgrade_pending = true;

                // 우선순위 계산 (호출 횟수 + 시간 가중치)
                let time_weight = total_time.as_micros() as u64;
                let priority = (call_count + time_weight / 1000) as u32;

                let request = CompileRequest {
                    func_name: name.to_string(),
                    target_tier,
                    priority,
                    requested_at: Instant::now(),
                };

                self.compile_queue.push(request.clone());
                return Some(request);
            }
        }

        None
    }

    /// 컴파일 우선순위 계산 (테스트 및 향후 사용을 위해 유지)
    #[allow(dead_code)]
    fn calculate_priority(&self, info: &FunctionTierInfo) -> u32 {
        let call_count = info.call_count.load(Ordering::Relaxed);
        let time_weight = info.total_time.as_micros() as u64;

        // 호출 횟수 + 총 실행 시간 기반 우선순위
        (call_count + time_weight / 1000) as u32
    }

    /// 다음 컴파일 요청 가져오기 (우선순위 기반)
    pub fn pop_compile_request(&mut self) -> Option<CompileRequest> {
        if self.compile_queue.is_empty() {
            return None;
        }

        // 우선순위가 가장 높은 것 선택
        let max_idx = self.compile_queue
            .iter()
            .enumerate()
            .max_by_key(|(_, r)| r.priority)
            .map(|(i, _)| i)?;

        Some(self.compile_queue.remove(max_idx))
    }

    /// 컴파일 완료 처리
    pub fn complete_compilation(
        &mut self,
        name: &str,
        tier: CompilationTier,
        code_ptr: *const u8,
        compile_time: Duration,
    ) {
        if let Some(info) = self.functions.get_mut(name) {
            info.upgrade(tier, code_ptr);
        }

        // 통계 업데이트
        *self.stats.functions_per_tier.entry(tier).or_insert(0) += 1;
        *self.stats.compile_time_per_tier.entry(tier).or_insert(Duration::ZERO) += compile_time;
        self.stats.total_upgrades += 1;
    }

    /// 함수의 현재 티어 조회
    pub fn current_tier(&self, name: &str) -> CompilationTier {
        self.functions
            .get(name)
            .map(|info| info.current_tier)
            .unwrap_or(CompilationTier::Interpreter)
    }

    /// 함수의 최적 코드 포인터 조회
    pub fn get_code(&self, name: &str) -> Option<*const u8> {
        self.functions.get(name).and_then(|info| info.current_code())
    }

    /// 업그레이드 후보 함수들
    pub fn get_upgrade_candidates(&self) -> Vec<&FunctionTierInfo> {
        self.functions
            .values()
            .filter(|info| info.should_upgrade(&self.thresholds))
            .collect()
    }

    /// 통계 출력
    pub fn print_stats(&self) {
        println!("\n=== Tiered JIT Statistics ===");

        // 티어별 함수 수
        println!("\nFunctions per tier:");
        for tier in [CompilationTier::Interpreter, CompilationTier::Baseline, CompilationTier::Optimized] {
            let count = self.functions
                .values()
                .filter(|f| f.current_tier == tier)
                .count();
            println!("  {}: {}", tier.name(), count);
        }

        // 컴파일 시간
        println!("\nCompile time per tier:");
        for (tier, time) in &self.stats.compile_time_per_tier {
            println!("  {}: {:?}", tier.name(), time);
        }

        println!("\nTotal upgrades: {}", self.stats.total_upgrades);
        println!("Pending compiles: {}", self.compile_queue.len());

        // 핫 함수 Top 10
        println!("\nHottest functions:");
        let mut funcs: Vec<_> = self.functions.values().collect();
        funcs.sort_by(|a, b| {
            b.call_count.load(Ordering::Relaxed)
                .cmp(&a.call_count.load(Ordering::Relaxed))
        });

        for info in funcs.iter().take(10) {
            let call_count = info.call_count.load(Ordering::Relaxed);
            println!(
                "  {} - {} calls, {:?} total, {} tier",
                info.name,
                call_count,
                info.total_time,
                info.current_tier.name()
            );
        }
    }

    /// 임계값 설정
    pub fn set_thresholds(&mut self, thresholds: TierThresholds) {
        self.thresholds = thresholds;
    }

    /// 현재 임계값 조회
    pub fn thresholds(&self) -> &TierThresholds {
        &self.thresholds
    }
}

impl Default for TieredManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 백그라운드 컴파일러 인터페이스
pub trait BackgroundCompiler {
    /// 컴파일 요청 제출
    fn submit(&mut self, request: CompileRequest);

    /// 완료된 컴파일 결과 폴링
    fn poll_completed(&mut self) -> Option<CompileResult>;

    /// 대기 중인 요청 수
    fn pending_count(&self) -> usize;
}

/// 컴파일 결과
#[derive(Debug)]
pub struct CompileResult {
    pub func_name: String,
    pub tier: CompilationTier,
    pub code_ptr: *const u8,
    pub compile_time: Duration,
    pub success: bool,
    pub error: Option<String>,
}

// 포인터를 안전하게 공유
unsafe impl Send for CompileResult {}
unsafe impl Sync for CompileResult {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tier_progression() {
        let mut manager = TieredManager::with_thresholds(TierThresholds {
            baseline_threshold: 5,
            optimized_threshold: 20,
            optimized_time_threshold: Duration::from_secs(10),
        });

        // 초기 상태
        assert_eq!(manager.current_tier("test_func"), CompilationTier::Interpreter);

        // 5번 호출 -> Baseline 요청
        for _ in 0..5 {
            manager.record_call("test_func", Duration::from_micros(100));
        }

        let request = manager.pop_compile_request();
        assert!(request.is_some());
        assert_eq!(request.as_ref().unwrap().target_tier, CompilationTier::Baseline);

        // 컴파일 완료
        let fake_ptr = 0x1000 as *const u8;
        manager.complete_compilation(
            "test_func",
            CompilationTier::Baseline,
            fake_ptr,
            Duration::from_millis(10),
        );

        assert_eq!(manager.current_tier("test_func"), CompilationTier::Baseline);
        assert_eq!(manager.get_code("test_func"), Some(fake_ptr));
    }

    #[test]
    fn test_priority_calculation() {
        let mut manager = TieredManager::new();

        // 함수 A: 많이 호출, 적당한 시간
        for _ in 0..1000 {
            manager.record_call("func_a", Duration::from_micros(100));
        }

        // 함수 B: 적게 호출, 짧은 시간
        for _ in 0..10 {
            manager.record_call("func_b", Duration::from_micros(10));
        }

        // 우선순위 확인 (호출 횟수 + 시간 가중치)
        let info_a = manager.get("func_a").unwrap();
        let info_b = manager.get("func_b").unwrap();

        let priority_a = manager.calculate_priority(info_a);
        let priority_b = manager.calculate_priority(info_b);

        // func_a가 더 많이 호출되었고 총 시간도 길어서 우선순위 높음
        // func_a: 1000 calls + 100ms total = 1000 + 100 = 1100
        // func_b: 10 calls + 0.1ms total = 10 + 0 = 10
        assert!(priority_a > priority_b);
    }

    #[test]
    fn test_upgrade_candidates() {
        let mut manager = TieredManager::with_thresholds(TierThresholds {
            baseline_threshold: 10,
            optimized_threshold: 100,
            optimized_time_threshold: Duration::from_secs(10),
        });

        // 임계값 미달
        for _ in 0..5 {
            manager.record_call("cold_func", Duration::from_micros(10));
        }

        // 임계값 도달
        for _ in 0..15 {
            manager.record_call("hot_func", Duration::from_micros(10));
        }

        // hot_func만 업그레이드 대상 (하지만 이미 pending 상태일 수 있음)
        // 실제로는 record_call에서 이미 요청이 생성됨
        assert!(manager.compile_queue.len() >= 1);
    }

    #[test]
    fn test_thresholds() {
        let thresholds = TierThresholds {
            baseline_threshold: 100,
            optimized_threshold: 5000,
            optimized_time_threshold: Duration::from_secs(5),
        };

        let manager = TieredManager::with_thresholds(thresholds.clone());

        assert_eq!(manager.thresholds().baseline_threshold, 100);
        assert_eq!(manager.thresholds().optimized_threshold, 5000);
    }
}
