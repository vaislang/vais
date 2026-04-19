//! System functions (environment, process, signal)

use std::collections::HashMap;

use super::*;

impl TypeChecker {
    pub(super) fn register_system_builtins(&mut self) {
        // ===== System functions (env/process/signal) =====
        #[allow(clippy::type_complexity)]
        let sys_fns: Vec<(&str, Vec<(String, ResolvedType, bool)>, ResolvedType)> = vec![
            (
                "getenv",
                vec![("name".to_string(), ResolvedType::Str, false)],
                ResolvedType::I64,
            ),
            (
                "setenv",
                vec![
                    ("name".to_string(), ResolvedType::Str, false),
                    ("value".to_string(), ResolvedType::Str, false),
                    ("overwrite".to_string(), ResolvedType::I32, false),
                ],
                ResolvedType::I32,
            ),
            (
                "unsetenv",
                vec![("name".to_string(), ResolvedType::Str, false)],
                ResolvedType::I32,
            ),
            (
                "system",
                vec![("command".to_string(), ResolvedType::Str, false)],
                ResolvedType::I32,
            ),
            (
                "popen",
                vec![
                    ("command".to_string(), ResolvedType::Str, false),
                    ("mode".to_string(), ResolvedType::Str, false),
                ],
                ResolvedType::I64,
            ),
            (
                "pclose",
                vec![("stream".to_string(), ResolvedType::I64, false)],
                ResolvedType::I32,
            ),
            (
                "exit",
                vec![("status".to_string(), ResolvedType::I32, false)],
                ResolvedType::Unit,
            ),
            (
                "signal",
                vec![
                    ("signum".to_string(), ResolvedType::I32, false),
                    ("handler".to_string(), ResolvedType::I64, false),
                ],
                ResolvedType::I64,
            ),
            (
                "raise",
                vec![("signum".to_string(), ResolvedType::I32, false)],
                ResolvedType::I32,
            ),
            // === Async / platform runtime helpers (mirror codegen platform builtins) ===
            (
                "usleep",
                vec![("usec".to_string(), ResolvedType::I64, false)],
                ResolvedType::I32,
            ),
            ("sched_yield", vec![], ResolvedType::I32),
            (
                "call_poll",
                vec![
                    ("poll_fn".to_string(), ResolvedType::I64, false),
                    ("future_ptr".to_string(), ResolvedType::I64, false),
                ],
                ResolvedType::I64,
            ),
            (
                "extract_poll_status",
                vec![("poll_result".to_string(), ResolvedType::I64, false)],
                ResolvedType::I64,
            ),
            (
                "extract_poll_value",
                vec![("poll_result".to_string(), ResolvedType::I64, false)],
                ResolvedType::I64,
            ),
            ("time_now_ms", vec![], ResolvedType::I64),
            ("async_platform", vec![], ResolvedType::I64),
            (
                "close",
                vec![("fd".to_string(), ResolvedType::I64, false)],
                ResolvedType::I64,
            ),
            (
                "pipe",
                vec![("fds_buf".to_string(), ResolvedType::I64, false)],
                ResolvedType::I64,
            ),
            (
                "write_byte",
                vec![
                    ("fd".to_string(), ResolvedType::I64, false),
                    ("value".to_string(), ResolvedType::I64, false),
                ],
                ResolvedType::I64,
            ),
            (
                "read_byte",
                vec![("fd".to_string(), ResolvedType::I64, false)],
                ResolvedType::I64,
            ),
            (
                "epoll_set_timer_ms",
                vec![
                    ("kq".to_string(), ResolvedType::I64, false),
                    ("timer_id".to_string(), ResolvedType::I64, false),
                    ("delay_ms".to_string(), ResolvedType::I64, false),
                ],
                ResolvedType::I64,
            ),
            (
                "iocp_set_timer_ms",
                vec![
                    ("kq".to_string(), ResolvedType::I64, false),
                    ("timer_id".to_string(), ResolvedType::I64, false),
                    ("delay_ms".to_string(), ResolvedType::I64, false),
                ],
                ResolvedType::I64,
            ),
            ("kqueue", vec![], ResolvedType::I64),
            (
                "kevent_register",
                vec![
                    ("kq".to_string(), ResolvedType::I64, false),
                    ("fd".to_string(), ResolvedType::I64, false),
                    ("filter".to_string(), ResolvedType::I64, false),
                    ("flags".to_string(), ResolvedType::I64, false),
                ],
                ResolvedType::I64,
            ),
            (
                "kevent_wait",
                vec![
                    ("kq".to_string(), ResolvedType::I64, false),
                    ("events_buf".to_string(), ResolvedType::I64, false),
                    ("max_events".to_string(), ResolvedType::I64, false),
                    ("timeout_ms".to_string(), ResolvedType::I64, false),
                ],
                ResolvedType::I64,
            ),
            (
                "kevent_get_fd",
                vec![
                    ("events_buf".to_string(), ResolvedType::I64, false),
                    ("index".to_string(), ResolvedType::I64, false),
                ],
                ResolvedType::I64,
            ),
            (
                "kevent_get_filter",
                vec![
                    ("events_buf".to_string(), ResolvedType::I64, false),
                    ("index".to_string(), ResolvedType::I64, false),
                ],
                ResolvedType::I64,
            ),
        ];
        for (name, params, ret) in sys_fns {
            self.functions.insert(
                name.to_string(),
                FunctionSig {
                    name: name.to_string(),
                    generics: vec![],
                    generic_bounds: HashMap::new(),
                    params,
                    ret,
                    is_async: false,
                    is_vararg: false,
                    required_params: None,
                    contracts: None,
                    effect_annotation: EffectAnnotation::Infer,
                    inferred_effects: None,
                    generic_callees: vec![],
                },
            );
        }
    }
}
