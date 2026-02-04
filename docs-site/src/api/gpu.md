# GPU API Reference

> GPU compute support for CUDA and Metal backends

## Import

```vais
U std/gpu
```

## Thread Indexing

| Function | Description |
|----------|-------------|
| `thread_idx_x()` | Thread index within block (x) |
| `thread_idx_y()` | Thread index within block (y) |
| `block_idx_x()` | Block index within grid (x) |
| `block_idx_y()` | Block index within grid (y) |
| `block_dim_x()` | Block dimension (x) |
| `block_dim_y()` | Block dimension (y) |
| `grid_dim_x()` | Grid dimension (x) |

## Host-Side API

| Function | Description |
|----------|-------------|
| `gpu_alloc(size)` | Allocate GPU memory |
| `gpu_free(ptr)` | Free GPU memory |
| `gpu_copy_to_device(dst, src, size)` | Copy host to device |
| `gpu_copy_to_host(dst, src, size)` | Copy device to host |
| `gpu_launch_kernel(fn, grid, block, args)` | Launch kernel |
| `gpu_synchronize()` | Wait for GPU completion |

## Usage

```vais
U std/gpu

#[gpu]
F vector_add(a: *f64, b: *f64, c: *f64, n: i64) -> i64 {
    idx := thread_idx_x() + block_idx_x() * block_dim_x()
    I idx < n {
        c[idx] = a[idx] + b[idx]
    }
    0
}
```
