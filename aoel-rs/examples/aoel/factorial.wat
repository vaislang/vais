(module
  ;; Memory for heap allocation
  (memory (export "memory") 1)
  
  ;; Heap pointer
  (global $heap_ptr (mut i32) (i32.const 1024))
  
  ;; WASI imports for I/O
  (import "wasi_snapshot_preview1" "fd_write"
    (func $fd_write (param i32 i32 i32 i32) (result i32))
  )
  
  ;; Simple bump allocator
  (func $alloc (param $size i32) (result i32)
    (local $ptr i32)
    (local.set $ptr (global.get $heap_ptr))
    (global.set $heap_ptr (i32.add (global.get $heap_ptr) (local.get $size)))
    (local.get $ptr)
  )
  
  ;; Print integer to stdout
  (func $print_i64 (param $val i64)
    ;; Simple implementation - write to memory and call fd_write
    (local $ptr i32)
    (local $len i32)
    (local $tmp i64)
    (local $neg i32)
    
    ;; For now, just drop the value (full implementation would convert to string)
    (drop (local.get $val))
  )
  
  
  ;; Function: factorial
  (func $factorial (param $n i64) (result i64)
    ;; Stack simulation locals
    (local $tmp1 i64)
    (local $tmp2 i64)
    (local $cond i32)
    
    (local.get $n)
    (i64.const 1)
    (i64.le_s)
    (i64.extend_i32_s)
    ;; jump if false to instruction 6
    (i32.wrap_i64)
    (i32.eqz)
    (if
      (then
        ;; branch to 6
      )
    )
    (i64.const 1)
    ;; jump to instruction 12
    (local.get $n)
    (local.get $n)
    (i64.const 1)
    (i64.sub)
    ;; call factorial with 1 args
    (call $factorial)
    (i64.mul)
    (return)
    ;; default return
    (i64.const 0)
  )
  
  ;; Function: __main__
  (func $__main__ (result i64)
    ;; Stack simulation locals
    (local $tmp1 i64)
    (local $tmp2 i64)
    (local $cond i32)
    
    (i64.const 10)
    ;; call factorial with 1 args
    (call $factorial)
    ;; default return
    (i64.const 0)
  )
  
  (export "main" (func $__main__))
  
  (start $__main__)
)
