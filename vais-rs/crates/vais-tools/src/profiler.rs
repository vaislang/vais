//! Vais Profiler
//!
//! 함수 호출 횟수, 실행 시간 등을 측정
//! Flame Graph 시각화 지원

use std::collections::HashMap;
use std::time::{Duration, Instant};
use vais_ir::Value;
use vais_lowering::CompiledFunction;

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

        if self.min_time.is_none_or(|min| duration < min) {
            self.min_time = Some(duration);
        }
        if self.max_time.is_none_or(|max| duration > max) {
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
                .or_default()
                .record(duration);
        }
    }

    /// 프로그램 실행 및 프로파일링
    pub fn profile(&mut self, functions: Vec<CompiledFunction>) -> ProfileResult {
        let start = Instant::now();

        // VM으로 실행
        let mut vm = vais_vm::Vm::new();
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
                .or_default();
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

// =============================================================================
// Flame Graph Support
// =============================================================================

/// Flame Graph 이벤트 (call stack sample)
#[derive(Debug, Clone)]
pub struct FlameEvent {
    /// 함수 이름
    pub name: String,
    /// 시작 시간 (나노초)
    pub start_ns: u64,
    /// 지속 시간 (나노초)
    pub duration_ns: u64,
    /// 스택 깊이
    pub depth: usize,
}

/// Flame Graph 데이터
#[derive(Debug, Clone, Default)]
pub struct FlameGraph {
    /// 모든 이벤트들
    pub events: Vec<FlameEvent>,
    /// 시작 시간
    start_time: Option<Instant>,
    /// 현재 스택
    call_stack: Vec<(String, Instant, usize)>, // (name, start, depth)
}

impl FlameGraph {
    pub fn new() -> Self {
        Self::default()
    }

    /// 프로파일링 시작
    pub fn start(&mut self) {
        self.start_time = Some(Instant::now());
        self.events.clear();
        self.call_stack.clear();
    }

    /// 함수 진입
    pub fn enter(&mut self, name: &str) {
        let depth = self.call_stack.len();
        self.call_stack.push((name.to_string(), Instant::now(), depth));
    }

    /// 함수 종료
    pub fn exit(&mut self) {
        if let Some((name, start, depth)) = self.call_stack.pop() {
            let duration = start.elapsed();
            let start_ns = if let Some(base) = self.start_time {
                start.duration_since(base).as_nanos() as u64
            } else {
                0
            };

            self.events.push(FlameEvent {
                name,
                start_ns,
                duration_ns: duration.as_nanos() as u64,
                depth,
            });
        }
    }

    /// Collapsed 형식 출력 (flamegraph.pl 호환)
    /// 형식: stack;stack;stack count
    pub fn to_collapsed(&self) -> String {
        // 스택 트레이스별로 시간 집계
        let mut stacks: HashMap<String, u64> = HashMap::new();

        // 이벤트를 스택 문자열로 변환
        for event in &self.events {
            // 단일 함수 스택
            let stack = event.name.clone();
            *stacks.entry(stack).or_default() += event.duration_ns / 1000; // 마이크로초로
        }

        let mut output = String::new();
        for (stack, count) in stacks {
            output.push_str(&format!("{} {}\n", stack, count));
        }
        output
    }

    /// Speedscope JSON 형식 출력 (브라우저 호환)
    pub fn to_speedscope_json(&self) -> String {
        let mut frames = Vec::new();
        let mut frame_index: HashMap<String, usize> = HashMap::new();

        // 프레임 인덱스 생성
        for event in &self.events {
            if !frame_index.contains_key(&event.name) {
                let idx = frames.len();
                frame_index.insert(event.name.clone(), idx);
                frames.push(format!(r#"{{"name": "{}"}}"#, event.name));
            }
        }

        // 이벤트를 샘플로 변환
        let mut samples = Vec::new();
        let mut weights = Vec::new();

        for event in &self.events {
            if let Some(&idx) = frame_index.get(&event.name) {
                samples.push(format!("[{}]", idx));
                weights.push(event.duration_ns.to_string());
            }
        }

        format!(
            r#"{{
  "$schema": "https://www.speedscope.app/file-format-schema.json",
  "version": "0.0.1",
  "shared": {{
    "frames": [{}]
  }},
  "profiles": [{{
    "type": "sampled",
    "name": "vais-profile",
    "unit": "nanoseconds",
    "startValue": 0,
    "endValue": {},
    "samples": [{}],
    "weights": [{}]
  }}]
}}"#,
            frames.join(", "),
            self.events.iter().map(|e| e.duration_ns).sum::<u64>(),
            samples.join(", "),
            weights.join(", ")
        )
    }

    /// HTML Flame Graph 생성 (인라인 JavaScript)
    pub fn to_html(&self) -> String {
        let events_json = self.events
            .iter()
            .map(|e| {
                format!(
                    r#"{{"name":"{}","start":{},"duration":{},"depth":{}}}"#,
                    e.name, e.start_ns, e.duration_ns, e.depth
                )
            })
            .collect::<Vec<_>>()
            .join(",");

        format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>Vais Flame Graph</title>
    <style>
        body {{ font-family: -apple-system, BlinkMacSystemFont, sans-serif; margin: 0; padding: 20px; background: #1e1e1e; color: #fff; }}
        h1 {{ font-size: 1.5em; margin-bottom: 20px; }}
        #flame {{ width: 100%; height: 600px; overflow: auto; }}
        .frame {{ position: absolute; height: 20px; line-height: 20px; font-size: 11px; padding: 0 4px; box-sizing: border-box; overflow: hidden; white-space: nowrap; text-overflow: ellipsis; border-radius: 2px; cursor: pointer; }}
        .frame:hover {{ z-index: 1000 !important; box-shadow: 0 0 10px rgba(255,255,255,0.5); }}
        .tooltip {{ position: fixed; background: #333; padding: 8px 12px; border-radius: 4px; font-size: 12px; pointer-events: none; z-index: 10000; }}
    </style>
</head>
<body>
    <h1>Vais Flame Graph</h1>
    <div id="flame" style="position: relative;"></div>
    <div id="tooltip" class="tooltip" style="display: none;"></div>
    <script>
        const events = [{events_json}];
        const container = document.getElementById('flame');
        const tooltip = document.getElementById('tooltip');

        // 색상 팔레트 (따뜻한 색상들)
        const colors = ['#f59e0b', '#ef4444', '#ec4899', '#8b5cf6', '#3b82f6', '#10b981', '#f97316', '#14b8a6'];

        function hashColor(name) {{
            let hash = 0;
            for (let i = 0; i < name.length; i++) {{
                hash = ((hash << 5) - hash) + name.charCodeAt(i);
            }}
            return colors[Math.abs(hash) % colors.length];
        }}

        // 전체 시간 범위 계산
        const maxEnd = Math.max(...events.map(e => e.start + e.duration));
        const maxDepth = Math.max(...events.map(e => e.depth)) + 1;

        container.style.height = (maxDepth * 24 + 40) + 'px';

        events.forEach((event, i) => {{
            const frame = document.createElement('div');
            frame.className = 'frame';
            frame.style.left = (event.start / maxEnd * 100) + '%';
            frame.style.width = Math.max(event.duration / maxEnd * 100, 0.1) + '%';
            frame.style.bottom = (event.depth * 24) + 'px';
            frame.style.background = hashColor(event.name);
            frame.textContent = event.name;

            frame.addEventListener('mousemove', (e) => {{
                tooltip.style.display = 'block';
                tooltip.style.left = (e.clientX + 10) + 'px';
                tooltip.style.top = (e.clientY + 10) + 'px';
                tooltip.innerHTML = `<b>${{event.name}}</b><br>Duration: ${{(event.duration / 1000000).toFixed(3)}} ms`;
            }});

            frame.addEventListener('mouseout', () => {{
                tooltip.style.display = 'none';
            }});

            container.appendChild(frame);
        }});
    </script>
</body>
</html>"#,
            events_json = events_json
        )
    }

    /// SVG Flame Graph 생성
    pub fn to_svg(&self, width: u32, height: u32) -> String {
        if self.events.is_empty() {
            return r#"<svg xmlns="http://www.w3.org/2000/svg" width="800" height="400"><text x="50%" y="50%" text-anchor="middle">No data</text></svg>"#.to_string();
        }

        let max_end = self.events.iter().map(|e| e.start_ns + e.duration_ns).max().unwrap_or(1);
        let max_depth = self.events.iter().map(|e| e.depth).max().unwrap_or(0) + 1;
        let row_height = height as f64 / max_depth as f64;

        let mut rects = String::new();
        let colors = ["#f59e0b", "#ef4444", "#ec4899", "#8b5cf6", "#3b82f6", "#10b981"];

        for (i, event) in self.events.iter().enumerate() {
            let x = (event.start_ns as f64 / max_end as f64) * width as f64;
            let w = (event.duration_ns as f64 / max_end as f64) * width as f64;
            let y = height as f64 - (event.depth + 1) as f64 * row_height;
            let color = colors[i % colors.len()];

            rects.push_str(&format!(
                r##"<rect x="{:.1}" y="{:.1}" width="{:.1}" height="{:.1}" fill="{}" stroke="#333" stroke-width="0.5"><title>{} ({:.3}ms)</title></rect>"##,
                x, y, w.max(1.0), row_height - 1.0, color, event.name, event.duration_ns as f64 / 1_000_000.0
            ));

            // 텍스트 (너비가 충분할 때만)
            if w > 40.0 {
                rects.push_str(&format!(
                    r#"<text x="{:.1}" y="{:.1}" font-size="11" fill="white" pointer-events="none">{}</text>"#,
                    x + 4.0, y + row_height / 2.0 + 4.0,
                    if event.name.len() > 20 { &event.name[..17] } else { &event.name }
                ));
            }
        }

        format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" style="background:#1e1e1e">
{}</svg>"#,
            width, height, rects
        )
    }
}

// =============================================================================
// Memory Profiler
// =============================================================================

/// 메모리 할당 이벤트
#[derive(Debug, Clone)]
pub struct MemoryEvent {
    /// 이벤트 타입
    pub event_type: MemoryEventType,
    /// 값 타입
    pub value_type: String,
    /// 크기 (바이트)
    pub size: usize,
    /// 타임스탬프 (나노초)
    pub timestamp_ns: u64,
}

/// 메모리 이벤트 타입
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MemoryEventType {
    Alloc,
    Free,
}

/// 메모리 프로파일러
#[derive(Debug, Clone, Default)]
pub struct MemoryProfiler {
    /// 이벤트 로그
    pub events: Vec<MemoryEvent>,
    /// 타입별 현재 할당량
    pub allocations: HashMap<String, usize>,
    /// 타입별 할당 횟수
    pub alloc_count: HashMap<String, u64>,
    /// 타입별 해제 횟수
    pub free_count: HashMap<String, u64>,
    /// 시작 시간
    start_time: Option<Instant>,
    /// 현재 총 메모리
    pub current_memory: usize,
    /// 최대 메모리
    pub peak_memory: usize,
    /// 총 할당된 메모리
    pub total_allocated: usize,
    /// 총 해제된 메모리
    pub total_freed: usize,
}

impl MemoryProfiler {
    pub fn new() -> Self {
        Self::default()
    }

    /// 프로파일링 시작
    pub fn start(&mut self) {
        self.start_time = Some(Instant::now());
        self.events.clear();
        self.allocations.clear();
        self.alloc_count.clear();
        self.free_count.clear();
        self.current_memory = 0;
        self.peak_memory = 0;
        self.total_allocated = 0;
        self.total_freed = 0;
    }

    /// 할당 기록
    pub fn record_alloc(&mut self, value_type: &str, size: usize) {
        let timestamp_ns = self.start_time
            .map(|t| t.elapsed().as_nanos() as u64)
            .unwrap_or(0);

        self.events.push(MemoryEvent {
            event_type: MemoryEventType::Alloc,
            value_type: value_type.to_string(),
            size,
            timestamp_ns,
        });

        *self.allocations.entry(value_type.to_string()).or_default() += size;
        *self.alloc_count.entry(value_type.to_string()).or_default() += 1;
        self.current_memory += size;
        self.total_allocated += size;

        if self.current_memory > self.peak_memory {
            self.peak_memory = self.current_memory;
        }
    }

    /// 해제 기록
    pub fn record_free(&mut self, value_type: &str, size: usize) {
        let timestamp_ns = self.start_time
            .map(|t| t.elapsed().as_nanos() as u64)
            .unwrap_or(0);

        self.events.push(MemoryEvent {
            event_type: MemoryEventType::Free,
            value_type: value_type.to_string(),
            size,
            timestamp_ns,
        });

        if let Some(alloc) = self.allocations.get_mut(value_type) {
            *alloc = alloc.saturating_sub(size);
        }
        *self.free_count.entry(value_type.to_string()).or_default() += 1;
        self.current_memory = self.current_memory.saturating_sub(size);
        self.total_freed += size;
    }

    /// Value 크기 추정 (바이트)
    pub fn estimate_value_size(value: &Value) -> usize {
        use std::mem::size_of;

        match value {
            Value::Void => 0,
            Value::Int(_) => size_of::<i64>(),
            Value::Float(_) => size_of::<f64>(),
            Value::Bool(_) => size_of::<bool>(),
            Value::String(s) => size_of::<String>() + s.len(),
            Value::Array(arr) => {
                size_of::<Vec<Value>>()
                    + arr.iter().map(Self::estimate_value_size).sum::<usize>()
            }
            Value::Bytes(b) => size_of::<Vec<u8>>() + b.len(),
            Value::Struct(fields) => {
                size_of::<HashMap<String, Value>>()
                    + fields
                        .iter()
                        .map(|(k, v)| k.len() + Self::estimate_value_size(v))
                        .sum::<usize>()
            }
            Value::Map(map) => {
                size_of::<HashMap<String, Value>>()
                    + map
                        .iter()
                        .map(|(k, v)| k.len() + Self::estimate_value_size(v))
                        .sum::<usize>()
            }
            Value::Closure { params, captured, .. } => {
                size_of::<usize>() * 2
                    + params.iter().map(|p| p.len()).sum::<usize>()
                    + captured.iter().map(|(k, v)| k.len() + Self::estimate_value_size(v)).sum::<usize>()
            }
            Value::Error(_) => size_of::<String>() + 32,
            Value::Optional(opt) => {
                size_of::<Option<Box<Value>>>()
                    + opt.as_ref().map(|v| Self::estimate_value_size(v)).unwrap_or(0)
            }
            Value::Future(_) | Value::Channel(_) => size_of::<u64>(),
        }
    }

    /// 요약 문자열
    pub fn summary(&self) -> String {
        let mut output = String::new();
        output.push_str("=== Memory Profile Summary ===\n\n");

        output.push_str(&format!(
            "Current Memory: {} bytes ({:.2} KB)\n",
            self.current_memory,
            self.current_memory as f64 / 1024.0
        ));
        output.push_str(&format!(
            "Peak Memory:    {} bytes ({:.2} KB)\n",
            self.peak_memory,
            self.peak_memory as f64 / 1024.0
        ));
        output.push_str(&format!(
            "Total Allocated: {} bytes ({:.2} KB)\n",
            self.total_allocated,
            self.total_allocated as f64 / 1024.0
        ));
        output.push_str(&format!(
            "Total Freed:     {} bytes ({:.2} KB)\n\n",
            self.total_freed,
            self.total_freed as f64 / 1024.0
        ));

        output.push_str("Type                   Allocs    Frees     Current\n");
        output.push_str("────────────────────────────────────────────────────\n");

        let mut types: Vec<_> = self.allocations.keys().collect();
        types.sort();

        for ty in types {
            let allocs = self.alloc_count.get(ty).unwrap_or(&0);
            let frees = self.free_count.get(ty).unwrap_or(&0);
            let current = self.allocations.get(ty).unwrap_or(&0);

            output.push_str(&format!(
                "{:<20} {:>8}  {:>8}  {:>8} bytes\n",
                ty, allocs, frees, current
            ));
        }

        output
    }

    /// JSON 형식 출력
    pub fn to_json(&self) -> String {
        let types_json: Vec<String> = self
            .allocations
            .iter()
            .map(|(ty, size)| {
                format!(
                    r#"    "{}": {{
      "allocs": {},
      "frees": {},
      "current_bytes": {}
    }}"#,
                    ty,
                    self.alloc_count.get(ty).unwrap_or(&0),
                    self.free_count.get(ty).unwrap_or(&0),
                    size
                )
            })
            .collect();

        format!(
            r#"{{
  "current_memory": {},
  "peak_memory": {},
  "total_allocated": {},
  "total_freed": {},
  "types": {{
{}
  }}
}}"#,
            self.current_memory,
            self.peak_memory,
            self.total_allocated,
            self.total_freed,
            types_json.join(",\n")
        )
    }

    /// 메모리 타임라인 SVG 생성
    pub fn to_timeline_svg(&self, width: u32, height: u32) -> String {
        if self.events.is_empty() {
            return r#"<svg xmlns="http://www.w3.org/2000/svg" width="800" height="400"><text x="50%" y="50%" text-anchor="middle">No data</text></svg>"#.to_string();
        }

        let max_time = self.events.iter().map(|e| e.timestamp_ns).max().unwrap_or(1);

        // 시간별 메모리 사용량 계산
        let mut points = Vec::new();
        let mut current = 0i64;

        for event in &self.events {
            match event.event_type {
                MemoryEventType::Alloc => current += event.size as i64,
                MemoryEventType::Free => current -= event.size as i64,
            }
            points.push((event.timestamp_ns, current.max(0) as usize));
        }

        let max_memory = points.iter().map(|(_, m)| *m).max().unwrap_or(1);

        // SVG 라인 생성
        let line_points: String = points
            .iter()
            .map(|(t, m)| {
                let x = (*t as f64 / max_time as f64) * width as f64;
                let y = height as f64 - (*m as f64 / max_memory as f64) * (height as f64 - 40.0);
                format!("{:.1},{:.1}", x, y)
            })
            .collect::<Vec<_>>()
            .join(" ");

        format!(
            r##"<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" style="background:#1e1e1e">
  <text x="10" y="20" fill="white" font-size="14">Memory Timeline</text>
  <text x="10" y="{}" fill="#888" font-size="11">Peak: {} KB</text>
  <polyline points="{}" fill="none" stroke="#3b82f6" stroke-width="2"/>
  <line x1="0" y1="{}" x2="{}" y2="{}" stroke="#444" stroke-width="1"/>
</svg>"##,
            width,
            height,
            height - 5,
            self.peak_memory / 1024,
            line_points,
            height - 20,
            width,
            height - 20
        )
    }
}

/// VM에 has_function 메서드 추가를 위한 트레잇
trait VmExt {
    fn has_function(&self, name: &str) -> bool;
}

impl VmExt for vais_vm::Vm {
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

    // === Flame Graph Tests ===

    #[test]
    fn test_flame_graph_basic() {
        let mut fg = FlameGraph::new();
        fg.start();

        fg.enter("main");
        fg.enter("helper");
        fg.exit(); // helper
        fg.exit(); // main

        assert_eq!(fg.events.len(), 2);
        assert_eq!(fg.events[0].name, "helper");
        assert_eq!(fg.events[1].name, "main");
    }

    #[test]
    fn test_flame_graph_depth() {
        let mut fg = FlameGraph::new();
        fg.start();

        fg.enter("a");  // depth 0
        fg.enter("b");  // depth 1
        fg.enter("c");  // depth 2
        fg.exit();      // c
        fg.exit();      // b
        fg.exit();      // a

        assert_eq!(fg.events.len(), 3);
        assert_eq!(fg.events[0].depth, 2); // c
        assert_eq!(fg.events[1].depth, 1); // b
        assert_eq!(fg.events[2].depth, 0); // a
    }

    #[test]
    fn test_flame_graph_collapsed_output() {
        let mut fg = FlameGraph::new();
        fg.start();

        fg.enter("main");
        std::thread::sleep(Duration::from_micros(100));
        fg.exit();

        let collapsed = fg.to_collapsed();
        assert!(collapsed.contains("main"));
    }

    #[test]
    fn test_flame_graph_svg_output() {
        let mut fg = FlameGraph::new();
        fg.start();

        fg.enter("test");
        fg.exit();

        let svg = fg.to_svg(800, 400);
        assert!(svg.contains("<svg"));
        assert!(svg.contains("</svg>"));
    }

    #[test]
    fn test_flame_graph_html_output() {
        let mut fg = FlameGraph::new();
        fg.start();

        fg.enter("test");
        fg.exit();

        let html = fg.to_html();
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("Flame Graph"));
    }

    #[test]
    fn test_flame_graph_speedscope_output() {
        let mut fg = FlameGraph::new();
        fg.start();

        fg.enter("test");
        fg.exit();

        let json = fg.to_speedscope_json();
        assert!(json.contains("speedscope.app"));
        assert!(json.contains("\"name\": \"test\""));
    }

    // === Memory Profiler Tests ===

    #[test]
    fn test_memory_profiler_basic() {
        let mut mp = MemoryProfiler::new();
        mp.start();

        mp.record_alloc("Array", 1024);
        mp.record_alloc("String", 256);

        assert_eq!(mp.current_memory, 1280);
        assert_eq!(mp.peak_memory, 1280);
        assert_eq!(mp.total_allocated, 1280);
    }

    #[test]
    fn test_memory_profiler_peak() {
        let mut mp = MemoryProfiler::new();
        mp.start();

        mp.record_alloc("Array", 1000);
        mp.record_alloc("Array", 500);
        mp.record_free("Array", 1000);

        assert_eq!(mp.current_memory, 500);
        assert_eq!(mp.peak_memory, 1500);
    }

    #[test]
    fn test_memory_profiler_type_tracking() {
        let mut mp = MemoryProfiler::new();
        mp.start();

        mp.record_alloc("Array", 100);
        mp.record_alloc("Array", 200);
        mp.record_alloc("String", 50);

        assert_eq!(mp.alloc_count.get("Array"), Some(&2));
        assert_eq!(mp.alloc_count.get("String"), Some(&1));
    }

    #[test]
    fn test_memory_profiler_summary() {
        let mut mp = MemoryProfiler::new();
        mp.start();

        mp.record_alloc("Array", 1024);

        let summary = mp.summary();
        assert!(summary.contains("Memory Profile Summary"));
        assert!(summary.contains("Array"));
    }

    #[test]
    fn test_memory_profiler_json() {
        let mut mp = MemoryProfiler::new();
        mp.start();

        mp.record_alloc("Array", 1024);

        let json = mp.to_json();
        assert!(json.contains("\"current_memory\": 1024"));
        assert!(json.contains("\"Array\""));
    }

    #[test]
    fn test_memory_profiler_timeline_svg() {
        let mut mp = MemoryProfiler::new();
        mp.start();

        mp.record_alloc("Array", 100);
        mp.record_alloc("Array", 200);
        mp.record_free("Array", 100);

        let svg = mp.to_timeline_svg(800, 400);
        assert!(svg.contains("<svg"));
        assert!(svg.contains("Memory Timeline"));
    }

    #[test]
    fn test_memory_value_size_estimation() {
        let int_size = MemoryProfiler::estimate_value_size(&Value::Int(42));
        assert_eq!(int_size, std::mem::size_of::<i64>());

        let bool_size = MemoryProfiler::estimate_value_size(&Value::Bool(true));
        assert_eq!(bool_size, std::mem::size_of::<bool>());
    }
}
