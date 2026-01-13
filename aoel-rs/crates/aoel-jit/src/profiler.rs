//! 핫 경로 프로파일러
//!
//! 함수 실행 횟수를 추적하여 JIT 컴파일 대상을 결정.

use std::collections::HashMap;
use std::time::{Duration, Instant};

/// JIT 컴파일 임계값 (이 횟수 이상 호출되면 JIT 컴파일)
pub const JIT_THRESHOLD: u64 = 100;

/// 함수별 실행 프로파일
#[derive(Debug, Clone)]
pub struct FunctionProfile {
    /// 함수 이름
    pub name: String,
    /// 호출 횟수
    pub call_count: u64,
    /// 총 실행 시간
    pub total_time: Duration,
    /// JIT 컴파일 여부
    pub is_jitted: bool,
    /// 타입 히스토그램 (인자 타입 패턴)
    pub type_histogram: HashMap<String, u64>,
}

impl FunctionProfile {
    pub fn new(name: String) -> Self {
        Self {
            name,
            call_count: 0,
            total_time: Duration::ZERO,
            is_jitted: false,
            type_histogram: HashMap::new(),
        }
    }

    /// 평균 실행 시간
    pub fn avg_time(&self) -> Duration {
        if self.call_count > 0 {
            self.total_time / self.call_count as u32
        } else {
            Duration::ZERO
        }
    }

    /// JIT 컴파일이 필요한지 확인
    pub fn should_jit(&self) -> bool {
        !self.is_jitted && self.call_count >= JIT_THRESHOLD
    }

    /// 가장 빈번한 타입 패턴 반환
    pub fn dominant_type_pattern(&self) -> Option<&str> {
        self.type_histogram
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(pattern, _)| pattern.as_str())
    }
}

/// 실행 프로파일러
#[derive(Debug)]
pub struct ExecutionProfiler {
    /// 함수별 프로파일
    profiles: HashMap<String, FunctionProfile>,
    /// 현재 실행 중인 함수 스택 (시작 시간 포함)
    execution_stack: Vec<(String, Instant)>,
    /// 프로파일링 활성화 여부
    enabled: bool,
}

impl ExecutionProfiler {
    pub fn new() -> Self {
        Self {
            profiles: HashMap::new(),
            execution_stack: Vec::new(),
            enabled: true,
        }
    }

    /// 프로파일링 활성화/비활성화
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// 함수 호출 시작 기록
    pub fn begin_call(&mut self, func_name: &str, arg_types: &str) {
        if !self.enabled {
            return;
        }

        let profile = self.profiles
            .entry(func_name.to_string())
            .or_insert_with(|| FunctionProfile::new(func_name.to_string()));

        profile.call_count += 1;
        *profile.type_histogram.entry(arg_types.to_string()).or_insert(0) += 1;

        self.execution_stack.push((func_name.to_string(), Instant::now()));
    }

    /// 함수 호출 종료 기록
    pub fn end_call(&mut self, func_name: &str) {
        if !self.enabled {
            return;
        }

        if let Some(pos) = self.execution_stack.iter().rposition(|(name, _)| name == func_name) {
            let (_, start_time) = self.execution_stack.remove(pos);
            let elapsed = start_time.elapsed();

            if let Some(profile) = self.profiles.get_mut(func_name) {
                profile.total_time += elapsed;
            }
        }
    }

    /// JIT 컴파일 대상 함수들 반환 (핫 함수)
    pub fn get_hot_functions(&self) -> Vec<&FunctionProfile> {
        self.profiles
            .values()
            .filter(|p| p.should_jit())
            .collect()
    }

    /// 모든 프로파일 반환
    pub fn get_all_profiles(&self) -> Vec<&FunctionProfile> {
        self.profiles.values().collect()
    }

    /// 특정 함수 프로파일 반환
    pub fn get_profile(&self, func_name: &str) -> Option<&FunctionProfile> {
        self.profiles.get(func_name)
    }

    /// 특정 함수 프로파일 가변 참조 반환
    pub fn get_profile_mut(&mut self, func_name: &str) -> Option<&mut FunctionProfile> {
        self.profiles.get_mut(func_name)
    }

    /// JIT 컴파일 완료 표시
    pub fn mark_jitted(&mut self, func_name: &str) {
        if let Some(profile) = self.profiles.get_mut(func_name) {
            profile.is_jitted = true;
        }
    }

    /// 프로파일 통계 출력
    pub fn print_stats(&self) {
        println!("\n=== Execution Profile ===");
        println!("{:<20} {:>10} {:>12} {:>10}", "Function", "Calls", "Total Time", "Avg Time");
        println!("{}", "-".repeat(56));

        let mut profiles: Vec<_> = self.profiles.values().collect();
        profiles.sort_by(|a, b| b.call_count.cmp(&a.call_count));

        for profile in profiles {
            let jit_marker = if profile.is_jitted { " [JIT]" } else { "" };
            println!(
                "{:<20} {:>10} {:>12.2?} {:>10.2?}{}",
                profile.name,
                profile.call_count,
                profile.total_time,
                profile.avg_time(),
                jit_marker
            );
        }
    }

    /// 프로파일 초기화
    pub fn reset(&mut self) {
        self.profiles.clear();
        self.execution_stack.clear();
    }
}

impl Default for ExecutionProfiler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;

    #[test]
    fn test_profiler_basic() {
        let mut profiler = ExecutionProfiler::new();

        // 함수 호출 시뮬레이션
        for _ in 0..150 {
            profiler.begin_call("factorial", "Int");
            sleep(Duration::from_micros(10));
            profiler.end_call("factorial");
        }

        let profile = profiler.get_profile("factorial").unwrap();
        assert_eq!(profile.call_count, 150);
        assert!(profile.should_jit());
    }

    #[test]
    fn test_hot_function_detection() {
        let mut profiler = ExecutionProfiler::new();

        // 핫 함수
        for _ in 0..150 {
            profiler.begin_call("hot_func", "Int");
            profiler.end_call("hot_func");
        }

        // 콜드 함수
        for _ in 0..10 {
            profiler.begin_call("cold_func", "Int");
            profiler.end_call("cold_func");
        }

        let hot_funcs = profiler.get_hot_functions();
        assert_eq!(hot_funcs.len(), 1);
        assert_eq!(hot_funcs[0].name, "hot_func");
    }

    #[test]
    fn test_type_histogram() {
        let mut profiler = ExecutionProfiler::new();

        for _ in 0..80 {
            profiler.begin_call("add", "Int,Int");
            profiler.end_call("add");
        }
        for _ in 0..20 {
            profiler.begin_call("add", "Float,Float");
            profiler.end_call("add");
        }

        let profile = profiler.get_profile("add").unwrap();
        assert_eq!(profile.dominant_type_pattern(), Some("Int,Int"));
    }
}
