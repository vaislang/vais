// Metal GPU runtime support for VAIS - macOS/iOS backend
// Provides Metal Compute Shader API wrappers for GPU memory management,
// kernel (compute pipeline) execution, and device management.
//
// Linked automatically by vaisc when std/gpu is imported with --gpu metal --gpu-compile.
// Requires: macOS 10.13+ / iOS 11+ with Metal support

#import <Foundation/Foundation.h>
#import <Metal/Metal.h>
#include <stdio.h>
#include <string.h>

// ============================================
// Global Metal state
// ============================================

static id<MTLDevice> g_device = nil;
static id<MTLCommandQueue> g_command_queue = nil;
static id<MTLLibrary> g_library = nil;

// ============================================
// Initialization
// ============================================

// Initialize Metal runtime with default device.
// Returns 0 on success, -1 on failure.
long metal_init(void) {
    @autoreleasepool {
        g_device = MTLCreateSystemDefaultDevice();
        if (!g_device) {
            fprintf(stderr, "[vais-gpu] Metal error: No Metal-capable device found\n");
            return -1;
        }
        g_command_queue = [g_device newCommandQueue];
        if (!g_command_queue) {
            fprintf(stderr, "[vais-gpu] Metal error: Failed to create command queue\n");
            return -1;
        }
        return 0;
    }
}

// Initialize Metal with a compiled .metallib file.
// Returns 0 on success, -1 on failure.
long metal_init_library(const char* metallib_path) {
    @autoreleasepool {
        if (!g_device) {
            if (metal_init() != 0) return -1;
        }

        NSError* error = nil;
        NSString* path = [NSString stringWithUTF8String:metallib_path];
        NSURL* url = [NSURL fileURLWithPath:path];
        g_library = [g_device newLibraryWithURL:url error:&error];
        if (!g_library) {
            fprintf(stderr, "[vais-gpu] Metal error: Failed to load library '%s': %s\n",
                    metallib_path,
                    error ? [[error localizedDescription] UTF8String] : "unknown error");
            return -1;
        }
        return 0;
    }
}

// Initialize Metal from source string (runtime compilation).
// Returns 0 on success, -1 on failure.
long metal_init_source(const char* source) {
    @autoreleasepool {
        if (!g_device) {
            if (metal_init() != 0) return -1;
        }

        NSError* error = nil;
        NSString* src = [NSString stringWithUTF8String:source];
        MTLCompileOptions* options = [[MTLCompileOptions alloc] init];
        g_library = [g_device newLibraryWithSource:src options:options error:&error];
        if (!g_library) {
            fprintf(stderr, "[vais-gpu] Metal error: Failed to compile shader source: %s\n",
                    error ? [[error localizedDescription] UTF8String] : "unknown error");
            return -1;
        }
        return 0;
    }
}

// ============================================
// Memory management API
// ============================================

// Allocate a Metal buffer on the GPU.
// Returns buffer handle (id<MTLBuffer> cast to void*), or NULL on failure.
void* metal_alloc(long size) {
    @autoreleasepool {
        if (!g_device) {
            fprintf(stderr, "[vais-gpu] Metal error: Device not initialized. Call metal_init() first.\n");
            return NULL;
        }
        id<MTLBuffer> buffer = [g_device newBufferWithLength:(NSUInteger)size
                                                     options:MTLResourceStorageModeShared];
        if (!buffer) {
            fprintf(stderr, "[vais-gpu] Metal error: Failed to allocate buffer of size %ld\n", size);
            return NULL;
        }
        return (__bridge_retained void*)buffer;
    }
}

// Free a Metal buffer.
long metal_free(void* buffer_handle) {
    if (!buffer_handle) return 0;
    @autoreleasepool {
        id<MTLBuffer> buffer = (__bridge_transfer id<MTLBuffer>)buffer_handle;
        buffer = nil;  // ARC releases
        return 0;
    }
}

// Copy data from host to Metal buffer.
long metal_memcpy_h2d(void* buffer_handle, const void* src, long size) {
    @autoreleasepool {
        if (!buffer_handle || !src) return -1;
        id<MTLBuffer> buffer = (__bridge id<MTLBuffer>)buffer_handle;
        if ((NSUInteger)size > buffer.length) {
            fprintf(stderr, "[vais-gpu] Metal error: Copy size %ld exceeds buffer size %lu\n",
                    size, (unsigned long)buffer.length);
            return -1;
        }
        memcpy(buffer.contents, src, (size_t)size);
        return 0;
    }
}

// Copy data from Metal buffer to host.
long metal_memcpy_d2h(void* dst, void* buffer_handle, long size) {
    @autoreleasepool {
        if (!dst || !buffer_handle) return -1;
        id<MTLBuffer> buffer = (__bridge id<MTLBuffer>)buffer_handle;
        if ((NSUInteger)size > buffer.length) {
            fprintf(stderr, "[vais-gpu] Metal error: Copy size %ld exceeds buffer size %lu\n",
                    size, (unsigned long)buffer.length);
            return -1;
        }
        memcpy(dst, buffer.contents, (size_t)size);
        return 0;
    }
}

// Get the raw contents pointer of a shared buffer (for direct access).
void* metal_buffer_contents(void* buffer_handle) {
    if (!buffer_handle) return NULL;
    id<MTLBuffer> buffer = (__bridge id<MTLBuffer>)buffer_handle;
    return buffer.contents;
}

// Get the length of a Metal buffer.
long metal_buffer_length(void* buffer_handle) {
    if (!buffer_handle) return 0;
    id<MTLBuffer> buffer = (__bridge id<MTLBuffer>)buffer_handle;
    return (long)buffer.length;
}

// ============================================
// Kernel (Compute Pipeline) execution
// ============================================

// Create a compute pipeline for a named kernel function.
// Returns pipeline handle (void*), or NULL on failure.
void* metal_create_pipeline(const char* kernel_name) {
    @autoreleasepool {
        if (!g_library || !kernel_name) {
            fprintf(stderr, "[vais-gpu] Metal error: Library not loaded or kernel name is NULL\n");
            return NULL;
        }

        NSString* name = [NSString stringWithUTF8String:kernel_name];
        id<MTLFunction> function = [g_library newFunctionWithName:name];
        if (!function) {
            fprintf(stderr, "[vais-gpu] Metal error: Kernel function '%s' not found in library\n",
                    kernel_name);
            return NULL;
        }

        NSError* error = nil;
        id<MTLComputePipelineState> pipeline =
            [g_device newComputePipelineStateWithFunction:function error:&error];
        if (!pipeline) {
            fprintf(stderr, "[vais-gpu] Metal error: Failed to create pipeline for '%s': %s\n",
                    kernel_name,
                    error ? [[error localizedDescription] UTF8String] : "unknown error");
            return NULL;
        }
        return (__bridge_retained void*)pipeline;
    }
}

// Destroy a compute pipeline.
long metal_destroy_pipeline(void* pipeline_handle) {
    if (!pipeline_handle) return 0;
    @autoreleasepool {
        id<MTLComputePipelineState> pipeline =
            (__bridge_transfer id<MTLComputePipelineState>)pipeline_handle;
        pipeline = nil;
        return 0;
    }
}

// Dispatch a compute kernel.
// pipeline_handle: from metal_create_pipeline()
// buffers: array of Metal buffer handles (void**)
// buffer_count: number of buffers
// grid_x/y/z: total threads
// block_x/y/z: threads per threadgroup
long metal_dispatch(
    void* pipeline_handle,
    void** buffers,
    long buffer_count,
    long grid_x, long grid_y, long grid_z,
    long block_x, long block_y, long block_z
) {
    @autoreleasepool {
        if (!pipeline_handle || !g_command_queue) {
            fprintf(stderr, "[vais-gpu] Metal error: Pipeline or command queue not initialized\n");
            return -1;
        }

        id<MTLComputePipelineState> pipeline =
            (__bridge id<MTLComputePipelineState>)pipeline_handle;

        id<MTLCommandBuffer> commandBuffer = [g_command_queue commandBuffer];
        if (!commandBuffer) {
            fprintf(stderr, "[vais-gpu] Metal error: Failed to create command buffer\n");
            return -1;
        }

        id<MTLComputeCommandEncoder> encoder = [commandBuffer computeCommandEncoder];
        if (!encoder) {
            fprintf(stderr, "[vais-gpu] Metal error: Failed to create compute encoder\n");
            return -1;
        }

        [encoder setComputePipelineState:pipeline];

        // Bind buffers
        for (long i = 0; i < buffer_count; i++) {
            if (buffers[i]) {
                id<MTLBuffer> buffer = (__bridge id<MTLBuffer>)buffers[i];
                [encoder setBuffer:buffer offset:0 atIndex:(NSUInteger)i];
            }
        }

        MTLSize gridSize = MTLSizeMake((NSUInteger)grid_x, (NSUInteger)grid_y, (NSUInteger)grid_z);
        MTLSize threadgroupSize = MTLSizeMake((NSUInteger)block_x, (NSUInteger)block_y, (NSUInteger)block_z);

        [encoder dispatchThreads:gridSize threadsPerThreadgroup:threadgroupSize];
        [encoder endEncoding];

        [commandBuffer commit];
        [commandBuffer waitUntilCompleted];

        if (commandBuffer.error) {
            fprintf(stderr, "[vais-gpu] Metal error: Compute failed: %s\n",
                    [[commandBuffer.error localizedDescription] UTF8String]);
            return -1;
        }

        return 0;
    }
}

// Dispatch with automatic threadgroup size calculation.
long metal_dispatch_auto(
    void* pipeline_handle,
    void** buffers,
    long buffer_count,
    long total_threads
) {
    @autoreleasepool {
        if (!pipeline_handle) return -1;

        id<MTLComputePipelineState> pipeline =
            (__bridge id<MTLComputePipelineState>)pipeline_handle;

        NSUInteger maxThreads = pipeline.maxTotalThreadsPerThreadgroup;
        NSUInteger threadgroupSize = maxThreads < (NSUInteger)total_threads
                                     ? maxThreads
                                     : (NSUInteger)total_threads;

        return metal_dispatch(
            pipeline_handle, buffers, buffer_count,
            total_threads, 1, 1,
            (long)threadgroupSize, 1, 1
        );
    }
}

// ============================================
// Device management API
// ============================================

// Get the number of Metal-capable devices (always 1 on most Macs).
long metal_device_count(void) {
#if TARGET_OS_OSX
    NSArray<id<MTLDevice>>* devices = MTLCopyAllDevices();
    long count = (long)devices.count;
    return count;
#else
    return g_device ? 1 : 0;
#endif
}

// Get the device name.
const char* metal_device_name(void) {
    static char name_buf[256];
    if (!g_device) return "not initialized";
    NSString* name = g_device.name;
    strncpy(name_buf, [name UTF8String], 255);
    name_buf[255] = '\0';
    return name_buf;
}

// Check if the device supports a feature set.
long metal_supports_family(long family) {
    if (!g_device) return 0;
    // MTLGPUFamily values: 1=Common1, 2=Common2, etc.
    return [g_device supportsFamily:(MTLGPUFamily)family] ? 1 : 0;
}

// Get recommended max working set size (bytes).
long metal_recommended_max_working_set(void) {
    if (!g_device) return 0;
#if TARGET_OS_OSX
    return (long)g_device.recommendedMaxWorkingSetSize;
#else
    return 0;
#endif
}

// Get max threadgroup memory length.
long metal_max_threadgroup_memory(void) {
    if (!g_device) return 0;
    return (long)g_device.maxThreadgroupMemoryLength;
}

// Get max threads per threadgroup.
long metal_max_threads_per_threadgroup(void) {
    if (!g_device) return 0;
    MTLSize maxSize = g_device.maxThreadsPerThreadgroup;
    return (long)maxSize.width;
}

// ============================================
// Event / Profiling
// ============================================

// Metal event wrapper struct for timing.
typedef struct {
    CFAbsoluteTime timestamp;
} metal_event_t;

// Create a profiling event. Returns handle or NULL on failure.
void* metal_event_create(void) {
    metal_event_t* event = (metal_event_t*)malloc(sizeof(metal_event_t));
    if (!event) return NULL;
    event->timestamp = 0.0;
    return (void*)event;
}

// Destroy a profiling event.
long metal_event_destroy(void* event_handle) {
    if (!event_handle) return 0;
    free(event_handle);
    return 0;
}

// Record current time into event.
long metal_event_record(void* event_handle) {
    if (!event_handle) return -1;
    metal_event_t* event = (metal_event_t*)event_handle;
    event->timestamp = CFAbsoluteTimeGetCurrent();
    return 0;
}

// Wait (no-op for CPU-side timestamps, included for API parity).
long metal_event_wait(void* event_handle) {
    if (!event_handle) return -1;
    return 0;
}

// Get elapsed time between two events in milliseconds.
double metal_event_elapsed(void* start_handle, void* end_handle) {
    if (!start_handle || !end_handle) return -1.0;
    metal_event_t* start = (metal_event_t*)start_handle;
    metal_event_t* end = (metal_event_t*)end_handle;
    return (end->timestamp - start->timestamp) * 1000.0;
}

// ============================================
// Async dispatch
// ============================================

// Dispatch a compute kernel asynchronously (does not wait for completion).
long metal_dispatch_async(
    void* pipeline_handle,
    void** buffers,
    long buffer_count,
    long grid_x, long grid_y, long grid_z,
    long block_x, long block_y, long block_z
) {
    @autoreleasepool {
        if (!pipeline_handle || !g_command_queue) {
            fprintf(stderr, "[vais-gpu] Metal error: Pipeline or command queue not initialized\n");
            return -1;
        }

        id<MTLComputePipelineState> pipeline =
            (__bridge id<MTLComputePipelineState>)pipeline_handle;

        id<MTLCommandBuffer> commandBuffer = [g_command_queue commandBuffer];
        if (!commandBuffer) {
            fprintf(stderr, "[vais-gpu] Metal error: Failed to create command buffer\n");
            return -1;
        }

        id<MTLComputeCommandEncoder> encoder = [commandBuffer computeCommandEncoder];
        if (!encoder) {
            fprintf(stderr, "[vais-gpu] Metal error: Failed to create compute encoder\n");
            return -1;
        }

        [encoder setComputePipelineState:pipeline];

        for (long i = 0; i < buffer_count; i++) {
            if (buffers[i]) {
                id<MTLBuffer> buffer = (__bridge id<MTLBuffer>)buffers[i];
                [encoder setBuffer:buffer offset:0 atIndex:(NSUInteger)i];
            }
        }

        MTLSize gridSize = MTLSizeMake((NSUInteger)grid_x, (NSUInteger)grid_y, (NSUInteger)grid_z);
        MTLSize threadgroupSize = MTLSizeMake((NSUInteger)block_x, (NSUInteger)block_y, (NSUInteger)block_z);

        [encoder dispatchThreads:gridSize threadsPerThreadgroup:threadgroupSize];
        [encoder endEncoding];

        // Commit but do NOT wait — async dispatch
        [commandBuffer commit];

        return 0;
    }
}

// ============================================
// Multi-GPU device selection
// ============================================

// Select a Metal device by index (supports eGPU).
// Returns 0 on success, -1 on failure.
long metal_device_select(long device_index) {
#if TARGET_OS_OSX
    @autoreleasepool {
        NSArray<id<MTLDevice>>* devices = MTLCopyAllDevices();
        if ((NSUInteger)device_index >= devices.count) {
            fprintf(stderr, "[vais-gpu] Metal error: Device index %ld out of range (count=%lu)\n",
                    device_index, (unsigned long)devices.count);
            return -1;
        }
        g_device = devices[(NSUInteger)device_index];
        // Recreate command queue for new device
        g_command_queue = [g_device newCommandQueue];
        if (!g_command_queue) {
            fprintf(stderr, "[vais-gpu] Metal error: Failed to create command queue for device %ld\n",
                    device_index);
            return -1;
        }
        // Library must be reloaded for new device
        g_library = nil;
        return 0;
    }
#else
    (void)device_index;
    fprintf(stderr, "[vais-gpu] Metal error: Multi-GPU not supported on this platform\n");
    return -1;
#endif
}

// ============================================
// Cleanup
// ============================================

// Release all Metal resources.
long metal_cleanup(void) {
    @autoreleasepool {
        g_library = nil;
        g_command_queue = nil;
        g_device = nil;
        return 0;
    }
}
