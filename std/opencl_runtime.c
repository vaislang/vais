// OpenCL GPU runtime support for VAIS - Cross-platform backend
// Provides OpenCL API wrappers for GPU memory management,
// kernel execution, and device management.
//
// Linked automatically by vaisc when std/gpu is imported with --gpu opencl --gpu-compile.
// Requires: OpenCL SDK (libOpenCL)
//   - macOS: Built-in (OpenCL.framework)
//   - Linux: Install via GPU vendor SDK or ocl-icd-opencl-dev
//   - Windows: Install via GPU vendor SDK

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#ifdef __APPLE__
#include <OpenCL/opencl.h>
#else
#include <CL/cl.h>
#endif

// ============================================
// Global OpenCL state
// ============================================

static cl_platform_id   g_platform   = NULL;
static cl_device_id     g_device     = NULL;
static cl_context        g_context    = NULL;
static cl_command_queue  g_queue      = NULL;
static cl_program        g_program    = NULL;
static cl_int            g_last_error = CL_SUCCESS;

// ============================================
// Error handling helpers
// ============================================

static int ocl_check_error(cl_int err, const char* operation) {
    g_last_error = err;
    if (err != CL_SUCCESS) {
        fprintf(stderr, "[vais-gpu] OpenCL error in %s: %d\n", operation, err);
        return -1;
    }
    return 0;
}

const char* opencl_error_string(long error_code) {
    switch ((cl_int)error_code) {
        case CL_SUCCESS:                         return "CL_SUCCESS";
        case CL_DEVICE_NOT_FOUND:                return "CL_DEVICE_NOT_FOUND";
        case CL_DEVICE_NOT_AVAILABLE:            return "CL_DEVICE_NOT_AVAILABLE";
        case CL_COMPILER_NOT_AVAILABLE:          return "CL_COMPILER_NOT_AVAILABLE";
        case CL_MEM_OBJECT_ALLOCATION_FAILURE:   return "CL_MEM_OBJECT_ALLOCATION_FAILURE";
        case CL_OUT_OF_RESOURCES:                return "CL_OUT_OF_RESOURCES";
        case CL_OUT_OF_HOST_MEMORY:              return "CL_OUT_OF_HOST_MEMORY";
        case CL_BUILD_PROGRAM_FAILURE:           return "CL_BUILD_PROGRAM_FAILURE";
        case CL_INVALID_VALUE:                   return "CL_INVALID_VALUE";
        case CL_INVALID_DEVICE:                  return "CL_INVALID_DEVICE";
        case CL_INVALID_CONTEXT:                 return "CL_INVALID_CONTEXT";
        case CL_INVALID_COMMAND_QUEUE:           return "CL_INVALID_COMMAND_QUEUE";
        case CL_INVALID_MEM_OBJECT:              return "CL_INVALID_MEM_OBJECT";
        case CL_INVALID_PROGRAM:                 return "CL_INVALID_PROGRAM";
        case CL_INVALID_KERNEL:                  return "CL_INVALID_KERNEL";
        case CL_INVALID_KERNEL_NAME:             return "CL_INVALID_KERNEL_NAME";
        case CL_INVALID_ARG_INDEX:               return "CL_INVALID_ARG_INDEX";
        case CL_INVALID_ARG_VALUE:               return "CL_INVALID_ARG_VALUE";
        case CL_INVALID_ARG_SIZE:                return "CL_INVALID_ARG_SIZE";
        case CL_INVALID_WORK_DIMENSION:          return "CL_INVALID_WORK_DIMENSION";
        case CL_INVALID_WORK_GROUP_SIZE:         return "CL_INVALID_WORK_GROUP_SIZE";
        case CL_INVALID_WORK_ITEM_SIZE:          return "CL_INVALID_WORK_ITEM_SIZE";
        case CL_INVALID_GLOBAL_WORK_SIZE:        return "CL_INVALID_GLOBAL_WORK_SIZE";
        default:                                 return "CL_UNKNOWN_ERROR";
    }
}

long opencl_last_error(void) {
    return (long)g_last_error;
}

// ============================================
// Platform & device discovery
// ============================================

static int discover_platform_and_device(void) {
    cl_int err;
    cl_uint num_platforms = 0;

    // Get number of platforms
    err = clGetPlatformIDs(0, NULL, &num_platforms);
    if (err != CL_SUCCESS || num_platforms == 0) {
        fprintf(stderr, "[vais-gpu] OpenCL error: No OpenCL platforms found\n");
        g_last_error = err;
        return -1;
    }

    // Get first available platform
    cl_platform_id* platforms = (cl_platform_id*)malloc(sizeof(cl_platform_id) * num_platforms);
    if (!platforms) return -1;

    err = clGetPlatformIDs(num_platforms, platforms, NULL);
    if (err != CL_SUCCESS) {
        free(platforms);
        g_last_error = err;
        return -1;
    }

    // Try to find a GPU device on any platform
    for (cl_uint i = 0; i < num_platforms; i++) {
        cl_uint num_devices = 0;
        err = clGetDeviceIDs(platforms[i], CL_DEVICE_TYPE_GPU, 0, NULL, &num_devices);
        if (err == CL_SUCCESS && num_devices > 0) {
            g_platform = platforms[i];
            err = clGetDeviceIDs(g_platform, CL_DEVICE_TYPE_GPU, 1, &g_device, NULL);
            if (err == CL_SUCCESS) {
                free(platforms);
                return 0;
            }
        }
    }

    // Fallback: try CPU device
    for (cl_uint i = 0; i < num_platforms; i++) {
        cl_uint num_devices = 0;
        err = clGetDeviceIDs(platforms[i], CL_DEVICE_TYPE_CPU, 0, NULL, &num_devices);
        if (err == CL_SUCCESS && num_devices > 0) {
            g_platform = platforms[i];
            err = clGetDeviceIDs(g_platform, CL_DEVICE_TYPE_CPU, 1, &g_device, NULL);
            if (err == CL_SUCCESS) {
                fprintf(stderr, "[vais-gpu] Warning: No GPU found, using CPU OpenCL device\n");
                free(platforms);
                return 0;
            }
        }
    }

    free(platforms);
    fprintf(stderr, "[vais-gpu] OpenCL error: No suitable OpenCL device found\n");
    return -1;
}

// ============================================
// Initialization
// ============================================

// Initialize OpenCL runtime with auto-detected platform and device.
// Returns 0 on success, -1 on failure.
long opencl_init(void) {
    cl_int err;

    // Discover platform and device
    if (discover_platform_and_device() != 0) {
        return -1;
    }

    // Create context
    g_context = clCreateContext(NULL, 1, &g_device, NULL, NULL, &err);
    if (ocl_check_error(err, "clCreateContext") != 0) {
        return -1;
    }

    // Create command queue
#ifdef CL_VERSION_2_0
    g_queue = clCreateCommandQueueWithProperties(g_context, g_device, NULL, &err);
#else
    g_queue = clCreateCommandQueue(g_context, g_device, 0, &err);
#endif
    if (ocl_check_error(err, "clCreateCommandQueue") != 0) {
        clReleaseContext(g_context);
        g_context = NULL;
        return -1;
    }

    return 0;
}

// Initialize OpenCL and build a program from kernel source string.
// Returns 0 on success, -1 on failure.
long opencl_init_source(const char* kernel_source) {
    if (!g_context) {
        if (opencl_init() != 0) return -1;
    }

    cl_int err;
    size_t src_len = strlen(kernel_source);

    g_program = clCreateProgramWithSource(g_context, 1, &kernel_source, &src_len, &err);
    if (ocl_check_error(err, "clCreateProgramWithSource") != 0) {
        return -1;
    }

    // Build program
    err = clBuildProgram(g_program, 1, &g_device, "-cl-std=CL1.2", NULL, NULL);
    if (err != CL_SUCCESS) {
        // Get build log
        size_t log_size = 0;
        clGetProgramBuildInfo(g_program, g_device, CL_PROGRAM_BUILD_LOG, 0, NULL, &log_size);
        if (log_size > 1) {
            char* log = (char*)malloc(log_size);
            if (log) {
                clGetProgramBuildInfo(g_program, g_device, CL_PROGRAM_BUILD_LOG, log_size, log, NULL);
                fprintf(stderr, "[vais-gpu] OpenCL build log:\n%s\n", log);
                free(log);
            }
        }
        g_last_error = err;
        clReleaseProgram(g_program);
        g_program = NULL;
        return -1;
    }

    return 0;
}

// Initialize OpenCL and build a program from a .cl file.
// Returns 0 on success, -1 on failure.
long opencl_init_file(const char* cl_file_path) {
    FILE* fp = fopen(cl_file_path, "r");
    if (!fp) {
        fprintf(stderr, "[vais-gpu] OpenCL error: Failed to open file '%s'\n", cl_file_path);
        return -1;
    }

    fseek(fp, 0, SEEK_END);
    long file_size = ftell(fp);
    fseek(fp, 0, SEEK_SET);

    char* source = (char*)malloc((size_t)file_size + 1);
    if (!source) {
        fclose(fp);
        return -1;
    }

    size_t read_size = fread(source, 1, (size_t)file_size, fp);
    source[read_size] = '\0';
    fclose(fp);

    long result = opencl_init_source(source);
    free(source);
    return result;
}

// Release all OpenCL resources.
long opencl_cleanup(void) {
    if (g_program) { clReleaseProgram(g_program); g_program = NULL; }
    if (g_queue)   { clReleaseCommandQueue(g_queue); g_queue = NULL; }
    if (g_context) { clReleaseContext(g_context); g_context = NULL; }
    g_platform = NULL;
    g_device = NULL;
    g_last_error = CL_SUCCESS;
    return 0;
}

// ============================================
// Memory management API
// ============================================

// Allocate a read/write GPU buffer.
// Returns cl_mem handle (cast to void*), or NULL on failure.
void* opencl_alloc(long size) {
    if (!g_context) {
        fprintf(stderr, "[vais-gpu] OpenCL error: Context not initialized. Call opencl_init() first.\n");
        return NULL;
    }
    cl_int err;
    cl_mem buffer = clCreateBuffer(g_context, CL_MEM_READ_WRITE, (size_t)size, NULL, &err);
    if (ocl_check_error(err, "opencl_alloc") != 0) {
        return NULL;
    }
    return (void*)buffer;
}

// Allocate a read-only GPU buffer.
void* opencl_alloc_read(long size) {
    if (!g_context) return NULL;
    cl_int err;
    cl_mem buffer = clCreateBuffer(g_context, CL_MEM_READ_ONLY, (size_t)size, NULL, &err);
    if (ocl_check_error(err, "opencl_alloc_read") != 0) return NULL;
    return (void*)buffer;
}

// Allocate a write-only GPU buffer.
void* opencl_alloc_write(long size) {
    if (!g_context) return NULL;
    cl_int err;
    cl_mem buffer = clCreateBuffer(g_context, CL_MEM_WRITE_ONLY, (size_t)size, NULL, &err);
    if (ocl_check_error(err, "opencl_alloc_write") != 0) return NULL;
    return (void*)buffer;
}

// Free a GPU buffer.
long opencl_free(void* buffer_handle) {
    if (!buffer_handle) return 0;
    cl_int err = clReleaseMemObject((cl_mem)buffer_handle);
    return ocl_check_error(err, "opencl_free");
}

// Copy data from host to device buffer (blocking).
long opencl_memcpy_h2d(void* buffer_handle, const void* src, long size) {
    if (!buffer_handle || !src || !g_queue) return -1;
    cl_int err = clEnqueueWriteBuffer(
        g_queue, (cl_mem)buffer_handle, CL_TRUE,
        0, (size_t)size, src, 0, NULL, NULL
    );
    return ocl_check_error(err, "opencl_memcpy_h2d");
}

// Copy data from device buffer to host (blocking).
long opencl_memcpy_d2h(void* dst, void* buffer_handle, long size) {
    if (!dst || !buffer_handle || !g_queue) return -1;
    cl_int err = clEnqueueReadBuffer(
        g_queue, (cl_mem)buffer_handle, CL_TRUE,
        0, (size_t)size, dst, 0, NULL, NULL
    );
    return ocl_check_error(err, "opencl_memcpy_d2h");
}

// ============================================
// Kernel execution API
// ============================================

// Create a kernel from the built program.
// Returns kernel handle (void*), or NULL on failure.
void* opencl_create_kernel(const char* kernel_name) {
    if (!g_program || !kernel_name) {
        fprintf(stderr, "[vais-gpu] OpenCL error: Program not built or kernel name is NULL\n");
        return NULL;
    }
    cl_int err;
    cl_kernel kernel = clCreateKernel(g_program, kernel_name, &err);
    if (ocl_check_error(err, "opencl_create_kernel") != 0) {
        return NULL;
    }
    return (void*)kernel;
}

// Destroy a kernel.
long opencl_destroy_kernel(void* kernel_handle) {
    if (!kernel_handle) return 0;
    cl_int err = clReleaseKernel((cl_kernel)kernel_handle);
    return ocl_check_error(err, "opencl_destroy_kernel");
}

// Set a scalar kernel argument.
long opencl_set_arg(void* kernel_handle, long index, long size, const void* value) {
    if (!kernel_handle) return -1;
    cl_int err = clSetKernelArg(
        (cl_kernel)kernel_handle, (cl_uint)index, (size_t)size, value
    );
    return ocl_check_error(err, "opencl_set_arg");
}

// Set a buffer kernel argument.
long opencl_set_arg_buffer(void* kernel_handle, long index, void* buffer_handle) {
    if (!kernel_handle) return -1;
    cl_mem mem = (cl_mem)buffer_handle;
    cl_int err = clSetKernelArg(
        (cl_kernel)kernel_handle, (cl_uint)index, sizeof(cl_mem), &mem
    );
    return ocl_check_error(err, "opencl_set_arg_buffer");
}

// Dispatch a kernel with explicit global and local work sizes.
// global_x/y/z: total work items in each dimension (0 = unused dimension)
// local_x/y/z: work group size in each dimension (0 = let OpenCL decide)
long opencl_dispatch(
    void* kernel_handle,
    long global_x, long global_y, long global_z,
    long local_x, long local_y, long local_z
) {
    if (!kernel_handle || !g_queue) return -1;

    // Determine work dimensions
    cl_uint work_dim = 1;
    if (global_z > 1) work_dim = 3;
    else if (global_y > 1) work_dim = 2;

    size_t global_work_size[3] = {
        (size_t)global_x,
        (size_t)(global_y > 0 ? global_y : 1),
        (size_t)(global_z > 0 ? global_z : 1)
    };

    size_t local_work_size[3] = {
        (size_t)local_x,
        (size_t)(local_y > 0 ? local_y : 1),
        (size_t)(local_z > 0 ? local_z : 1)
    };

    // If local size is 0, let OpenCL decide
    size_t* local_ptr = (local_x > 0) ? local_work_size : NULL;

    cl_int err = clEnqueueNDRangeKernel(
        g_queue, (cl_kernel)kernel_handle,
        work_dim, NULL, global_work_size, local_ptr,
        0, NULL, NULL
    );
    return ocl_check_error(err, "opencl_dispatch");
}

// Dispatch a 1D kernel with automatic work group size.
long opencl_dispatch_auto(void* kernel_handle, long total_work_items) {
    return opencl_dispatch(kernel_handle, total_work_items, 0, 0, 0, 0, 0);
}

// Wait for all queued operations to complete.
long opencl_synchronize(void) {
    if (!g_queue) return -1;
    cl_int err = clFinish(g_queue);
    return ocl_check_error(err, "opencl_synchronize");
}

// ============================================
// Device management API
// ============================================

// Get the number of available OpenCL platforms.
long opencl_platform_count(void) {
    cl_uint count = 0;
    cl_int err = clGetPlatformIDs(0, NULL, &count);
    if (err != CL_SUCCESS) return 0;
    return (long)count;
}

// Get the number of GPU devices on the current platform.
long opencl_device_count(void) {
    if (!g_platform) return 0;
    cl_uint count = 0;
    cl_int err = clGetDeviceIDs(g_platform, CL_DEVICE_TYPE_GPU, 0, NULL, &count);
    if (err != CL_SUCCESS) return 0;
    return (long)count;
}

// Get the current platform name.
const char* opencl_platform_name(void) {
    static char name_buf[256];
    if (!g_platform) return "not initialized";
    cl_int err = clGetPlatformInfo(g_platform, CL_PLATFORM_NAME, 256, name_buf, NULL);
    if (err != CL_SUCCESS) return "unknown";
    return name_buf;
}

// Get the current device name.
const char* opencl_device_name(void) {
    static char name_buf[256];
    if (!g_device) return "not initialized";
    cl_int err = clGetDeviceInfo(g_device, CL_DEVICE_NAME, 256, name_buf, NULL);
    if (err != CL_SUCCESS) return "unknown";
    return name_buf;
}

// Get the current device vendor.
const char* opencl_device_vendor(void) {
    static char vendor_buf[256];
    if (!g_device) return "not initialized";
    cl_int err = clGetDeviceInfo(g_device, CL_DEVICE_VENDOR, 256, vendor_buf, NULL);
    if (err != CL_SUCCESS) return "unknown";
    return vendor_buf;
}

// Get the number of compute units on the device.
long opencl_device_max_compute_units(void) {
    if (!g_device) return 0;
    cl_uint units = 0;
    clGetDeviceInfo(g_device, CL_DEVICE_MAX_COMPUTE_UNITS, sizeof(units), &units, NULL);
    return (long)units;
}

// Get the maximum work group size.
long opencl_device_max_work_group_size(void) {
    if (!g_device) return 0;
    size_t size = 0;
    clGetDeviceInfo(g_device, CL_DEVICE_MAX_WORK_GROUP_SIZE, sizeof(size), &size, NULL);
    return (long)size;
}

// Get total global memory in bytes.
long opencl_device_global_mem(void) {
    if (!g_device) return 0;
    cl_ulong mem = 0;
    clGetDeviceInfo(g_device, CL_DEVICE_GLOBAL_MEM_SIZE, sizeof(mem), &mem, NULL);
    return (long)mem;
}

// Get local memory size in bytes.
long opencl_device_local_mem(void) {
    if (!g_device) return 0;
    cl_ulong mem = 0;
    clGetDeviceInfo(g_device, CL_DEVICE_LOCAL_MEM_SIZE, sizeof(mem), &mem, NULL);
    return (long)mem;
}
