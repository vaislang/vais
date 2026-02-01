// Metal GPU runtime API for VAIS - Header file
// C-compatible function declarations for linking with Vais compiler output.

#ifndef VAIS_METAL_RUNTIME_H
#define VAIS_METAL_RUNTIME_H

#ifdef __cplusplus
extern "C" {
#endif

// ============================================
// Initialization
// ============================================
long  metal_init(void);
long  metal_init_library(const char* metallib_path);
long  metal_init_source(const char* source);

// ============================================
// Memory management
// ============================================
void* metal_alloc(long size);
long  metal_free(void* buffer_handle);
long  metal_memcpy_h2d(void* buffer_handle, const void* src, long size);
long  metal_memcpy_d2h(void* dst, void* buffer_handle, long size);
void* metal_buffer_contents(void* buffer_handle);
long  metal_buffer_length(void* buffer_handle);

// ============================================
// Kernel execution
// ============================================
void* metal_create_pipeline(const char* kernel_name);
long  metal_destroy_pipeline(void* pipeline_handle);
long  metal_dispatch(
    void* pipeline_handle,
    void** buffers,
    long buffer_count,
    long grid_x, long grid_y, long grid_z,
    long block_x, long block_y, long block_z
);
long  metal_dispatch_auto(
    void* pipeline_handle,
    void** buffers,
    long buffer_count,
    long total_threads
);

// ============================================
// Device management
// ============================================
long        metal_device_count(void);
const char* metal_device_name(void);
long        metal_supports_family(long family);
long        metal_recommended_max_working_set(void);
long        metal_max_threadgroup_memory(void);
long        metal_max_threads_per_threadgroup(void);

// ============================================
// Cleanup
// ============================================
long  metal_cleanup(void);

#ifdef __cplusplus
}
#endif

#endif // VAIS_METAL_RUNTIME_H
