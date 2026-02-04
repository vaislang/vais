# GPU API Reference

> GPU compute support for CUDA, Metal, OpenCL with host-side runtime management

## Import

```vais
U std/gpu
```

## Overview

The GPU module provides two categories of functions:

1. **Kernel-side intrinsics**: Thread indexing, synchronization, atomics, math - replaced by GPU codegen when compiling with `--gpu`
2. **Host-side runtime API**: Memory allocation, kernel launching, device management - linked from `gpu_runtime.c`

Compile with: `vaisc build file.vais --gpu cuda --gpu-compile`

## Thread Indexing Functions

### Basic Thread/Block Indices

| Function | Signature | Description |
|----------|-----------|-------------|
| `thread_idx_x` | `F thread_idx_x() -> i64` | Thread index within block (x) |
| `thread_idx_y` | `F thread_idx_y() -> i64` | Thread index within block (y) |
| `thread_idx_z` | `F thread_idx_z() -> i64` | Thread index within block (z) |
| `block_idx_x` | `F block_idx_x() -> i64` | Block index within grid (x) |
| `block_idx_y` | `F block_idx_y() -> i64` | Block index within grid (y) |
| `block_idx_z` | `F block_idx_z() -> i64` | Block index within grid (z) |

### Dimensions

| Function | Signature | Description |
|----------|-----------|-------------|
| `block_dim_x` | `F block_dim_x() -> i64` | Threads per block (x) |
| `block_dim_y` | `F block_dim_y() -> i64` | Threads per block (y) |
| `block_dim_z` | `F block_dim_z() -> i64` | Threads per block (z) |
| `grid_dim_x` | `F grid_dim_x() -> i64` | Blocks per grid (x) |
| `grid_dim_y` | `F grid_dim_y() -> i64` | Blocks per grid (y) |
| `grid_dim_z` | `F grid_dim_z() -> i64` | Blocks per grid (z) |

### Global Indexing

| Function | Signature | Description |
|----------|-----------|-------------|
| `global_idx` | `F global_idx() -> i64` | Global thread index (1D) |
| `global_idx_x` | `F global_idx_x() -> i64` | Global thread index (2D x) |
| `global_idx_y` | `F global_idx_y() -> i64` | Global thread index (2D y) |

## Synchronization Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `sync_threads` | `F sync_threads() -> i64` | Block-level barrier (all threads) |
| `thread_fence` | `F thread_fence() -> i64` | Global memory fence |
| `thread_fence_block` | `F thread_fence_block() -> i64` | Shared memory fence |

## Atomic Operations

| Function | Signature | Description |
|----------|-----------|-------------|
| `atomic_add` | `F atomic_add(addr: *i64, val: i64) -> i64` | Atomic add, returns old value |
| `atomic_add_f64` | `F atomic_add_f64(addr: *f64, val: f64) -> f64` | Atomic add for f64 |
| `atomic_sub` | `F atomic_sub(addr: *i64, val: i64) -> i64` | Atomic subtract |
| `atomic_min` | `F atomic_min(addr: *i64, val: i64) -> i64` | Atomic minimum |
| `atomic_max` | `F atomic_max(addr: *i64, val: i64) -> i64` | Atomic maximum |
| `atomic_and` | `F atomic_and(addr: *i64, val: i64) -> i64` | Atomic bitwise AND |
| `atomic_or` | `F atomic_or(addr: *i64, val: i64) -> i64` | Atomic bitwise OR |
| `atomic_xor` | `F atomic_xor(addr: *i64, val: i64) -> i64` | Atomic bitwise XOR |
| `atomic_cas` | `F atomic_cas(addr: *i64, compare: i64, val: i64) -> i64` | Compare-and-swap |
| `atomic_exch` | `F atomic_exch(addr: *i64, val: i64) -> i64` | Atomic exchange |

## GPU Math Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `gpu_sqrt` | `F gpu_sqrt(x: f64) -> f64` | Fast square root |
| `gpu_rsqrt` | `F gpu_rsqrt(x: f64) -> f64` | Fast reciprocal square root |
| `gpu_sin` | `F gpu_sin(x: f64) -> f64` | Fast sine |
| `gpu_cos` | `F gpu_cos(x: f64) -> f64` | Fast cosine |
| `gpu_exp` | `F gpu_exp(x: f64) -> f64` | Fast exponential |
| `gpu_log` | `F gpu_log(x: f64) -> f64` | Fast logarithm |
| `gpu_fma` | `F gpu_fma(a: f64, b: f64, c: f64) -> f64` | Fused multiply-add: a*b+c |

## Shared Memory

| Function | Signature | Description |
|----------|-----------|-------------|
| `shared_alloc` | `F shared_alloc(size: i64) -> i64` | Allocate shared memory (per-block) |

## Memory Operations

| Function | Signature | Description |
|----------|-----------|-------------|
| `gpu_load` | `F gpu_load(addr: *f64) -> f64` | Coalesced load from global memory |
| `gpu_store` | `F gpu_store(addr: *f64, val: f64) -> i64` | Coalesced store to global memory |

## Utility Functions

| Function | Signature | Description |
|----------|-----------|-------------|
| `gpu_clamp` | `F gpu_clamp(x: f64, lo: f64, hi: f64) -> f64` | Clamp value to range |
| `gpu_lerp` | `F gpu_lerp(a: f64, b: f64, t: f64) -> f64` | Linear interpolation |
| `gpu_step` | `F gpu_step(edge: f64, x: f64) -> f64` | Step function |
| `gpu_smoothstep` | `F gpu_smoothstep(edge0: f64, edge1: f64, x: f64) -> f64` | Smooth Hermite interpolation |

## Warp/Wavefront Operations

| Function | Signature | Description |
|----------|-----------|-------------|
| `lane_id` | `F lane_id() -> i64` | Lane index within warp (0-31 or 0-63) |
| `warp_all` | `F warp_all(condition: i64) -> i64` | True if all lanes have condition true |
| `warp_any` | `F warp_any(condition: i64) -> i64` | True if any lane has condition true |
| `warp_ballot` | `F warp_ballot(condition: i64) -> i64` | Bitmask of lanes with condition true |
| `warp_shuffle` | `F warp_shuffle(val: i64, src_lane: i64) -> i64` | Get value from another lane |
| `warp_shuffle_down` | `F warp_shuffle_down(val: i64, delta: i64) -> i64` | Get value from lane + delta |
| `warp_shuffle_up` | `F warp_shuffle_up(val: i64, delta: i64) -> i64` | Get value from lane - delta |
| `warp_shuffle_xor` | `F warp_shuffle_xor(val: i64, mask: i64) -> i64` | Get value from lane ^ mask |

## Block Reduction Operations

| Function | Signature | Description |
|----------|-----------|-------------|
| `block_reduce_sum` | `F block_reduce_sum(val: f64) -> f64` | Block-level sum reduction |
| `block_reduce_max` | `F block_reduce_max(val: f64) -> f64` | Block-level max reduction |
| `block_reduce_min` | `F block_reduce_min(val: f64) -> f64` | Block-level min reduction |

## Grid Configuration Helpers

| Function | Signature | Description |
|----------|-----------|-------------|
| `calc_blocks` | `F calc_blocks(n: i64, block_size: i64) -> i64` | Calculate blocks needed for n elements |
| `calc_threads` | `F calc_threads(n: i64, block_size: i64) -> i64` | Calculate total threads for n elements |

## Struct

### KernelConfig

Configure kernel launch parameters.

**Fields:**
- `grid_x: i64` - Grid dimension x
- `grid_y: i64` - Grid dimension y
- `grid_z: i64` - Grid dimension z
- `block_x: i64` - Block dimension x
- `block_y: i64` - Block dimension y
- `block_z: i64` - Block dimension z
- `shared_memory: i64` - Shared memory bytes

| Function | Signature | Description |
|----------|-----------|-------------|
| `kernel_config_default` | `F kernel_config_default() -> KernelConfig` | Default config (1 block, 256 threads) |
| `kernel_config_1d` | `F kernel_config_1d(n: i64, block_size: i64) -> KernelConfig` | 1D kernel config |
| `kernel_config_2d` | `F kernel_config_2d(width: i64, height: i64, block_x: i64, block_y: i64) -> KernelConfig` | 2D kernel config |

## Host-Side API

### Memory Management

| Function | Signature | Description |
|----------|-----------|-------------|
| `gpu_alloc` | `F gpu_alloc(size: i64) -> *i64` | Allocate GPU device memory |
| `gpu_free` | `F gpu_free(ptr: *i64) -> i64` | Free GPU device memory |
| `gpu_memcpy_h2d` | `F gpu_memcpy_h2d(dst: *i64, src: *i64, size: i64) -> i64` | Copy host to device |
| `gpu_memcpy_d2h` | `F gpu_memcpy_d2h(dst: *i64, src: *i64, size: i64) -> i64` | Copy device to host |
| `gpu_memcpy_d2d` | `F gpu_memcpy_d2d(dst: *i64, src: *i64, size: i64) -> i64` | Copy device to device |
| `gpu_memset` | `F gpu_memset(ptr: *i64, value: i64, size: i64) -> i64` | Set device memory to value |
| `gpu_alloc_managed` | `F gpu_alloc_managed(size: i64) -> *i64` | Allocate unified/managed memory |

### Kernel Execution

| Function | Signature | Description |
|----------|-----------|-------------|
| `gpu_launch_kernel` | `F gpu_launch_kernel(kernel_func: *i64, grid_x: i64, grid_y: i64, grid_z: i64, block_x: i64, block_y: i64, block_z: i64, shared_mem: i64, args: *i64, arg_count: i64) -> i64` | Launch CUDA kernel |
| `gpu_synchronize` | `F gpu_synchronize() -> i64` | Wait for all GPU operations |

### Stream Management

| Function | Signature | Description |
|----------|-----------|-------------|
| `gpu_stream_create` | `F gpu_stream_create() -> *i64` | Create CUDA stream |
| `gpu_stream_destroy` | `F gpu_stream_destroy(stream: *i64) -> i64` | Destroy stream |
| `gpu_stream_synchronize` | `F gpu_stream_synchronize(stream: *i64) -> i64` | Synchronize stream |

### Device Management

| Function | Signature | Description |
|----------|-----------|-------------|
| `gpu_device_count` | `F gpu_device_count() -> i64` | Get number of CUDA devices |
| `gpu_set_device` | `F gpu_set_device(device_id: i64) -> i64` | Set active device |
| `gpu_get_device` | `F gpu_get_device() -> i64` | Get current device ID |
| `gpu_device_name` | `F gpu_device_name(device_id: i64) -> *i8` | Get device name |
| `gpu_device_total_mem` | `F gpu_device_total_mem(device_id: i64) -> i64` | Get total device memory |
| `gpu_device_max_threads` | `F gpu_device_max_threads(device_id: i64) -> i64` | Get max threads per block |

### Event Timing

| Function | Signature | Description |
|----------|-----------|-------------|
| `gpu_event_create` | `F gpu_event_create() -> *i64` | Create CUDA event |
| `gpu_event_destroy` | `F gpu_event_destroy(event: *i64) -> i64` | Destroy event |
| `gpu_event_record` | `F gpu_event_record(event: *i64) -> i64` | Record event |
| `gpu_event_synchronize` | `F gpu_event_synchronize(event: *i64) -> i64` | Wait for event |
| `gpu_event_elapsed` | `F gpu_event_elapsed(start: *i64, end: *i64) -> f64` | Get elapsed time (ms) between events |
| `gpu_event_record_stream` | `F gpu_event_record_stream(event: *i64, stream: *i64) -> i64` | Record event on stream |

### Async Memory Transfer

| Function | Signature | Description |
|----------|-----------|-------------|
| `gpu_memcpy_h2d_async` | `F gpu_memcpy_h2d_async(dst: *i64, src: *i64, size: i64, stream: *i64) -> i64` | Async host-to-device copy |
| `gpu_memcpy_d2h_async` | `F gpu_memcpy_d2h_async(dst: *i64, src: *i64, size: i64, stream: *i64) -> i64` | Async device-to-host copy |

### Unified Memory Hints

| Function | Signature | Description |
|----------|-----------|-------------|
| `gpu_mem_prefetch` | `F gpu_mem_prefetch(ptr: *i64, size: i64, device_id: i64) -> i64` | Prefetch unified memory to device |
| `gpu_mem_advise` | `F gpu_mem_advise(ptr: *i64, size: i64, advice: i64, device_id: i64) -> i64` | Advise memory access pattern |

### Multi-GPU Peer Access

| Function | Signature | Description |
|----------|-----------|-------------|
| `gpu_peer_access_enable` | `F gpu_peer_access_enable(peer_device: i64) -> i64` | Enable peer-to-peer access |
| `gpu_peer_access_disable` | `F gpu_peer_access_disable(peer_device: i64) -> i64` | Disable peer-to-peer access |
| `gpu_peer_can_access` | `F gpu_peer_can_access(device: i64, peer: i64) -> i64` | Check if peer access possible |
| `gpu_memcpy_peer` | `F gpu_memcpy_peer(dst: *i64, dst_device: i64, src: *i64, src_device: i64, size: i64) -> i64` | Copy between devices |

### Error Handling

| Function | Signature | Description |
|----------|-----------|-------------|
| `gpu_last_error` | `F gpu_last_error() -> i64` | Get last CUDA error code (0=success) |
| `gpu_last_error_string` | `F gpu_last_error_string() -> *i8` | Get last error as string |
| `gpu_reset_error` | `F gpu_reset_error() -> i64` | Reset/clear last error |

## Metal-Specific Functions (Apple GPU)

| Function | Signature | Description |
|----------|-----------|-------------|
| `threadgroup_barrier` | `F threadgroup_barrier() -> i64` | Threadgroup memory barrier |
| `device_barrier` | `F device_barrier() -> i64` | Device memory barrier |
| `simd_sum` | `F simd_sum(val: f64) -> f64` | SIMD group sum |
| `simd_min` | `F simd_min(val: f64) -> f64` | SIMD group minimum |
| `simd_max` | `F simd_max(val: f64) -> f64` | SIMD group maximum |
| `simd_broadcast` | `F simd_broadcast(val: f64, lane: i64) -> f64` | Broadcast from lane |
| `quad_sum` | `F quad_sum(val: f64) -> f64` | Quad (4-wide) sum |
| `quad_broadcast` | `F quad_broadcast(val: f64, lane: i64) -> f64` | Quad broadcast |

## AVX-512 SIMD Operations (Intel/AMD)

### Load/Store (512-bit vectors)

| Function | Signature | Description |
|----------|-----------|-------------|
| `avx512_load_f32` | `F avx512_load_f32(addr: *i64) -> i64` | Load 16 x f32 |
| `avx512_store_f32` | `F avx512_store_f32(addr: *i64, vec: i64) -> i64` | Store 16 x f32 |
| `avx512_load_f64` | `F avx512_load_f64(addr: *f64) -> i64` | Load 8 x f64 |
| `avx512_store_f64` | `F avx512_store_f64(addr: *f64, vec: i64) -> i64` | Store 8 x f64 |

### Arithmetic

| Function | Signature | Description |
|----------|-----------|-------------|
| `avx512_add_f32` | `F avx512_add_f32(a: i64, b: i64) -> i64` | Vector add |
| `avx512_sub_f32` | `F avx512_sub_f32(a: i64, b: i64) -> i64` | Vector subtract |
| `avx512_mul_f32` | `F avx512_mul_f32(a: i64, b: i64) -> i64` | Vector multiply |
| `avx512_div_f32` | `F avx512_div_f32(a: i64, b: i64) -> i64` | Vector divide |
| `avx512_fma_f32` | `F avx512_fma_f32(a: i64, b: i64, c: i64) -> i64` | Vector FMA |

### Reduction

| Function | Signature | Description |
|----------|-----------|-------------|
| `avx512_reduce_add_f32` | `F avx512_reduce_add_f32(vec: i64) -> f64` | Horizontal sum |
| `avx512_reduce_min_f32` | `F avx512_reduce_min_f32(vec: i64) -> f64` | Horizontal minimum |
| `avx512_reduce_max_f32` | `F avx512_reduce_max_f32(vec: i64) -> f64` | Horizontal maximum |

### Broadcast

| Function | Signature | Description |
|----------|-----------|-------------|
| `avx512_broadcast_f32` | `F avx512_broadcast_f32(val: f64) -> i64` | Broadcast f32 to vector |
| `avx512_broadcast_f64` | `F avx512_broadcast_f64(val: f64) -> i64` | Broadcast f64 to vector |

## AVX2 SIMD Operations (Intel/AMD)

### Load/Store (256-bit vectors)

| Function | Signature | Description |
|----------|-----------|-------------|
| `avx2_load_f32` | `F avx2_load_f32(addr: *i64) -> i64` | Load 8 x f32 |
| `avx2_store_f32` | `F avx2_store_f32(addr: *i64, vec: i64) -> i64` | Store 8 x f32 |
| `avx2_load_f64` | `F avx2_load_f64(addr: *f64) -> i64` | Load 4 x f64 |
| `avx2_store_f64` | `F avx2_store_f64(addr: *f64, vec: i64) -> i64` | Store 4 x f64 |

### Arithmetic

| Function | Signature | Description |
|----------|-----------|-------------|
| `avx2_add_f32` | `F avx2_add_f32(a: i64, b: i64) -> i64` | Vector add |
| `avx2_sub_f32` | `F avx2_sub_f32(a: i64, b: i64) -> i64` | Vector subtract |
| `avx2_mul_f32` | `F avx2_mul_f32(a: i64, b: i64) -> i64` | Vector multiply |
| `avx2_fma_f32` | `F avx2_fma_f32(a: i64, b: i64, c: i64) -> i64` | Vector FMA |

### Broadcast

| Function | Signature | Description |
|----------|-----------|-------------|
| `avx2_broadcast_f32` | `F avx2_broadcast_f32(val: f64) -> i64` | Broadcast f32 to vector |

## ARM NEON SIMD Operations

### Load/Store (128-bit vectors)

| Function | Signature | Description |
|----------|-----------|-------------|
| `neon_load_f32` | `F neon_load_f32(addr: *i64) -> i64` | Load 4 x f32 |
| `neon_store_f32` | `F neon_store_f32(addr: *i64, vec: i64) -> i64` | Store 4 x f32 |
| `neon_load_f64` | `F neon_load_f64(addr: *f64) -> i64` | Load 2 x f64 |
| `neon_store_f64` | `F neon_store_f64(addr: *f64, vec: i64) -> i64` | Store 2 x f64 |

### Arithmetic

| Function | Signature | Description |
|----------|-----------|-------------|
| `neon_add_f32` | `F neon_add_f32(a: i64, b: i64) -> i64` | Vector add |
| `neon_sub_f32` | `F neon_sub_f32(a: i64, b: i64) -> i64` | Vector subtract |
| `neon_mul_f32` | `F neon_mul_f32(a: i64, b: i64) -> i64` | Vector multiply |
| `neon_fma_f32` | `F neon_fma_f32(a: i64, b: i64, c: i64) -> i64` | Vector FMA |

### Reduction

| Function | Signature | Description |
|----------|-----------|-------------|
| `neon_reduce_add_f32` | `F neon_reduce_add_f32(vec: i64) -> f64` | Horizontal sum |
| `neon_reduce_min_f32` | `F neon_reduce_min_f32(vec: i64) -> f64` | Horizontal minimum |
| `neon_reduce_max_f32` | `F neon_reduce_max_f32(vec: i64) -> f64` | Horizontal maximum |

### Broadcast

| Function | Signature | Description |
|----------|-----------|-------------|
| `neon_dup_f32` | `F neon_dup_f32(val: f64) -> i64` | Duplicate f32 to vector |

## Usage

### Basic Vector Addition Kernel

```vais
U std/gpu

#[gpu]
F vector_add(a: *f64, b: *f64, c: *f64, n: i64) -> i64 {
    idx := global_idx()
    I idx < n {
        c[idx] = a[idx] + b[idx]
    }
    0
}

F main() -> i64 {
    n := 1000000
    size := n * 8  # 8 bytes per f64

    # Allocate device memory
    d_a := gpu_alloc(size)
    d_b := gpu_alloc(size)
    d_c := gpu_alloc(size)

    # Copy input data
    gpu_memcpy_h2d(d_a, host_a, size)
    gpu_memcpy_h2d(d_b, host_b, size)

    # Launch kernel
    block_size := 256
    grid_size := calc_blocks(n, block_size)
    gpu_launch_kernel(vector_add, grid_size, 1, 1, block_size, 1, 1, 0, [d_a, d_b, d_c, n], 4)

    # Copy results back
    gpu_memcpy_d2h(host_c, d_c, size)

    # Synchronize and cleanup
    gpu_synchronize()
    gpu_free(d_a)
    gpu_free(d_b)
    gpu_free(d_c)

    0
}
```

### Matrix Multiplication (2D Grid)

```vais
U std/gpu

#[gpu]
F matmul(A: *f64, B: *f64, C: *f64, N: i64) -> i64 {
    row := global_idx_y()
    col := global_idx_x()

    I row < N && col < N {
        sum := 0.0
        k := 0
        L k < N {
            sum = sum + A[row * N + k] * B[k * N + col]
            k = k + 1
        }
        C[row * N + col] = sum
    }
    0
}

F main() -> i64 {
    N := 1024
    config := kernel_config_2d(N, N, 16, 16)

    # Launch 2D kernel
    gpu_launch_kernel(matmul, config.grid_x, config.grid_y, 1,
                      config.block_x, config.block_y, 1,
                      0, [d_A, d_B, d_C, N], 4)

    gpu_synchronize()
    0
}
```

### Using Shared Memory

```vais
U std/gpu

#[gpu]
F reduce_sum(input: *f64, output: *f64, n: i64) -> i64 {
    tid := thread_idx_x()
    idx := global_idx()

    # Allocate shared memory
    shared := shared_alloc(256 * 8) as *f64

    # Load into shared memory
    I idx < n {
        shared[tid] = input[idx]
    } ! {
        shared[tid] = 0.0
    }

    sync_threads()

    # Reduction in shared memory
    stride := 128
    L stride > 0 {
        I tid < stride {
            shared[tid] = shared[tid] + shared[tid + stride]
        }
        sync_threads()
        stride = stride / 2
    }

    # Write result
    I tid == 0 {
        output[block_idx_x()] = shared[0]
    }

    0
}
```

### Atomic Operations

```vais
U std/gpu

#[gpu]
F histogram(data: *i64, bins: *i64, n: i64, num_bins: i64) -> i64 {
    idx := global_idx()
    I idx < n {
        bin := data[idx] % num_bins
        atomic_add(&bins[bin], 1)
    }
    0
}
```

### Warp-Level Reduction

```vais
U std/gpu

#[gpu]
F warp_reduce(input: *f64, output: *f64, n: i64) -> i64 {
    idx := global_idx()
    val := I idx < n { input[idx] } ! { 0.0 }

    # Warp-level shuffle reduction
    val = val + warp_shuffle_down(val, 16)
    val = val + warp_shuffle_down(val, 8)
    val = val + warp_shuffle_down(val, 4)
    val = val + warp_shuffle_down(val, 2)
    val = val + warp_shuffle_down(val, 1)

    # First lane writes result
    I lane_id() == 0 {
        output[block_idx_x() * 32 + thread_idx_x() / 32] = val
    }

    0
}
```

### Stream-Based Async Execution

```vais
U std/gpu

F main() -> i64 {
    # Create streams
    stream1 := gpu_stream_create()
    stream2 := gpu_stream_create()

    # Async copies and kernels
    gpu_memcpy_h2d_async(d_a1, h_a1, size, stream1)
    gpu_memcpy_h2d_async(d_a2, h_a2, size, stream2)

    # Launch on different streams
    gpu_launch_kernel_stream(kernel1, grid, block, stream1, args1)
    gpu_launch_kernel_stream(kernel2, grid, block, stream2, args2)

    # Async copy results
    gpu_memcpy_d2h_async(h_c1, d_c1, size, stream1)
    gpu_memcpy_d2h_async(h_c2, d_c2, size, stream2)

    # Synchronize streams
    gpu_stream_synchronize(stream1)
    gpu_stream_synchronize(stream2)

    # Cleanup
    gpu_stream_destroy(stream1)
    gpu_stream_destroy(stream2)

    0
}
```

### GPU Timing with Events

```vais
U std/gpu

F main() -> i64 {
    start := gpu_event_create()
    stop := gpu_event_create()

    gpu_event_record(start)

    # Launch kernel
    gpu_launch_kernel(my_kernel, grid, block, 0, args, arg_count)

    gpu_event_record(stop)
    gpu_event_synchronize(stop)

    elapsed_ms := gpu_event_elapsed(start, stop)

    gpu_event_destroy(start)
    gpu_event_destroy(stop)

    0
}
```

## Notes

- **Kernel-side functions** (thread_idx_*, atomic_*, etc.) are replaced by the GPU codegen backend. Host-side placeholders return dummy values.
- **Host-side functions** (gpu_alloc, gpu_launch_kernel, etc.) are extern C functions linked from `gpu_runtime.c`.
- Compile with `--gpu cuda` for NVIDIA, `--gpu metal` for Apple, or `--gpu opencl` for cross-platform.
- Memory pointers returned by `gpu_alloc` are device pointers and cannot be dereferenced on the host.
- Always call `gpu_synchronize()` before reading results back to the host.
- SIMD functions (AVX-512, AVX2, NEON) are CPU-side optimizations, not GPU kernels.
