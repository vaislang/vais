# Vais GPU Code Generation

Vais supports generating GPU compute shader code for CUDA, OpenCL, and WebGPU targets.

## Quick Start

```bash
# Compile to CUDA
vaisc build kernel.vais --gpu cuda

# Compile to OpenCL
vaisc build kernel.vais --gpu opencl

# Compile to WebGPU WGSL
vaisc build kernel.vais --gpu webgpu
```

## Writing GPU Kernels

Use the `#[gpu]` or `#[kernel]` attribute to mark functions as GPU kernels:

```vais
U std/gpu

# GPU kernel for vector addition
#[gpu]
F vector_add(a: *f64, b: *f64, c: *f64, n: i64) -> i64 {
    idx := global_idx()
    I idx < n {
        c[idx] = a[idx] + b[idx]
    }
    0
}
```

## Supported Targets

### CUDA (.cu)

For NVIDIA GPUs. Generates CUDA C code that can be compiled with `nvcc`.

```bash
vaisc build kernel.vais --gpu cuda -o kernel.cu
nvcc -c kernel.cu -o kernel.o
```

### OpenCL (.cl)

Cross-platform GPU code. Works with NVIDIA, AMD, and Intel GPUs.

```bash
vaisc build kernel.vais --gpu opencl -o kernel.cl
```

### WebGPU WGSL (.wgsl)

For browser-based GPU computing. Generates WGSL shaders.

```bash
vaisc build kernel.vais --gpu webgpu -o kernel.wgsl
```

## GPU Built-in Functions

### Thread Indexing

| Function | Description |
|----------|-------------|
| `thread_idx_x()` | Thread index within block (x) |
| `thread_idx_y()` | Thread index within block (y) |
| `thread_idx_z()` | Thread index within block (z) |
| `block_idx_x()` | Block index within grid (x) |
| `block_idx_y()` | Block index within grid (y) |
| `block_idx_z()` | Block index within grid (z) |
| `block_dim_x()` | Block dimension (x) |
| `global_idx()` | Global linear thread index |

### Synchronization

| Function | Description |
|----------|-------------|
| `sync_threads()` | Synchronize all threads in block |
| `thread_fence()` | Memory fence (global) |

### Atomic Operations

| Function | Description |
|----------|-------------|
| `atomic_add(addr, val)` | Atomic add |
| `atomic_sub(addr, val)` | Atomic subtract |
| `atomic_min(addr, val)` | Atomic minimum |
| `atomic_max(addr, val)` | Atomic maximum |
| `atomic_cas(addr, cmp, val)` | Compare-and-swap |

### Math Functions

Standard math functions (`sqrt`, `sin`, `cos`, `exp`, `log`, `pow`, etc.) are mapped to their GPU equivalents.

## Examples

### Vector Addition

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
```

Generated CUDA:
```cuda
__global__ void vector_add(double* a, double* b, double* c, long long n) {
    long long idx = threadIdx.x + blockIdx.x * blockDim.x;
    if (idx < n) {
        c[idx] = a[idx] + b[idx];
    }
}
```

### Matrix Multiplication

```vais
U std/gpu

#[gpu]
F matmul(A: *f64, B: *f64, C: *f64, N: i64) -> i64 {
    row := block_idx_y() * block_dim_y() + thread_idx_y()
    col := block_idx_x() * block_dim_x() + thread_idx_x()

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
```

### Reduction (Sum)

```vais
U std/gpu

#[gpu]
F reduce_sum(data: *f64, result: *f64, n: i64) -> i64 {
    # Shared memory for partial sums (per block)
    shared := shared_alloc(256 * 8)

    tid := thread_idx_x()
    idx := global_idx()

    # Load data
    I idx < n {
        shared[tid] = data[idx]
    } E {
        shared[tid] = 0.0
    }

    sync_threads()

    # Reduction in shared memory
    s := 128
    L s > 0 {
        I tid < s {
            shared[tid] = shared[tid] + shared[tid + s]
        }
        sync_threads()
        s = s / 2
    }

    # Write result
    I tid == 0 {
        atomic_add(result, shared[0])
    }
    0
}
```

## Limitations

1. **No closures**: GPU kernels cannot use closures or captures
2. **Limited types**: Only primitive types and pointers supported
3. **No recursion**: GPU kernels cannot be recursive
4. **No dynamic allocation**: Use `shared_alloc()` for shared memory
5. **Fixed function signatures**: Kernel parameters must be pointers or primitives

## Type Mapping

| Vais Type | CUDA | OpenCL | WGSL |
|-----------|------|--------|------|
| `i64` | `long long` | `long` | `i32`* |
| `i32` | `int` | `int` | `i32` |
| `f64` | `double` | `double` | `f32`* |
| `f32` | `float` | `float` | `f32` |
| `bool` | `bool` | `bool` | `bool` |
| `*T` | `T*` | `__global T*` | `ptr<storage, T>` |

*WebGPU has limited 64-bit support

## Performance Tips

1. **Coalesced memory access**: Access memory in stride-1 pattern
2. **Shared memory**: Use `shared_alloc()` for frequently accessed data
3. **Avoid divergence**: Minimize branching within warps
4. **Occupancy**: Choose block sizes that maximize GPU utilization
5. **Memory transfers**: Minimize host-device data transfers

## Future Enhancements

- [ ] Automatic memory transfer code generation
- [ ] Texture and surface memory support
- [ ] CUDA cooperative groups
- [ ] Multi-GPU support
- [ ] Performance profiling integration
