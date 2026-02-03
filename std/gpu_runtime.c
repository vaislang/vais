// GPU runtime support for VAIS - CUDA backend
// Provides CUDA Runtime API wrappers for GPU memory management,
// kernel execution, and device management.
//
// Linked automatically by vaisc when std/gpu is imported with --gpu cuda.
// Requires: CUDA Toolkit (libcudart)

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

// CUDA Runtime API header
#include <cuda_runtime.h>

// ============================================
// Error handling helpers
// ============================================

static int gpu_check_error(cudaError_t err, const char* operation) {
    if (err != cudaSuccess) {
        fprintf(stderr, "[vais-gpu] CUDA error in %s: %s\n",
                operation, cudaGetErrorString(err));
        return -1;
    }
    return 0;
}

// ============================================
// Memory management API
// ============================================

// Allocate device memory. Returns device pointer, or NULL on failure.
void* gpu_alloc(long size) {
    void* ptr = NULL;
    cudaError_t err = cudaMalloc(&ptr, (size_t)size);
    if (gpu_check_error(err, "gpu_alloc") != 0) {
        return NULL;
    }
    return ptr;
}

// Free device memory.
long gpu_free(void* ptr) {
    if (ptr == NULL) return 0;
    cudaError_t err = cudaFree(ptr);
    return gpu_check_error(err, "gpu_free");
}

// Copy data from host to device.
long gpu_memcpy_h2d(void* dst, const void* src, long size) {
    cudaError_t err = cudaMemcpy(dst, src, (size_t)size, cudaMemcpyHostToDevice);
    return gpu_check_error(err, "gpu_memcpy_h2d");
}

// Copy data from device to host.
long gpu_memcpy_d2h(void* dst, const void* src, long size) {
    cudaError_t err = cudaMemcpy(dst, src, (size_t)size, cudaMemcpyDeviceToHost);
    return gpu_check_error(err, "gpu_memcpy_d2h");
}

// Copy data between device buffers.
long gpu_memcpy_d2d(void* dst, const void* src, long size) {
    cudaError_t err = cudaMemcpy(dst, src, (size_t)size, cudaMemcpyDeviceToDevice);
    return gpu_check_error(err, "gpu_memcpy_d2d");
}

// Set device memory to a byte value.
long gpu_memset(void* ptr, long value, long size) {
    cudaError_t err = cudaMemset(ptr, (int)value, (size_t)size);
    return gpu_check_error(err, "gpu_memset");
}

// ============================================
// Kernel execution API
// ============================================

// Launch a pre-compiled CUDA kernel by function pointer.
// grid_x/y/z: grid dimensions
// block_x/y/z: block dimensions
// shared_mem: shared memory size in bytes
// args: array of kernel argument pointers
// arg_count: number of arguments
long gpu_launch_kernel(
    void* kernel_func,
    long grid_x, long grid_y, long grid_z,
    long block_x, long block_y, long block_z,
    long shared_mem,
    void** args,
    long arg_count
) {
    dim3 grid((unsigned int)grid_x, (unsigned int)grid_y, (unsigned int)grid_z);
    dim3 block((unsigned int)block_x, (unsigned int)block_y, (unsigned int)block_z);

    cudaError_t err = cudaLaunchKernel(
        kernel_func,
        grid,
        block,
        args,
        (size_t)shared_mem,
        NULL  // default stream
    );
    return gpu_check_error(err, "gpu_launch_kernel");
}

// Launch kernel on a specific CUDA stream.
long gpu_launch_kernel_stream(
    void* kernel_func,
    long grid_x, long grid_y, long grid_z,
    long block_x, long block_y, long block_z,
    long shared_mem,
    void** args,
    long arg_count,
    void* stream
) {
    dim3 grid((unsigned int)grid_x, (unsigned int)grid_y, (unsigned int)grid_z);
    dim3 block((unsigned int)block_x, (unsigned int)block_y, (unsigned int)block_z);

    cudaError_t err = cudaLaunchKernel(
        kernel_func,
        grid,
        block,
        args,
        (size_t)shared_mem,
        (cudaStream_t)stream
    );
    return gpu_check_error(err, "gpu_launch_kernel_stream");
}

// Synchronize device - wait for all pending GPU operations.
long gpu_synchronize(void) {
    cudaError_t err = cudaDeviceSynchronize();
    return gpu_check_error(err, "gpu_synchronize");
}

// ============================================
// Stream management
// ============================================

// Create a new CUDA stream. Returns stream handle, or NULL on failure.
void* gpu_stream_create(void) {
    cudaStream_t stream;
    cudaError_t err = cudaStreamCreate(&stream);
    if (gpu_check_error(err, "gpu_stream_create") != 0) {
        return NULL;
    }
    return (void*)stream;
}

// Destroy a CUDA stream.
long gpu_stream_destroy(void* stream) {
    cudaError_t err = cudaStreamDestroy((cudaStream_t)stream);
    return gpu_check_error(err, "gpu_stream_destroy");
}

// Synchronize a specific stream.
long gpu_stream_synchronize(void* stream) {
    cudaError_t err = cudaStreamSynchronize((cudaStream_t)stream);
    return gpu_check_error(err, "gpu_stream_synchronize");
}

// ============================================
// Device management API
// ============================================

// Get the number of CUDA-capable devices.
long gpu_device_count(void) {
    int count = 0;
    cudaError_t err = cudaGetDeviceCount(&count);
    if (gpu_check_error(err, "gpu_device_count") != 0) {
        return 0;
    }
    return (long)count;
}

// Set the active CUDA device.
long gpu_set_device(long device_id) {
    cudaError_t err = cudaSetDevice((int)device_id);
    return gpu_check_error(err, "gpu_set_device");
}

// Get the current active device ID.
long gpu_get_device(void) {
    int device_id = 0;
    cudaError_t err = cudaGetDevice(&device_id);
    if (gpu_check_error(err, "gpu_get_device") != 0) {
        return -1;
    }
    return (long)device_id;
}

// ============================================
// Device properties
// ============================================

// GPU device properties struct (Vais-compatible layout)
typedef struct {
    char name[256];
    long total_global_mem;      // Total global memory in bytes
    long shared_mem_per_block;  // Shared memory per block in bytes
    long max_threads_per_block; // Maximum threads per block
    long max_block_dim_x;       // Maximum block dimension X
    long max_block_dim_y;       // Maximum block dimension Y
    long max_block_dim_z;       // Maximum block dimension Z
    long max_grid_dim_x;        // Maximum grid dimension X
    long max_grid_dim_y;        // Maximum grid dimension Y
    long max_grid_dim_z;        // Maximum grid dimension Z
    long warp_size;             // Warp size
    long multiprocessor_count;  // Number of SMs
    long clock_rate_khz;        // Clock rate in KHz
    long compute_major;         // Compute capability major
    long compute_minor;         // Compute capability minor
} gpu_device_props_t;

// Get device properties. Returns 0 on success, -1 on failure.
long gpu_get_properties(long device_id, gpu_device_props_t* props) {
    if (props == NULL) return -1;

    cudaDeviceProp cuda_props;
    cudaError_t err = cudaGetDeviceProperties(&cuda_props, (int)device_id);
    if (gpu_check_error(err, "gpu_get_properties") != 0) {
        return -1;
    }

    memset(props, 0, sizeof(gpu_device_props_t));
    strncpy(props->name, cuda_props.name, 255);
    props->name[255] = '\0';
    props->total_global_mem      = (long)cuda_props.totalGlobalMem;
    props->shared_mem_per_block  = (long)cuda_props.sharedMemPerBlock;
    props->max_threads_per_block = (long)cuda_props.maxThreadsPerBlock;
    props->max_block_dim_x       = (long)cuda_props.maxThreadsDim[0];
    props->max_block_dim_y       = (long)cuda_props.maxThreadsDim[1];
    props->max_block_dim_z       = (long)cuda_props.maxThreadsDim[2];
    props->max_grid_dim_x        = (long)cuda_props.maxGridSize[0];
    props->max_grid_dim_y        = (long)cuda_props.maxGridSize[1];
    props->max_grid_dim_z        = (long)cuda_props.maxGridSize[2];
    props->warp_size             = (long)cuda_props.warpSize;
    props->multiprocessor_count  = (long)cuda_props.multiProcessorCount;
    props->clock_rate_khz        = (long)cuda_props.clockRate;
    props->compute_major         = (long)cuda_props.major;
    props->compute_minor         = (long)cuda_props.minor;

    return 0;
}

// Convenience: get device name as string pointer.
const char* gpu_device_name(long device_id) {
    static char name_buf[256];
    cudaDeviceProp props;
    cudaError_t err = cudaGetDeviceProperties(&props, (int)device_id);
    if (err != cudaSuccess) {
        return "unknown";
    }
    strncpy(name_buf, props.name, 255);
    name_buf[255] = '\0';
    return name_buf;
}

// Convenience: get total device memory in bytes.
long gpu_device_total_mem(long device_id) {
    cudaDeviceProp props;
    cudaError_t err = cudaGetDeviceProperties(&props, (int)device_id);
    if (err != cudaSuccess) return 0;
    return (long)props.totalGlobalMem;
}

// Convenience: get max threads per block.
long gpu_device_max_threads(long device_id) {
    cudaDeviceProp props;
    cudaError_t err = cudaGetDeviceProperties(&props, (int)device_id);
    if (err != cudaSuccess) return 0;
    return (long)props.maxThreadsPerBlock;
}

// ============================================
// Event timing API
// ============================================

// Create a CUDA event. Returns event handle, or NULL on failure.
void* gpu_event_create(void) {
    cudaEvent_t event;
    cudaError_t err = cudaEventCreate(&event);
    if (gpu_check_error(err, "gpu_event_create") != 0) {
        return NULL;
    }
    return (void*)event;
}

// Destroy a CUDA event.
long gpu_event_destroy(void* event) {
    cudaError_t err = cudaEventDestroy((cudaEvent_t)event);
    return gpu_check_error(err, "gpu_event_destroy");
}

// Record an event on the default stream.
long gpu_event_record(void* event) {
    cudaError_t err = cudaEventRecord((cudaEvent_t)event, 0);
    return gpu_check_error(err, "gpu_event_record");
}

// Synchronize on an event.
long gpu_event_synchronize(void* event) {
    cudaError_t err = cudaEventSynchronize((cudaEvent_t)event);
    return gpu_check_error(err, "gpu_event_synchronize");
}

// Get elapsed time between two events in milliseconds.
// Returns elapsed time as a float-encoded long (use *(float*)&result to decode).
double gpu_event_elapsed(void* start, void* end) {
    float ms = 0.0f;
    cudaError_t err = cudaEventElapsedTime(&ms, (cudaEvent_t)start, (cudaEvent_t)end);
    if (gpu_check_error(err, "gpu_event_elapsed") != 0) {
        return -1.0;
    }
    return (double)ms;
}

// ============================================
// Unified memory (CUDA managed memory)
// ============================================

// Allocate unified memory accessible from both host and device.
void* gpu_alloc_managed(long size) {
    void* ptr = NULL;
    cudaError_t err = cudaMallocManaged(&ptr, (size_t)size, cudaMemAttachGlobal);
    if (gpu_check_error(err, "gpu_alloc_managed") != 0) {
        return NULL;
    }
    return ptr;
}

// ============================================
// Async memory transfer
// ============================================

// Asynchronous host-to-device memory copy on a stream.
long gpu_memcpy_h2d_async(void* dst, const void* src, long size, void* stream) {
    cudaError_t err = cudaMemcpyAsync(dst, src, (size_t)size,
                                       cudaMemcpyHostToDevice, (cudaStream_t)stream);
    return gpu_check_error(err, "gpu_memcpy_h2d_async");
}

// Asynchronous device-to-host memory copy on a stream.
long gpu_memcpy_d2h_async(void* dst, const void* src, long size, void* stream) {
    cudaError_t err = cudaMemcpyAsync(dst, src, (size_t)size,
                                       cudaMemcpyDeviceToHost, (cudaStream_t)stream);
    return gpu_check_error(err, "gpu_memcpy_d2h_async");
}

// ============================================
// Unified memory hints
// ============================================

// Prefetch unified memory to a specific device.
long gpu_mem_prefetch(void* ptr, long size, long device_id) {
    cudaError_t err = cudaMemPrefetchAsync(ptr, (size_t)size, (int)device_id, 0);
    return gpu_check_error(err, "gpu_mem_prefetch");
}

// Advise the runtime about memory access patterns.
// advice values: 1=ReadMostly, 2=PreferredLocation, 3=AccessedBy
long gpu_mem_advise(void* ptr, long size, long advice, long device_id) {
    cudaMemoryAdvise cuda_advice;
    switch ((int)advice) {
        case 1: cuda_advice = cudaMemAdviseSetReadMostly; break;
        case 2: cuda_advice = cudaMemAdviseSetPreferredLocation; break;
        case 3: cuda_advice = cudaMemAdviseSetAccessedBy; break;
        default:
            fprintf(stderr, "[vais-gpu] CUDA error in gpu_mem_advise: unknown advice %ld\n", advice);
            return -1;
    }
    cudaError_t err = cudaMemAdvise(ptr, (size_t)size, cuda_advice, (int)device_id);
    return gpu_check_error(err, "gpu_mem_advise");
}

// ============================================
// Event-stream operations
// ============================================

// Record an event on a specific stream.
long gpu_event_record_stream(void* event, void* stream) {
    cudaError_t err = cudaEventRecord((cudaEvent_t)event, (cudaStream_t)stream);
    return gpu_check_error(err, "gpu_event_record_stream");
}

// ============================================
// Multi-GPU peer access
// ============================================

// Enable peer access from the current device to peer_device.
long gpu_peer_access_enable(long peer_device) {
    cudaError_t err = cudaDeviceEnablePeerAccess((int)peer_device, 0);
    return gpu_check_error(err, "gpu_peer_access_enable");
}

// Disable peer access from the current device to peer_device.
long gpu_peer_access_disable(long peer_device) {
    cudaError_t err = cudaDeviceDisablePeerAccess((int)peer_device);
    return gpu_check_error(err, "gpu_peer_access_disable");
}

// Check if device can access peer's memory directly.
// Returns 1 if peer access is possible, 0 if not, -1 on error.
long gpu_peer_can_access(long device, long peer) {
    int can_access = 0;
    cudaError_t err = cudaDeviceCanAccessPeer(&can_access, (int)device, (int)peer);
    if (gpu_check_error(err, "gpu_peer_can_access") != 0) {
        return -1;
    }
    return (long)can_access;
}

// Copy memory between devices (peer-to-peer).
long gpu_memcpy_peer(void* dst, long dst_device, const void* src, long src_device, long size) {
    cudaError_t err = cudaMemcpyPeer(dst, (int)dst_device, src, (int)src_device, (size_t)size);
    return gpu_check_error(err, "gpu_memcpy_peer");
}

// ============================================
// Error query
// ============================================

// Get the last CUDA error code (0 = success).
long gpu_last_error(void) {
    return (long)cudaGetLastError();
}

// Get the error string for the last CUDA error.
const char* gpu_last_error_string(void) {
    cudaError_t err = cudaGetLastError();
    return cudaGetErrorString(err);
}

// Reset/clear the last error.
long gpu_reset_error(void) {
    cudaGetLastError();  // Consumes and resets
    return 0;
}
