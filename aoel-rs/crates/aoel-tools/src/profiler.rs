//! AOEL Profiler
//!
//! 함수 호출 횟수, 실행 시간 등을 측정

use std::collections::HashMap;
use std::time::{Duration, Instant};
use aoel_ir::Value;
use aoel_lowering::CompiledFunction;

/// 함수별 프로파일링 정보
#[derive(Debug, Clone, Default)]
pub struct FunctionProfile {
    /// 호출 횟수
    pub call_count: u64,
    /// 총 실행 시간
    pub total_time: Duration,
    /// 최소 실행 시간
    pub min_time: Option<Duration>,
    /// 최대 실행 시간
    pub max_time: Option<Duration>,
    /// 자식 함수 호출 시간 제외한 순수 시간
    pub self_time: Duration,
}

impl FunctionProfile {
    pub fn new() -> Self {
        Self::default()
    }

    /// 평균 실행 시간
    pub fn avg_time(&self) -> Duration {
        if self.call_count == 0 {
            Duration::ZERO
        } else {
            self.total_time / self.call_count as u32
        }
    }

    /// 호출 기록
    pub fn record(&mut self, duration: Duration) {
        self.call_count += 1;
        self.total_time += duration;

        if self.min_time.map_or(true, |min| duration < min) {
            self.min_time = Some(duration);
        }
        if self.max_time.map_or(true, |max| duration > max) {
            self.max_time = Some(duration);
        }
    }
}

/// 프로파일링 결과
#[derive(Debug, Clone)]
pub struct ProfileResult {
    /// 함수별 프로파일
    pub functions: HashMap<String, FunctionProfile>,
    /// 전체 실행 시간
    pub total_time: Duration,
    /// 실행 결과
    pub result: Option<Value>,
}

impl ProfileResult {
    /// 프로파일 요약 출력
    pub fn summary(&self) -> String {
        let mut output = String::new();
        output.push_str("=== Profile Summary ===\n\n");
        output.push_str(&format!("Total execution time: {:?}\n\n", self.total_time));

        // 함수별 통계 (시간 순 정렬)
        let mut funcs: Vec<_> = self.functions.iter().collect();
        funcs.sort_by(|a, b| b.1.total_time.cmp(&a.1.total_time));

        output.push_str("Function                 Calls      Total       Avg        Min        Max\n");
        output.push_str("─────────────────────────────────────────────────────────────────────────\n");

        for (name, profile) in funcs {
            let name_truncated = if name.len() > 20 {
                format!("{}...", &name[..17])
            } else {
                name.clone()
            };

            output.push_str(&format!(
                "{:<20} {:>8}  {:>10.2?}  {:>10.2?}  {:>10.2?}  {:>10.2?}\n",
                name_truncated,
                profile.call_count,
                profile.total_time,
                profile.avg_time(),
                profile.min_time.unwrap_or(Duration::ZERO),
                profile.max_time.unwrap_or(Duration::ZERO),
            ));
        }

        if let Some(result) = &self.result {
            output.push_str(&format!("\nResult: {}\n", result));
        }

        output
    }

    /// JSON 형식으로 출력
    pub fn to_json(&self) -> String {
        let mut funcs = Vec::new();
        for (name, profile) in &self.functions {
            funcs.push(format!(
                r#"    "{}": {{
      "calls": {},
      "total_ms": {:.3},
      "avg_ms": {:.3},
      "min_ms": {:.3},
      "max_ms": {:.3}
    }}"#,
                name,
                profile.call_count,
                profile.total_time.as_secs_f64() * 1000.0,
                profile.avg_time().as_secs_f64() * 1000.0,
                profile.min_time.unwrap_or(Duration::ZERO).as_secs_f64() * 1000.0,
                profile.max_time.unwrap_or(Duration::ZERO).as_secs_f64() * 1000.0,
            ));
        }

        format!(
            r#"{{
  "total_ms": {:.3},
  "functions": {{
{}
  }}
}}"#,
            self.total_time.as_secs_f64() * 1000.0,
            funcs.join(",\n")
        )
    }
}

/// 프로파일링 VM
pub struct Profiler {
    profiles: HashMap<String, FunctionProfile>,
    call_stack: Vec<(String, Instant)>,
}

impl Profiler {
    pub fn new() -> Self {
        Self {
            profiles: HashMap::new(),
            call_stack: Vec::new(),
        }
    }

    /// 함수 호출 시작
    pub fn enter_function(&mut self, name: &str) {
        self.call_stack.push((name.to_string(), Instant::now()));
    }

    /// 함수 호출 종료
    pub fn exit_function(&mut self) {
        if let Some((name, start)) = self.call_stack.pop() {
            let duration = start.elapsed();
            self.profiles
                .entry(name)
                .or_insert_with(FunctionProfile::new)
                .record(duration);
        }
    }

    /// 프로그램 실행 및 프로파일링
    pub fn profile(&mut self, functions: Vec<CompiledFunction>) -> ProfileResult {
        let start = Instant::now();

        // VM으로 실행
        let mut vm = aoel_vm::Vm::new();
        vm.load_functions(functions.clone());

        // __main__ 또는 첫 번째 함수 실행
        let target = if vm.has_function("__main__") {
            "__main__"
        } else if let Some(f) = functions.first() {
            &f.name
        } else {
            return ProfileResult {
                functions: self.profiles.clone(),
                total_time: start.elapsed(),
                result: None,
            };
        };

        // 프로파일링 VM 실행
        self.enter_function(target);
        let result = vm.call_function(target, vec![]).ok();
        self.exit_function();

        // 함수 호출 횟수 기반 추정 (실제로는 VM instrumentation 필요)
        // 여기서는 단순히 함수 목록을 기록
        for func in &functions {
            self.profiles
                .entry(func.name.clone())
                .or_insert_with(FunctionProfile::new);
        }

        ProfileResult {
            functions: self.profiles.clone(),
            total_time: start.elapsed(),
            result,
        }
    }

    /// 프로파일 초기화
    pub fn reset(&mut self) {
        self.profiles.clear();
        self.call_stack.clear();
    }
}

impl Default for Profiler {
    fn default() -> Self {
        Self::new()
    }
}

/// VM에 has_function 메서드 추가를 위한 트레잇
trait VmExt {
    fn has_function(&self, name: &str) -> bool;
}

impl VmExt for aoel_vm::Vm {
    fn has_function(&self, _name: &str) -> bool {
        // 실제 구현은 VM 수정 필요
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_profile() {
        let mut profile = FunctionProfile::new();

        profile.record(Duration::from_millis(10));
        profile.record(Duration::from_millis(20));
        profile.record(Duration::from_millis(15));

        assert_eq!(profile.call_count, 3);
        assert_eq!(profile.min_time, Some(Duration::from_millis(10)));
        assert_eq!(profile.max_time, Some(Duration::from_millis(20)));
    }

    #[test]
    fn test_profiler_basic() {
        let mut profiler = Profiler::new();

        profiler.enter_function("test_fn");
        std::thread::sleep(Duration::from_millis(10));
        profiler.exit_function();

        assert!(profiler.profiles.contains_key("test_fn"));
        assert_eq!(profiler.profiles["test_fn"].call_count, 1);
    }
}
