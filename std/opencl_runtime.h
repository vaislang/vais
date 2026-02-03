// OpenCL GPU runtime API for VAIS - Header file
// C-compatible function declarations for linking with Vais compiler output.

#ifndef VAIS_OPENCL_RUNTIME_H
#define VAIS_OPENCL_RUNTIME_H

#ifdef __cplusplus
extern "C" {
#endif

// ============================================
// Initialization & cleanup
// ============================================
long  opencl_init(void);
long  opencl_init_source(const char* kernel_source);
long  opencl_init_file(const char* cl_file_path);
long  opencl_cleanup(void);

// ============================================
// Memory management
// ============================================
void* opencl_alloc(long size);
void* opencl_alloc_read(long size);
void* opencl_alloc_write(long size);
long  opencl_free(void* buffer_handle);
long  opencl_memcpy_h2d(void* buffer_handle, const void* src, long size);
long  opencl_memcpy_d2h(void* dst, void* buffer_handle, long size);

// ============================================
// Kernel execution
// ============================================
void* opencl_create_kernel(const char* kernel_name);
long  opencl_destroy_kernel(void* kernel_handle);
long  opencl_set_arg(void* kernel_handle, long index, long size, const void* value);
long  opencl_set_arg_buffer(void* kernel_handle, long index, void* buffer_handle);
long  opencl_dispatch(
    void* kernel_handle,
    long global_x, long global_y, long global_z,
    long local_x, long local_y, long local_z
);
long  opencl_dispatch_auto(
    void* kernel_handle,
    long total_work_items
);
long  opencl_synchronize(void);

// ============================================
// Device management
// ============================================
long        opencl_platform_count(void);
long        opencl_device_count(void);
const char* opencl_platform_name(void);
const char* opencl_device_name(void);
const char* opencl_device_vendor(void);
long        opencl_device_max_compute_units(void);
long        opencl_device_max_work_group_size(void);
long        opencl_device_global_mem(void);
long        opencl_device_local_mem(void);

// ============================================
// Event / Profiling
// ============================================
void*  opencl_event_create(void);
long   opencl_event_destroy(void* event_handle);
long   opencl_event_wait(void* event_handle);
double opencl_event_elapsed(void* start_handle, void* end_handle);

// ============================================
// Async dispatch
// ============================================
long  opencl_dispatch_async(
    void* kernel_handle,
    long global_x, long global_y, long global_z,
    long local_x, long local_y, long local_z,
    void** out_event
);

// ============================================
// Multi-device selection
// ============================================
long  opencl_device_select(long device_index);

// ============================================
// Error handling
// ============================================
long        opencl_last_error(void);
const char* opencl_error_string(long error_code);

#ifdef __cplusplus
}
#endif

#endif // VAIS_OPENCL_RUNTIME_H
