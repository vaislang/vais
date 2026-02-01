// GPU runtime API for VAIS - Header file
// C-compatible function declarations for linking with Vais compiler output.

#ifndef VAIS_GPU_RUNTIME_H
#define VAIS_GPU_RUNTIME_H

#ifdef __cplusplus
extern "C" {
#endif

// ============================================
// Memory management
// ============================================
void* gpu_alloc(long size);
long  gpu_free(void* ptr);
long  gpu_memcpy_h2d(void* dst, const void* src, long size);
long  gpu_memcpy_d2h(void* dst, const void* src, long size);
long  gpu_memcpy_d2d(void* dst, const void* src, long size);
long  gpu_memset(void* ptr, long value, long size);
void* gpu_alloc_managed(long size);

// ============================================
// Kernel execution
// ============================================
long gpu_launch_kernel(
    void* kernel_func,
    long grid_x, long grid_y, long grid_z,
    long block_x, long block_y, long block_z,
    long shared_mem,
    void** args,
    long arg_count
);
long gpu_launch_kernel_stream(
    void* kernel_func,
    long grid_x, long grid_y, long grid_z,
    long block_x, long block_y, long block_z,
    long shared_mem,
    void** args,
    long arg_count,
    void* stream
);
long gpu_synchronize(void);

// ============================================
// Stream management
// ============================================
void* gpu_stream_create(void);
long  gpu_stream_destroy(void* stream);
long  gpu_stream_synchronize(void* stream);

// ============================================
// Device management
// ============================================
long        gpu_device_count(void);
long        gpu_set_device(long device_id);
long        gpu_get_device(void);
const char* gpu_device_name(long device_id);
long        gpu_device_total_mem(long device_id);
long        gpu_device_max_threads(long device_id);

// ============================================
// Device properties (struct)
// ============================================
typedef struct {
    char name[256];
    long total_global_mem;
    long shared_mem_per_block;
    long max_threads_per_block;
    long max_block_dim_x;
    long max_block_dim_y;
    long max_block_dim_z;
    long max_grid_dim_x;
    long max_grid_dim_y;
    long max_grid_dim_z;
    long warp_size;
    long multiprocessor_count;
    long clock_rate_khz;
    long compute_major;
    long compute_minor;
} gpu_device_props_t;

long gpu_get_properties(long device_id, gpu_device_props_t* props);

// ============================================
// Event timing
// ============================================
void*  gpu_event_create(void);
long   gpu_event_destroy(void* event);
long   gpu_event_record(void* event);
long   gpu_event_synchronize(void* event);
double gpu_event_elapsed(void* start, void* end);

// ============================================
// Error handling
// ============================================
long        gpu_last_error(void);
const char* gpu_last_error_string(void);
long        gpu_reset_error(void);

#ifdef __cplusplus
}
#endif

#endif // VAIS_GPU_RUNTIME_H
