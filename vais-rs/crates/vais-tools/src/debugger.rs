//! Vais Debugger
//!
//! 브레이크포인트, 스텝 실행, 변수 검사 등 디버깅 기능

use std::collections::HashMap;
use vais_ir::{Instruction, Value};
use vais_lowering::CompiledFunction;

/// 브레이크포인트
#[derive(Debug, Clone)]
pub struct Breakpoint {
    pub id: usize,
    pub function: String,
    pub instruction: usize,
    pub enabled: bool,
    pub hit_count: usize,
    pub condition: Option<String>,
}

/// 스택 프레임
#[derive(Debug, Clone)]
pub struct StackFrame {
    pub function: String,
    pub instruction_pointer: usize,
    pub locals: HashMap<String, Value>,
}

/// 디버거 상태
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DebugState {
    /// 실행 전
    NotStarted,
    /// 실행 중
    Running,
    /// 일시 정지 (브레이크포인트)
    Paused,
    /// 스텝 실행 중
    Stepping,
    /// 실행 완료
    Finished,
}

/// 디버그 이벤트
#[derive(Debug, Clone)]
pub enum DebugEvent {
    /// 브레이크포인트 도달
    BreakpointHit { breakpoint_id: usize, function: String },
    /// 스텝 완료
    StepComplete { function: String, instruction: usize },
    /// 함수 진입
    FunctionEnter { name: String },
    /// 함수 종료
    FunctionExit { name: String, result: Value },
    /// 에러 발생
    Error { message: String },
    /// 실행 완료
    Finished { result: Value },
}

/// Vais Debugger
pub struct Debugger {
    /// 컴파일된 함수들
    functions: HashMap<String, CompiledFunction>,
    /// 브레이크포인트
    breakpoints: Vec<Breakpoint>,
    /// 다음 브레이크포인트 ID
    next_breakpoint_id: usize,
    /// 콜 스택
    call_stack: Vec<StackFrame>,
    /// 현재 상태
    state: DebugState,
    /// 이벤트 로그
    events: Vec<DebugEvent>,
    /// 감시 표현식
    watches: Vec<String>,
}

impl Debugger {
    pub fn new() -> Self {
        Self {
            functions: HashMap::new(),
            breakpoints: Vec::new(),
            next_breakpoint_id: 1,
            call_stack: Vec::new(),
            state: DebugState::NotStarted,
            events: Vec::new(),
            watches: Vec::new(),
        }
    }

    /// 함수 로드
    pub fn load_functions(&mut self, functions: Vec<CompiledFunction>) {
        for func in functions {
            self.functions.insert(func.name.clone(), func);
        }
    }

    /// 브레이크포인트 설정
    pub fn set_breakpoint(&mut self, function: &str, instruction: usize) -> usize {
        let id = self.next_breakpoint_id;
        self.next_breakpoint_id += 1;

        self.breakpoints.push(Breakpoint {
            id,
            function: function.to_string(),
            instruction,
            enabled: true,
            hit_count: 0,
            condition: None,
        });

        id
    }

    /// 조건부 브레이크포인트 설정
    pub fn set_conditional_breakpoint(
        &mut self,
        function: &str,
        instruction: usize,
        condition: &str,
    ) -> usize {
        let id = self.set_breakpoint(function, instruction);
        if let Some(bp) = self.breakpoints.iter_mut().find(|b| b.id == id) {
            bp.condition = Some(condition.to_string());
        }
        id
    }

    /// 브레이크포인트 제거
    pub fn remove_breakpoint(&mut self, id: usize) -> bool {
        if let Some(pos) = self.breakpoints.iter().position(|b| b.id == id) {
            self.breakpoints.remove(pos);
            true
        } else {
            false
        }
    }

    /// 브레이크포인트 활성화/비활성화
    pub fn toggle_breakpoint(&mut self, id: usize) -> Option<bool> {
        if let Some(bp) = self.breakpoints.iter_mut().find(|b| b.id == id) {
            bp.enabled = !bp.enabled;
            Some(bp.enabled)
        } else {
            None
        }
    }

    /// 모든 브레이크포인트 목록
    pub fn list_breakpoints(&self) -> &[Breakpoint] {
        &self.breakpoints
    }

    /// 감시 표현식 추가
    pub fn add_watch(&mut self, expr: &str) {
        self.watches.push(expr.to_string());
    }

    /// 감시 표현식 제거
    pub fn remove_watch(&mut self, expr: &str) -> bool {
        if let Some(pos) = self.watches.iter().position(|w| w == expr) {
            self.watches.remove(pos);
            true
        } else {
            false
        }
    }

    /// 현재 콜 스택
    pub fn call_stack(&self) -> &[StackFrame] {
        &self.call_stack
    }

    /// 현재 스택 프레임
    pub fn current_frame(&self) -> Option<&StackFrame> {
        self.call_stack.last()
    }

    /// 변수 값 조회
    pub fn get_variable(&self, name: &str) -> Option<&Value> {
        self.call_stack
            .last()
            .and_then(|frame| frame.locals.get(name))
    }

    /// 모든 로컬 변수
    pub fn locals(&self) -> Option<&HashMap<String, Value>> {
        self.call_stack.last().map(|frame| &frame.locals)
    }

    /// 현재 상태
    pub fn state(&self) -> DebugState {
        self.state
    }

    /// 이벤트 로그
    pub fn events(&self) -> &[DebugEvent] {
        &self.events
    }

    /// 이벤트 로그 클리어
    pub fn clear_events(&mut self) {
        self.events.clear();
    }

    /// 디버그 세션 시작
    pub fn start(&mut self, entry: &str, args: Vec<Value>) -> Result<(), String> {
        if !self.functions.contains_key(entry) {
            return Err(format!("Function '{}' not found", entry));
        }

        self.state = DebugState::Running;
        self.call_stack.clear();
        self.events.clear();

        // 초기 스택 프레임 생성
        let func = &self.functions[entry];
        let mut locals = HashMap::new();
        for (i, param) in func.params.iter().enumerate() {
            if i < args.len() {
                locals.insert(param.clone(), args[i].clone());
            }
        }

        self.call_stack.push(StackFrame {
            function: entry.to_string(),
            instruction_pointer: 0,
            locals,
        });

        self.events.push(DebugEvent::FunctionEnter {
            name: entry.to_string(),
        });

        Ok(())
    }

    /// 다음 명령어까지 스텝
    pub fn step(&mut self) -> Option<DebugEvent> {
        if self.state != DebugState::Paused && self.state != DebugState::Running {
            return None;
        }

        self.state = DebugState::Stepping;

        // 현재 프레임
        let frame = self.call_stack.last_mut()?;
        let func = self.functions.get(&frame.function)?;

        if frame.instruction_pointer >= func.instructions.len() {
            // 함수 종료
            let result = Value::Void; // 실제로는 스택에서 가져와야 함
            let name = frame.function.clone();
            self.call_stack.pop();

            let event = DebugEvent::FunctionExit {
                name: name.clone(),
                result: result.clone(),
            };
            self.events.push(event.clone());

            if self.call_stack.is_empty() {
                self.state = DebugState::Finished;
                return Some(DebugEvent::Finished { result });
            }

            return Some(event);
        }

        // 다음 명령어 실행
        let _instr = &func.instructions[frame.instruction_pointer];
        frame.instruction_pointer += 1;

        // 브레이크포인트 체크
        for bp in &mut self.breakpoints {
            if bp.enabled
                && bp.function == frame.function
                && bp.instruction == frame.instruction_pointer - 1
            {
                bp.hit_count += 1;
                self.state = DebugState::Paused;
                let event = DebugEvent::BreakpointHit {
                    breakpoint_id: bp.id,
                    function: frame.function.clone(),
                };
                self.events.push(event.clone());
                return Some(event);
            }
        }

        self.state = DebugState::Paused;
        let event = DebugEvent::StepComplete {
            function: frame.function.clone(),
            instruction: frame.instruction_pointer - 1,
        };
        self.events.push(event.clone());
        Some(event)
    }

    /// 다음 브레이크포인트까지 실행
    pub fn continue_execution(&mut self) -> Option<DebugEvent> {
        self.state = DebugState::Running;

        loop {
            match self.step() {
                Some(DebugEvent::BreakpointHit { .. }) => {
                    return self.events.last().cloned();
                }
                Some(DebugEvent::Finished { .. }) => {
                    return self.events.last().cloned();
                }
                Some(DebugEvent::Error { .. }) => {
                    return self.events.last().cloned();
                }
                None => {
                    return None;
                }
                _ => continue,
            }
        }
    }

    /// 함수 진입 (step into)
    pub fn step_into(&mut self) -> Option<DebugEvent> {
        // 다음 명령어가 Call이면 함수 내부로 진입
        self.step()
    }

    /// 라인 넘기기 (step over) - 함수 호출을 건너뜀
    pub fn step_over(&mut self) -> Option<DebugEvent> {
        let current_depth = self.call_stack.len();

        loop {
            match self.step() {
                Some(event @ DebugEvent::StepComplete { .. }) => {
                    // 같은 깊이로 돌아오면 완료
                    if self.call_stack.len() <= current_depth {
                        return Some(event);
                    }
                }
                Some(DebugEvent::FunctionExit { .. }) => {
                    if self.call_stack.len() < current_depth {
                        return self.events.last().cloned();
                    }
                }
                Some(DebugEvent::Finished { .. }) | Some(DebugEvent::Error { .. }) | None => {
                    return self.events.last().cloned();
                }
                Some(DebugEvent::BreakpointHit { .. }) => {
                    return self.events.last().cloned();
                }
                _ => continue,
            }
        }
    }

    /// 일시 정지
    pub fn pause(&mut self) {
        if self.state == DebugState::Running {
            self.state = DebugState::Paused;
        }
    }

    /// 함수 나가기 (step out)
    pub fn step_out(&mut self) -> Option<DebugEvent> {
        let current_depth = self.call_stack.len();

        loop {
            match self.step() {
                Some(DebugEvent::FunctionExit { .. }) => {
                    if self.call_stack.len() < current_depth {
                        return self.events.last().cloned();
                    }
                }
                Some(DebugEvent::Finished { .. }) | Some(DebugEvent::Error { .. }) | None => {
                    return self.events.last().cloned();
                }
                _ => continue,
            }
        }
    }

    /// 현재 명령어 정보
    pub fn current_instruction(&self) -> Option<&Instruction> {
        let frame = self.call_stack.last()?;
        let func = self.functions.get(&frame.function)?;
        func.instructions.get(frame.instruction_pointer)
    }

    /// 콜 스택 반환
    pub fn get_call_stack(&self) -> &[StackFrame] {
        &self.call_stack
    }

    /// 디버그 정보 요약
    pub fn summary(&self) -> String {
        let mut output = String::new();

        output.push_str(&format!("State: {:?}\n", self.state));
        output.push_str(&format!("Call stack depth: {}\n", self.call_stack.len()));

        if let Some(frame) = self.current_frame() {
            output.push_str(&format!(
                "Current: {}:{}\n",
                frame.function, frame.instruction_pointer
            ));

            output.push_str("Locals:\n");
            for (name, value) in &frame.locals {
                output.push_str(&format!("  {} = {}\n", name, value));
            }
        }

        output.push_str(&format!("\nBreakpoints: {}\n", self.breakpoints.len()));
        for bp in &self.breakpoints {
            output.push_str(&format!(
                "  #{}: {}:{} [{}] hits={}\n",
                bp.id,
                bp.function,
                bp.instruction,
                if bp.enabled { "enabled" } else { "disabled" },
                bp.hit_count
            ));
        }

        output
    }

    /// 리셋
    pub fn reset(&mut self) {
        self.call_stack.clear();
        self.events.clear();
        self.state = DebugState::NotStarted;
        for bp in &mut self.breakpoints {
            bp.hit_count = 0;
        }
    }
}

impl Default for Debugger {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_breakpoint_management() {
        let mut debugger = Debugger::new();

        let id1 = debugger.set_breakpoint("main", 0);
        let id2 = debugger.set_breakpoint("main", 5);

        assert_eq!(debugger.list_breakpoints().len(), 2);

        debugger.toggle_breakpoint(id1);
        assert!(!debugger.list_breakpoints()[0].enabled);

        debugger.remove_breakpoint(id2);
        assert_eq!(debugger.list_breakpoints().len(), 1);
    }

    #[test]
    fn test_watch_management() {
        let mut debugger = Debugger::new();

        debugger.add_watch("x");
        debugger.add_watch("y + 1");

        assert_eq!(debugger.watches.len(), 2);

        debugger.remove_watch("x");
        assert_eq!(debugger.watches.len(), 1);
        assert_eq!(debugger.watches[0], "y + 1");
    }
}
