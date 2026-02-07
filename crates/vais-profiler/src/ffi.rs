use crate::{Profiler, ProfilerConfig};
use parking_lot::Mutex;
use std::ffi::{c_char, c_void, CStr};
use std::sync::Arc;

static GLOBAL_PROFILER: Mutex<Option<Arc<Profiler>>> = Mutex::new(None);

#[repr(C)]
#[derive(Clone, Copy)]
pub struct VaisProfilerConfig {
    pub sample_interval_ms: u64,
    pub track_memory: bool,
    pub build_call_graph: bool,
    pub max_samples: usize,
}

impl Default for VaisProfilerConfig {
    fn default() -> Self {
        Self {
            sample_interval_ms: 1,
            track_memory: true,
            build_call_graph: true,
            max_samples: 1_000_000,
        }
    }
}

impl From<VaisProfilerConfig> for ProfilerConfig {
    fn from(config: VaisProfilerConfig) -> Self {
        Self {
            mode: crate::ProfilerMode::All,
            sample_interval: std::time::Duration::from_millis(config.sample_interval_ms),
            track_memory: config.track_memory,
            build_call_graph: config.build_call_graph,
            max_samples: config.max_samples,
        }
    }
}

#[no_mangle]
pub extern "C" fn vais_profiler_create(config: *const VaisProfilerConfig) -> *mut c_void {
    let config = if config.is_null() {
        ProfilerConfig::default()
    } else {
        unsafe { (*config).into() }
    };

    let profiler = Arc::new(Profiler::new(config));
    Arc::into_raw(profiler) as *mut c_void
}

#[no_mangle]
pub extern "C" fn vais_profiler_destroy(profiler: *mut c_void) {
    if !profiler.is_null() {
        unsafe {
            let _ = Arc::from_raw(profiler as *const Profiler);
        }
    }
}

#[no_mangle]
pub extern "C" fn vais_profiler_start(profiler: *mut c_void) -> bool {
    if profiler.is_null() {
        return false;
    }

    let profiler = unsafe { &*(profiler as *const Profiler) };
    profiler.start().is_ok()
}

#[no_mangle]
pub extern "C" fn vais_profiler_stop(profiler: *mut c_void) -> bool {
    if profiler.is_null() {
        return false;
    }

    let profiler = unsafe { &*(profiler as *const Profiler) };
    profiler.stop().is_ok()
}

#[no_mangle]
pub extern "C" fn vais_profiler_is_running(profiler: *mut c_void) -> bool {
    if profiler.is_null() {
        return false;
    }

    let profiler = unsafe { &*(profiler as *const Profiler) };
    profiler.is_running()
}

#[no_mangle]
pub extern "C" fn vais_profiler_record_sample(
    profiler: *mut c_void,
    function_name: *const c_char,
    instruction_pointer: usize,
) {
    if profiler.is_null() || function_name.is_null() {
        return;
    }

    let profiler = unsafe { &*(profiler as *const Profiler) };
    let function_name = unsafe { CStr::from_ptr(function_name) };

    if let Ok(function_name) = function_name.to_str() {
        profiler.record_sample(function_name, instruction_pointer);
    }
}

#[no_mangle]
pub extern "C" fn vais_profiler_record_allocation(
    profiler: *mut c_void,
    size: usize,
    address: usize,
) {
    if profiler.is_null() {
        return;
    }

    let profiler = unsafe { &*(profiler as *const Profiler) };
    profiler.record_allocation(size, address);
}

#[no_mangle]
pub extern "C" fn vais_profiler_record_deallocation(profiler: *mut c_void, address: usize) {
    if profiler.is_null() {
        return;
    }

    let profiler = unsafe { &*(profiler as *const Profiler) };
    profiler.record_deallocation(address);
}

#[no_mangle]
pub extern "C" fn vais_profiler_record_call(
    profiler: *mut c_void,
    caller: *const c_char,
    callee: *const c_char,
) {
    if profiler.is_null() || caller.is_null() || callee.is_null() {
        return;
    }

    let profiler = unsafe { &*(profiler as *const Profiler) };
    let caller = unsafe { CStr::from_ptr(caller) };
    let callee = unsafe { CStr::from_ptr(callee) };

    if let (Ok(caller), Ok(callee)) = (caller.to_str(), callee.to_str()) {
        profiler.record_call(caller, callee);
    }
}

#[no_mangle]
pub extern "C" fn vais_profiler_global_init(config: *const VaisProfilerConfig) -> bool {
    let config = if config.is_null() {
        ProfilerConfig::default()
    } else {
        unsafe { (*config).into() }
    };

    let mut global = GLOBAL_PROFILER.lock();
    if global.is_some() {
        return false;
    }

    *global = Some(Arc::new(Profiler::new(config)));
    true
}

#[no_mangle]
pub extern "C" fn vais_profiler_global_destroy() {
    let mut global = GLOBAL_PROFILER.lock();
    *global = None;
}

#[no_mangle]
pub extern "C" fn vais_profiler_global_start() -> bool {
    let global = GLOBAL_PROFILER.lock();
    if let Some(profiler) = global.as_ref() {
        profiler.start().is_ok()
    } else {
        false
    }
}

#[no_mangle]
pub extern "C" fn vais_profiler_global_stop() -> bool {
    let global = GLOBAL_PROFILER.lock();
    if let Some(profiler) = global.as_ref() {
        profiler.stop().is_ok()
    } else {
        false
    }
}

#[no_mangle]
pub extern "C" fn vais_profiler_global_record_sample(
    function_name: *const c_char,
    instruction_pointer: usize,
) {
    if function_name.is_null() {
        return;
    }

    let global = GLOBAL_PROFILER.lock();
    if let Some(profiler) = global.as_ref() {
        let function_name = unsafe { CStr::from_ptr(function_name) };
        if let Ok(function_name) = function_name.to_str() {
            profiler.record_sample(function_name, instruction_pointer);
        }
    }
}

#[no_mangle]
pub extern "C" fn vais_profiler_global_record_allocation(size: usize, address: usize) {
    let global = GLOBAL_PROFILER.lock();
    if let Some(profiler) = global.as_ref() {
        profiler.record_allocation(size, address);
    }
}

#[no_mangle]
pub extern "C" fn vais_profiler_global_record_deallocation(address: usize) {
    let global = GLOBAL_PROFILER.lock();
    if let Some(profiler) = global.as_ref() {
        profiler.record_deallocation(address);
    }
}

#[no_mangle]
pub extern "C" fn vais_profiler_global_record_call(caller: *const c_char, callee: *const c_char) {
    if caller.is_null() || callee.is_null() {
        return;
    }

    let global = GLOBAL_PROFILER.lock();
    if let Some(profiler) = global.as_ref() {
        let caller = unsafe { CStr::from_ptr(caller) };
        let callee = unsafe { CStr::from_ptr(callee) };

        if let (Ok(caller), Ok(callee)) = (caller.to_str(), callee.to_str()) {
            profiler.record_call(caller, callee);
        }
    }
}

#[repr(C)]
pub struct VaisProfileStats {
    pub sample_count: usize,
    pub total_allocations: usize,
    pub total_allocated_bytes: usize,
    pub current_allocated_bytes: usize,
    pub peak_allocated_bytes: usize,
    pub call_graph_edges: usize,
}

#[no_mangle]
pub extern "C" fn vais_profiler_get_stats(profiler: *mut c_void) -> VaisProfileStats {
    if profiler.is_null() {
        return VaisProfileStats {
            sample_count: 0,
            total_allocations: 0,
            total_allocated_bytes: 0,
            current_allocated_bytes: 0,
            peak_allocated_bytes: 0,
            call_graph_edges: 0,
        };
    }

    let profiler = unsafe { &*(profiler as *const Profiler) };
    let snapshot = profiler.snapshot();

    VaisProfileStats {
        sample_count: snapshot.samples.iter().map(|(_, count)| count).sum(),
        total_allocations: snapshot.memory_stats.total_allocations,
        total_allocated_bytes: snapshot.memory_stats.total_allocated_bytes,
        current_allocated_bytes: snapshot.memory_stats.current_allocated_bytes,
        peak_allocated_bytes: snapshot.memory_stats.peak_allocated_bytes,
        call_graph_edges: snapshot.call_graph.len(),
    }
}

#[no_mangle]
pub extern "C" fn vais_profiler_global_get_stats() -> VaisProfileStats {
    let global = GLOBAL_PROFILER.lock();
    if let Some(profiler) = global.as_ref() {
        let snapshot = profiler.snapshot();
        VaisProfileStats {
            sample_count: snapshot.samples.iter().map(|(_, count)| count).sum(),
            total_allocations: snapshot.memory_stats.total_allocations,
            total_allocated_bytes: snapshot.memory_stats.total_allocated_bytes,
            current_allocated_bytes: snapshot.memory_stats.current_allocated_bytes,
            peak_allocated_bytes: snapshot.memory_stats.peak_allocated_bytes,
            call_graph_edges: snapshot.call_graph.len(),
        }
    } else {
        VaisProfileStats {
            sample_count: 0,
            total_allocations: 0,
            total_allocated_bytes: 0,
            current_allocated_bytes: 0,
            peak_allocated_bytes: 0,
            call_graph_edges: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn test_profiler_create_destroy() {
        let profiler = vais_profiler_create(std::ptr::null());
        assert!(!profiler.is_null());
        vais_profiler_destroy(profiler);
    }

    #[test]
    fn test_profiler_lifecycle() {
        let profiler = vais_profiler_create(std::ptr::null());

        assert!(!vais_profiler_is_running(profiler));
        assert!(vais_profiler_start(profiler));
        assert!(vais_profiler_is_running(profiler));
        assert!(vais_profiler_stop(profiler));
        assert!(!vais_profiler_is_running(profiler));

        vais_profiler_destroy(profiler);
    }

    #[test]
    fn test_profiler_record_sample() {
        let profiler = vais_profiler_create(std::ptr::null());
        vais_profiler_start(profiler);

        let func_name = CString::new("test_function").unwrap();
        vais_profiler_record_sample(profiler, func_name.as_ptr(), 0x1000);

        let stats = vais_profiler_get_stats(profiler);
        assert_eq!(stats.sample_count, 1);

        vais_profiler_stop(profiler);
        vais_profiler_destroy(profiler);
    }

    #[test]
    fn test_profiler_record_allocation() {
        let profiler = vais_profiler_create(std::ptr::null());
        vais_profiler_start(profiler);

        vais_profiler_record_allocation(profiler, 100, 0x1000);
        vais_profiler_record_allocation(profiler, 200, 0x2000);

        let stats = vais_profiler_get_stats(profiler);
        assert_eq!(stats.total_allocations, 2);
        assert_eq!(stats.total_allocated_bytes, 300);

        vais_profiler_stop(profiler);
        vais_profiler_destroy(profiler);
    }

    #[test]
    fn test_profiler_record_call() {
        let profiler = vais_profiler_create(std::ptr::null());
        vais_profiler_start(profiler);

        let caller = CString::new("main").unwrap();
        let callee = CString::new("foo").unwrap();
        vais_profiler_record_call(profiler, caller.as_ptr(), callee.as_ptr());

        let stats = vais_profiler_get_stats(profiler);
        assert_eq!(stats.call_graph_edges, 1);

        vais_profiler_stop(profiler);
        vais_profiler_destroy(profiler);
    }

    #[test]
    fn test_global_profiler() {
        // All global profiler tests run in a single test to avoid race conditions
        // since they share the same global Mutex<Option<Arc<Profiler>>> state.
        vais_profiler_global_destroy();

        // Test init/double-init/start/stop lifecycle
        assert!(vais_profiler_global_init(std::ptr::null()));
        assert!(!vais_profiler_global_init(std::ptr::null()));
        assert!(vais_profiler_global_start());
        assert!(vais_profiler_global_stop());
        vais_profiler_global_destroy();

        // Test recording samples, allocations, and call graph edges
        vais_profiler_global_init(std::ptr::null());
        vais_profiler_global_start();

        let func_name = CString::new("test").unwrap();
        vais_profiler_global_record_sample(func_name.as_ptr(), 0x1000);
        vais_profiler_global_record_allocation(100, 0x1000);

        let caller = CString::new("main").unwrap();
        let callee = CString::new("foo").unwrap();
        vais_profiler_global_record_call(caller.as_ptr(), callee.as_ptr());

        let stats = vais_profiler_global_get_stats();
        assert_eq!(stats.sample_count, 1);
        assert_eq!(stats.total_allocations, 1);
        assert_eq!(stats.call_graph_edges, 1);

        vais_profiler_global_stop();
        vais_profiler_global_destroy();
    }

    #[test]
    fn test_custom_config() {
        let config = VaisProfilerConfig {
            sample_interval_ms: 10,
            track_memory: false,
            build_call_graph: true,
            max_samples: 1000,
        };

        let profiler = vais_profiler_create(&config as *const _);
        assert!(!profiler.is_null());
        vais_profiler_destroy(profiler);
    }
}
