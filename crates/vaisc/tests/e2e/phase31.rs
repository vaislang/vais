use super::helpers::*;

// ===== Map Literal Tests =====

#[test]
fn test_map_literal_basic() {
    let source = r#"
fn main() -> i64 {
    m := {1: 10, 2: 20, 3: 30}
    0
}
"#;
    assert_exit_code(source, 0);
}

#[test]
fn test_map_literal_single_entry() {
    let source = r#"
fn main() -> i64 {
    m := {42: 100}
    0
}
"#;
    assert_exit_code(source, 0);
}

#[test]
fn test_map_literal_trailing_comma() {
    let source = r#"
fn main() -> i64 {
    m := {1: 10, 2: 20,}
    0
}
"#;
    assert_exit_code(source, 0);
}

#[test]
fn test_map_literal_with_expressions() {
    let source = r#"
fn main() -> i64 {
    a := 5
    m := {a: a * 2, 10: 20 + 30}
    0
}
"#;
    assert_exit_code(source, 0);
}

// ==================== Tuple Destructuring ====================

#[test]
fn e2e_tuple_destructure_simple() {
    let source = r#"
fn main() -> i64 {
    (a, b) := (10, 20)
    return a + b
}
"#;
    assert_exit_code(source, 30);
}

#[test]
fn e2e_tuple_destructure_from_function() {
    let source = r#"
fn pair() -> (i64, i64) = (3, 7)
fn main() -> i64 {
    (x, y) := pair()
    return x + y
}
"#;
    assert_exit_code(source, 10);
}

#[test]
fn e2e_tuple_destructure_three_elements() {
    let source = r#"
fn main() -> i64 {
    (a, b, c) := (10, 20, 12)
    return a + b + c
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_tuple_destructure_with_arithmetic() {
    let source = r#"
fn main() -> i64 {
    (a, b) := (100, 58)
    return a - b
}
"#;
    assert_exit_code(source, 42);
}

#[test]
fn e2e_tuple_field_access_extracts_element() {
    let source = r#"
fn main() -> i64 {
    t := (10, 32)
    return t.0 + t.1
}
"#;
    assert_exit_code(source, 42);
}

// ==================== Phase 31: File System Durability ====================

#[test]
#[cfg(unix)]
fn e2e_fsync_write_and_sync() {
    // Test: write file, fsync via fileno, read back and verify
    let source = r#"
fn main() -> i64 {
    # Write a file
    fp := fopen("/tmp/vais_fsync_test.txt", "w")
    I fp == 0 { return 1 }
    fputs("hello fsync", fp)
    fflush(fp)
    fd := fileno(fp)
    I fd < 0 {
        fclose(fp)
        return 2
    }
    result := fsync(fd)
    fclose(fp)
    I result != 0 { return 3 }

    # Read back
    fp2 := fopen("/tmp/vais_fsync_test.txt", "r")
    I fp2 == 0 { return 4 }
    buf := malloc(64)
    fgets(buf, 64, fp2)
    fclose(fp2)

    # Verify content starts with 'h' (104)
    ch := load_byte(buf)
    free(buf)
    remove("/tmp/vais_fsync_test.txt")
    I ch == 104 { return 0 } else { return 5 }
}
"#;
    assert_exit_code(source, 0);
}

#[test]
#[cfg(unix)]
fn e2e_fileno_valid_stream() {
    // Test: fileno returns valid fd for an open file
    let source = r#"
fn main() -> i64 {
    fp := fopen("/tmp/vais_fileno_test.txt", "w")
    I fp == 0 { return 1 }
    fd := fileno(fp)
    fclose(fp)
    remove("/tmp/vais_fileno_test.txt")
    I fd >= 0 { return 0 } else { return 2 }
}
"#;
    assert_exit_code(source, 0);
}

#[test]
#[cfg(unix)]
fn e2e_file_sync_method() {
    // Test File.sync() method via the std/file.vais pattern
    // (simplified: directly test fsync + fflush combo)
    let source = r#"
fn main() -> i64 {
    fp := fopen("/tmp/vais_sync_method_test.txt", "w")
    I fp == 0 { return 1 }
    fputs("sync test data", fp)
    # Simulate File.sync(): fflush then fsync(fileno(fp))
    fflush(fp)
    fd := fileno(fp)
    result := fsync(fd)
    fclose(fp)
    remove("/tmp/vais_sync_method_test.txt")
    I result == 0 { return 0 } else { return 2 }
}
"#;
    assert_exit_code(source, 0);
}

#[test]
#[cfg(unix)]
fn e2e_dir_sync_tmp() {
    // Test: open directory fd, fsync it, close it
    let source = r#"
fn main() -> i64 {
    # O_RDONLY = 0
    fd := posix_open("/tmp", 0, 0)
    I fd < 0 { return 1 }
    result := fsync(fd)
    posix_close(fd)
    I result == 0 { return 0 } else { return 2 }
}
"#;
    assert_exit_code(source, 0);
}

#[test]
#[cfg(unix)]
fn e2e_mmap_read_file() {
    // Test: write a file, mmap it for reading, verify content via load_byte
    let source = r#"
fn main() -> i64 {
    # Write test file
    fp := fopen("/tmp/vais_mmap_test.txt", "w")
    I fp == 0 { return 1 }
    fputs("MMAP", fp)
    fclose(fp)

    # Open with POSIX open for fd
    fd := posix_open("/tmp/vais_mmap_test.txt", 0, 0)
    I fd < 0 { return 2 }

    # mmap: PROT_READ=1, MAP_PRIVATE=2
    addr := mmap(0, 4, 1, 2, fd, 0)
    I addr == 0 - 1 { posix_close(fd); return 3 }

    # Read first byte: 'M' = 77
    ch := load_byte(addr)
    munmap(addr, 4)
    posix_close(fd)
    remove("/tmp/vais_mmap_test.txt")
    I ch == 77 { return 0 } else { return 4 }
}
"#;
    assert_exit_code(source, 0);
}

#[test]
#[cfg(unix)]
#[cfg_attr(
    not(target_os = "macos"),
    ignore = "msync flags differ on Linux (MS_SYNC=16 on macOS, 4 on Linux)"
)]
fn e2e_mmap_write_and_msync() {
    // Test: mmap a file for read-write, modify, msync, read back
    let source = r#"
fn main() -> i64 {
    # Create file with initial content
    fp := fopen("/tmp/vais_mmap_rw_test.txt", "w")
    I fp == 0 { return 1 }
    fputs("AAAA", fp)
    fclose(fp)

    # Open for read-write: O_RDWR = 2
    fd := posix_open("/tmp/vais_mmap_rw_test.txt", 2, 0)
    I fd < 0 { return 2 }

    # mmap: PROT_READ|PROT_WRITE=3, MAP_SHARED=1
    addr := mmap(0, 4, 3, 1, fd, 0)
    I addr == 0 - 1 { posix_close(fd); return 3 }

    # Write 'Z' (90) at offset 0
    store_byte(addr, 90)

    # msync: MS_SYNC=16 (macOS)
    result := msync(addr, 4, 16)
    munmap(addr, 4)
    posix_close(fd)
    I result != 0 {
        remove("/tmp/vais_mmap_rw_test.txt")
        return 4
    }

    # Read back and verify
    fp2 := fopen("/tmp/vais_mmap_rw_test.txt", "r")
    I fp2 == 0 { return 5 }
    buf := malloc(8)
    fgets(buf, 8, fp2)
    fclose(fp2)
    ch := load_byte(buf)
    free(buf)
    remove("/tmp/vais_mmap_rw_test.txt")
    I ch == 90 { return 0 } else { return 6 }
}
"#;
    assert_exit_code(source, 0);
}

#[test]
#[cfg(unix)]
fn e2e_mmap_invalid_fd() {
    // Test: mmap with invalid fd returns MAP_FAILED (-1)
    let source = r#"
fn main() -> i64 {
    # mmap with invalid fd (-1) should fail
    # PROT_READ=1, MAP_PRIVATE=2
    addr := mmap(0, 4096, 1, 2, 0 - 1, 0)
    I addr == 0 - 1 { return 0 } else { return 1 }
}
"#;
    assert_exit_code(source, 0);
}

#[test]
#[cfg(unix)]
fn e2e_mmap_madvise() {
    // Test: mmap a file and call madvise with MADV_SEQUENTIAL
    let source = r#"
fn main() -> i64 {
    fp := fopen("/tmp/vais_madvise_test.txt", "w")
    I fp == 0 { return 1 }
    fputs("advise test data here!!", fp)
    fclose(fp)

    fd := posix_open("/tmp/vais_madvise_test.txt", 0, 0)
    I fd < 0 { return 2 }

    # PROT_READ=1, MAP_PRIVATE=2
    addr := mmap(0, 23, 1, 2, fd, 0)
    I addr == 0 - 1 { posix_close(fd); return 3 }

    # MADV_SEQUENTIAL=2
    result := madvise(addr, 23, 2)
    munmap(addr, 23)
    posix_close(fd)
    remove("/tmp/vais_madvise_test.txt")
    I result == 0 { return 0 } else { return 4 }
}
"#;
    assert_exit_code(source, 0);
}

#[test]
#[cfg(unix)]
fn e2e_flock_exclusive_lock() {
    // Test: open a file, acquire exclusive lock, unlock, close
    let source = r#"
fn main() -> i64 {
    # Create test file
    fp := fopen("/tmp/vais_flock_test.txt", "w")
    I fp == 0 { return 1 }
    fputs("lock test", fp)
    fclose(fp)

    # Open with POSIX open for fd (O_RDWR=2)
    fd := posix_open("/tmp/vais_flock_test.txt", 2, 0)
    I fd < 0 { return 2 }

    # LOCK_EX=2 (exclusive lock)
    result := flock(fd, 2)
    I result != 0 { posix_close(fd); return 3 }

    # LOCK_UN=8 (unlock)
    result2 := flock(fd, 8)
    posix_close(fd)
    remove("/tmp/vais_flock_test.txt")
    I result2 == 0 { return 0 } else { return 4 }
}
"#;
    assert_exit_code(source, 0);
}

#[test]
#[cfg(unix)]
fn e2e_flock_shared_lock() {
    // Test: acquire shared lock on a file
    let source = r#"
fn main() -> i64 {
    fp := fopen("/tmp/vais_flock_sh_test.txt", "w")
    I fp == 0 { return 1 }
    fputs("shared lock test", fp)
    fclose(fp)

    fd := posix_open("/tmp/vais_flock_sh_test.txt", 0, 0)
    I fd < 0 { return 2 }

    # LOCK_SH=1
    result := flock(fd, 1)
    I result != 0 { posix_close(fd); return 3 }

    # LOCK_UN=8
    flock(fd, 8)
    posix_close(fd)
    remove("/tmp/vais_flock_sh_test.txt")
    return 0
}
"#;
    assert_exit_code(source, 0);
}

#[test]
#[cfg(unix)]
fn e2e_flock_try_nonblocking() {
    // Test: try non-blocking exclusive lock (LOCK_EX | LOCK_NB)
    let source = r#"
fn main() -> i64 {
    fp := fopen("/tmp/vais_flock_nb_test.txt", "w")
    I fp == 0 { return 1 }
    fputs("nb lock test", fp)
    fclose(fp)

    fd := posix_open("/tmp/vais_flock_nb_test.txt", 2, 0)
    I fd < 0 { return 2 }

    # LOCK_EX=2 + LOCK_NB=4 = 6
    result := flock(fd, 6)
    I result != 0 { posix_close(fd); return 3 }

    # Unlock and close
    flock(fd, 8)
    posix_close(fd)
    remove("/tmp/vais_flock_nb_test.txt")
    return 0
}
"#;
    assert_exit_code(source, 0);
}

// ==================== Phase 31 Stage 4: Allocator Pointer-Based State Mutation ====================

#[test]
fn test_bump_allocator_state_mutation() {
    // Verify BumpAllocator.alloc() actually advances offset via pointer-based self
    let source = r#"
struct BumpAllocator {
    buffer: i64,
    capacity: i64,
    offset: i64,
    allocated: i64
}

impl BumpAllocator {
    fn new(capacity: i64) -> BumpAllocator {
        buffer := malloc(capacity)
        BumpAllocator { buffer: buffer, capacity: capacity, offset: 0, allocated: 0 }
    }

    fn alloc(&self, size: i64, align: i64) -> i64 {
        mask := align - 1
        aligned_offset := (self.offset + mask) & (~mask)
        new_offset := aligned_offset + size
        I new_offset > self.capacity { return 0 }
        ptr := self.buffer + aligned_offset
        self.offset = new_offset
        self.allocated = self.allocated + size
        ptr
    }

    fn remaining(&self) -> i64 = self.capacity - self.offset
    fn total_allocated(&self) -> i64 = self.allocated

    fn reset(&self) -> i64 {
        self.offset = 0
        self.allocated = 0
        0
    }

    fn drop(&self) -> i64 {
        free(self.buffer)
        0
    }
}

fn main() -> i64 {
    alloc := BumpAllocator.new(1024)
    ptr1 := alloc.alloc(64, 8)
    I ptr1 == 0 { return 1 }
    ptr2 := alloc.alloc(128, 8)
    I ptr2 == 0 { return 2 }
    I ptr2 <= ptr1 { return 3 }
    I ptr2 < ptr1 + 64 { return 4 }
    I alloc.total_allocated() != 192 { return 5 }
    I alloc.remaining() != 832 { return 6 }
    alloc.reset()
    I alloc.remaining() != 1024 { return 7 }
    ptr3 := alloc.alloc(64, 8)
    I ptr3 != ptr1 { return 8 }
    alloc.drop()
    0
}
"#;
    assert_exit_code(source, 0);
}

#[test]
fn test_pool_allocator_state_mutation() {
    // Verify pool allocator with free list correctly updates state via pointer-based self
    let source = r#"
struct Pool {
    buf: i64,
    head: i64,
    count: i64
}

impl Pool {
    fn new(n: i64) -> Pool {
        buf := malloc(n * 8)
        # Initialize 3-element free list manually for testing
        store_i64(buf, buf + 8)
        store_i64(buf + 8, buf + 16)
        store_i64(buf + 16, buf + 24)
        store_i64(buf + 24, buf + 32)
        store_i64(buf + 32, buf + 40)
        store_i64(buf + 40, buf + 48)
        store_i64(buf + 48, buf + 56)
        store_i64(buf + 56, buf + 64)
        store_i64(buf + 64, buf + 72)
        store_i64(buf + 72, 0)
        Pool { buf: buf, head: buf, count: n }
    }

    fn alloc(&self) -> i64 {
        I self.head == 0 { return 0 }
        block := self.head
        self.head = load_i64(block)
        self.count = self.count - 1
        block
    }

    fn dealloc(&self, ptr: i64) -> i64 {
        store_i64(ptr, self.head)
        self.head = ptr
        self.count = self.count + 1
        0
    }

    fn available(&self) -> i64 = self.count
    fn drop(&self) -> i64 { free(self.buf); 0 }
}

fn main() -> i64 {
    p := Pool.new(10)
    I p.available() != 10 { return 1 }
    a := p.alloc()
    I a == 0 { return 2 }
    b := p.alloc()
    I b == 0 { return 3 }
    I a == b { return 4 }
    I p.available() != 8 { return 5 }
    p.dealloc(a)
    I p.available() != 9 { return 6 }
    c := p.alloc()
    I c != a { return 7 }
    p.drop()
    0
}
"#;
    assert_exit_code(source, 0);
}

#[test]
fn test_freelist_allocator_state_mutation() {
    // Verify free list allocator with block splitting correctly updates state
    let source = r#"
struct FLAlloc {
    buf: i64,
    cap: i64,
    head: i64,
    used: i64
}

impl FLAlloc {
    fn new(cap: i64) -> FLAlloc {
        buf := malloc(cap)
        store_i64(buf, cap)
        store_i64(buf + 8, 0)
        FLAlloc { buf: buf, cap: cap, head: buf, used: 0 }
    }

    fn alloc(&self, size: i64) -> i64 {
        needed := size + 16
        needed := I needed < 32 { 32 } else { needed }
        curr := self.head
        I curr == 0 { return 0 }
        bsz := load_i64(curr)
        nxt := load_i64(curr + 8)
        I bsz >= needed {
            I bsz >= needed + 32 {
                new_block := curr + needed
                store_i64(new_block, bsz - needed)
                store_i64(new_block + 8, nxt)
                store_i64(curr, needed)
                self.head = new_block
            } else {
                self.head = nxt
            }
            self.used = self.used + load_i64(curr)
            return curr + 16
        }
        0
    }

    fn dealloc(&self, ptr: i64) -> i64 {
        I ptr == 0 { return 0 }
        block := ptr - 16
        bsz := load_i64(block)
        store_i64(block + 8, self.head)
        self.head = block
        self.used = self.used - bsz
        0
    }

    fn total_used(&self) -> i64 = self.used
    fn drop(&self) -> i64 { free(self.buf); 0 }
}

fn main() -> i64 {
    a := FLAlloc.new(4096)
    p1 := a.alloc(64)
    I p1 == 0 { return 1 }
    p2 := a.alloc(128)
    I p2 == 0 { return 2 }
    I p2 <= p1 { return 3 }
    I a.total_used() == 0 { return 4 }
    a.drop()
    0
}
"#;
    assert_exit_code(source, 0);
}

#[test]
fn test_stack_allocator_state_mutation() {
    // Verify StackAllocator alloc/pop correctly track offset
    let source = r#"
struct StackAllocator {
    buffer: i64, capacity: i64, offset: i64, prev_offset: i64
}

impl StackAllocator {
    fn new(capacity: i64) -> StackAllocator {
        buffer := malloc(capacity)
        StackAllocator {
            buffer: buffer,
            capacity: I buffer != 0 { capacity } else { 0 },
            offset: 0, prev_offset: 0
        }
    }

    fn alloc(&self, size: i64, align: i64) -> i64 {
        header_size := 8
        mask := align - 1
        aligned_offset := (self.offset + header_size + mask) & (~mask)
        new_offset := aligned_offset + size
        I new_offset > self.capacity { return 0 }
        store_i64(self.buffer + aligned_offset - header_size, self.offset)
        self.prev_offset = self.offset
        self.offset = new_offset
        self.buffer + aligned_offset
    }

    fn pop(&self) -> i64 {
        I self.offset == 0 { return 0 }
        self.offset = self.prev_offset
        0
    }

    fn remaining(&self) -> i64 = self.capacity - self.offset

    fn reset(&self) -> i64 {
        self.offset = 0
        self.prev_offset = 0
        0
    }

    fn drop(&self) -> i64 { free(self.buffer); 0 }
}

fn main() -> i64 {
    stack := StackAllocator.new(1024)
    I stack.remaining() != 1024 { return 1 }
    ptr1 := stack.alloc(64, 8)
    I ptr1 == 0 { return 2 }
    rem1 := stack.remaining()
    I rem1 >= 1024 { return 3 }
    ptr2 := stack.alloc(128, 8)
    I ptr2 == 0 { return 4 }
    rem2 := stack.remaining()
    I rem2 >= rem1 { return 5 }
    stack.pop()
    I stack.remaining() != rem1 { return 6 }
    stack.reset()
    I stack.remaining() != 1024 { return 7 }
    stack.drop()
    0
}
"#;
    assert_exit_code(source, 0);
}

// ===== Phase 31 Stage 5: StringMap & OwnedString Tests =====

#[test]
fn e2e_stringmap_insert_and_get() {
    // Test StringMap with str keys: insert, get, update, remove
    let source = r#"
# Hash a string key using DJB2 (operates on i64 pointer)
fn hash_str(p: i64) -> i64 {
    hash_str_rec(p, 5381, 0)
}
fn hash_str_rec(p: i64, h: i64, i: i64) -> i64 {
    b := load_byte(p + i)
    I b == 0 { I h < 0 { 0 - h } else { h } }
    else { hash_str_rec(p, h * 33 + b, i + 1) }
}

# Compare two i64 string pointers byte-by-byte
fn ptr_str_eq(a: i64, b: i64) -> i64 {
    I a == b { return 1 }
    I a == 0 || b == 0 { return 0 }
    ptr_str_eq_rec(a, b, 0)
}
fn ptr_str_eq_rec(a: i64, b: i64, i: i64) -> i64 {
    ca := load_byte(a + i)
    cb := load_byte(b + i)
    I ca != cb { 0 }
    else I ca == 0 { 1 }
    else { ptr_str_eq_rec(a, b, i + 1) }
}

# Duplicate a string from i64 pointer
fn ptr_str_dup(p: i64) -> i64 {
    I p == 0 { return 0 }
    len := str_len_raw(p, 0)
    buf := malloc(len + 1)
    memcpy(buf, p, len + 1)
    buf
}
fn str_len_raw(p: i64, i: i64) -> i64 {
    I load_byte(p + i) == 0 { i } else { str_len_raw(p, i + 1) }
}

fn init_buckets(buckets: i64, i: i64, cap: i64) -> i64 {
    I i >= cap { 0 }
    else { store_i64(buckets + i * 8, 0); init_buckets(buckets, i + 1, cap) }
}

struct StringMap { buckets: i64, size: i64, cap: i64 }

impl StringMap {
    fn with_capacity(capacity: i64) -> StringMap {
        cap := I capacity < 8 { 8 } else { capacity }
        buckets := malloc(cap * 8)
        init_buckets(buckets, 0, cap)
        StringMap { buckets: buckets, size: 0, cap: cap }
    }
    fn len(&self) -> i64 = self.size

    # Public API uses str, converts to i64 via str_to_ptr
    fn set(&self, key: str, value: i64) -> i64 {
        kp := str_to_ptr(key)
        @.set_raw(kp, value)
    }
    fn get(&self, key: str) -> i64 {
        kp := str_to_ptr(key)
        @.get_raw(kp)
    }
    fn contains(&self, key: str) -> i64 {
        kp := str_to_ptr(key)
        @.contains_raw(kp)
    }
    fn remove(&self, key: str) -> i64 {
        kp := str_to_ptr(key)
        @.remove_raw(kp)
    }

    # Internal i64 pointer API
    fn hash_key(&self, kp: i64) -> i64 { h := hash_str(kp); h % self.cap }

    fn set_raw(&self, kp: i64, value: i64) -> i64 {
        idx := @.hash_key(kp)
        ep := load_i64(self.buckets + idx * 8)
        result := @.update_chain(ep, kp, value)
        I result >= 0 { result }
        else {
            kc := ptr_str_dup(kp)
            ne := malloc(24)
            store_i64(ne, kc)
            store_i64(ne + 8, value)
            store_i64(ne + 16, ep)
            store_i64(self.buckets + idx * 8, ne)
            self.size = self.size + 1
            0
        }
    }
    fn get_raw(&self, kp: i64) -> i64 {
        idx := @.hash_key(kp)
        ep := load_i64(self.buckets + idx * 8)
        @.get_chain(ep, kp)
    }
    fn get_chain(&self, ep: i64, kp: i64) -> i64 {
        I ep == 0 { 0 }
        else {
            ek := load_i64(ep)
            I ptr_str_eq(ek, kp) == 1 { load_i64(ep + 8) }
            else { @.get_chain(load_i64(ep + 16), kp) }
        }
    }
    fn contains_raw(&self, kp: i64) -> i64 {
        idx := @.hash_key(kp)
        ep := load_i64(self.buckets + idx * 8)
        @.contains_chain(ep, kp)
    }
    fn contains_chain(&self, ep: i64, kp: i64) -> i64 {
        I ep == 0 { 0 }
        else {
            ek := load_i64(ep)
            I ptr_str_eq(ek, kp) == 1 { 1 }
            else { @.contains_chain(load_i64(ep + 16), kp) }
        }
    }
    fn update_chain(&self, ep: i64, kp: i64, value: i64) -> i64 {
        I ep == 0 { 0 - 1 }
        else {
            ek := load_i64(ep)
            I ptr_str_eq(ek, kp) == 1 {
                old := load_i64(ep + 8)
                store_i64(ep + 8, value)
                old
            } else { @.update_chain(load_i64(ep + 16), kp, value) }
        }
    }
    fn remove_raw(&self, kp: i64) -> i64 {
        idx := @.hash_key(kp)
        ep := load_i64(self.buckets + idx * 8)
        @.remove_chain(idx, 0, ep, kp)
    }
    fn remove_chain(&self, bi: i64, prev: i64, ep: i64, kp: i64) -> i64 {
        I ep == 0 { 0 }
        else {
            ek := load_i64(ep)
            I ptr_str_eq(ek, kp) == 1 {
                val := load_i64(ep + 8)
                nxt := load_i64(ep + 16)
                _ := I prev == 0 { store_i64(self.buckets + bi * 8, nxt); 0 }
                     else { store_i64(prev + 16, nxt); 0 }
                free(ek)
                free(ep)
                self.size = self.size - 1
                val
            } else { @.remove_chain(bi, ep, load_i64(ep + 16), kp) }
        }
    }
}

fn main() -> i64 {
    m := StringMap.with_capacity(16)
    m.set("hello", 100)
    m.set("world", 200)
    m.set("vais", 300)
    I m.len() != 3 { return 1 }
    I m.get("hello") != 100 { return 2 }
    I m.get("world") != 200 { return 3 }
    I m.get("vais") != 300 { return 4 }
    I m.contains("hello") != 1 { return 5 }
    I m.contains("missing") != 0 { return 6 }
    m.set("hello", 999)
    I m.get("hello") != 999 { return 7 }
    I m.len() != 3 { return 8 }
    removed := m.remove("world")
    I removed != 200 { return 9 }
    I m.len() != 2 { return 10 }
    I m.contains("world") != 0 { return 11 }
    0
}
"#;
    assert_exit_code(source, 0);
}

#[test]
fn e2e_stringmap_collision_handling() {
    // Test collision handling with very small bucket count
    let source = r#"
fn hash_str(p: i64) -> i64 { hash_str_rec(p, 5381, 0) }
fn hash_str_rec(p: i64, h: i64, i: i64) -> i64 {
    b := load_byte(p + i)
    I b == 0 { I h < 0 { 0 - h } else { h } }
    else { hash_str_rec(p, h * 33 + b, i + 1) }
}
fn ptr_str_eq(a: i64, b: i64) -> i64 {
    I a == b { return 1 }
    I a == 0 || b == 0 { return 0 }
    ptr_str_eq_rec(a, b, 0)
}
fn ptr_str_eq_rec(a: i64, b: i64, i: i64) -> i64 {
    ca := load_byte(a + i)
    cb := load_byte(b + i)
    I ca != cb { 0 } else I ca == 0 { 1 } else { ptr_str_eq_rec(a, b, i + 1) }
}
fn ptr_str_dup(p: i64) -> i64 {
    I p == 0 { return 0 }
    len := str_len_raw(p, 0)
    buf := malloc(len + 1)
    memcpy(buf, p, len + 1)
    buf
}
fn str_len_raw(p: i64, i: i64) -> i64 {
    I load_byte(p + i) == 0 { i } else { str_len_raw(p, i + 1) }
}
fn init_buckets(buckets: i64, i: i64, cap: i64) -> i64 {
    I i >= cap { 0 } else { store_i64(buckets + i * 8, 0); init_buckets(buckets, i + 1, cap) }
}

struct StringMap { buckets: i64, size: i64, cap: i64 }
impl StringMap {
    fn with_capacity(capacity: i64) -> StringMap {
        cap := I capacity < 8 { 8 } else { capacity }
        buckets := malloc(cap * 8)
        init_buckets(buckets, 0, cap)
        StringMap { buckets: buckets, size: 0, cap: cap }
    }
    fn len(&self) -> i64 = self.size
    fn set(&self, key: str, value: i64) -> i64 { kp := str_to_ptr(key); @.set_raw(kp, value) }
    fn get(&self, key: str) -> i64 { kp := str_to_ptr(key); @.get_raw(kp) }
    fn contains(&self, key: str) -> i64 { kp := str_to_ptr(key); @.contains_raw(kp) }
    fn hash_key(&self, kp: i64) -> i64 { h := hash_str(kp); h % self.cap }
    fn set_raw(&self, kp: i64, value: i64) -> i64 {
        idx := @.hash_key(kp)
        ep := load_i64(self.buckets + idx * 8)
        result := @.update_chain(ep, kp, value)
        I result >= 0 { result }
        else {
            kc := ptr_str_dup(kp)
            ne := malloc(24)
            store_i64(ne, kc); store_i64(ne + 8, value); store_i64(ne + 16, ep)
            store_i64(self.buckets + idx * 8, ne)
            self.size = self.size + 1; 0
        }
    }
    fn get_raw(&self, kp: i64) -> i64 {
        idx := @.hash_key(kp)
        @.get_chain(load_i64(self.buckets + idx * 8), kp)
    }
    fn get_chain(&self, ep: i64, kp: i64) -> i64 {
        I ep == 0 { 0 } else {
            ek := load_i64(ep)
            I ptr_str_eq(ek, kp) == 1 { load_i64(ep + 8) }
            else { @.get_chain(load_i64(ep + 16), kp) }
        }
    }
    fn contains_raw(&self, kp: i64) -> i64 {
        idx := @.hash_key(kp)
        @.contains_chain(load_i64(self.buckets + idx * 8), kp)
    }
    fn contains_chain(&self, ep: i64, kp: i64) -> i64 {
        I ep == 0 { 0 } else {
            ek := load_i64(ep)
            I ptr_str_eq(ek, kp) == 1 { 1 }
            else { @.contains_chain(load_i64(ep + 16), kp) }
        }
    }
    fn update_chain(&self, ep: i64, kp: i64, value: i64) -> i64 {
        I ep == 0 { 0 - 1 } else {
            ek := load_i64(ep)
            I ptr_str_eq(ek, kp) == 1 { old := load_i64(ep + 8); store_i64(ep + 8, value); old }
            else { @.update_chain(load_i64(ep + 16), kp, value) }
        }
    }
}

fn main() -> i64 {
    # Small capacity forces collisions
    m := StringMap.with_capacity(2)
    m.set("alpha", 1)
    m.set("beta", 2)
    m.set("gamma", 3)
    m.set("delta", 4)
    m.set("epsilon", 5)
    I m.len() != 5 { return 1 }
    I m.get("alpha") != 1 { return 2 }
    I m.get("beta") != 2 { return 3 }
    I m.get("gamma") != 3 { return 4 }
    I m.get("delta") != 4 { return 5 }
    I m.get("epsilon") != 5 { return 6 }
    I m.contains("alpha") != 1 { return 7 }
    I m.contains("nonexistent") != 0 { return 8 }
    0
}
"#;
    assert_exit_code(source, 0);
}

#[test]
fn e2e_owned_string_basic() {
    // Test OwnedString: from_str, push_char, push_str, eq_str, clone, clear
    let source = r#"
struct OwnedString { data: i64, len: i64, cap: i64 }

impl OwnedString {
    fn with_capacity(capacity: i64) -> OwnedString {
        cap := mut I capacity < 16 { 16 } else { capacity }
        data := mut malloc(cap)
        store_byte(data, 0)
        OwnedString { data: data, len: 0, cap: cap }
    }
    fn from_cstr(s: str) -> OwnedString {
        p := str_to_ptr(s)
        len := mut strlen(s)
        cap := mut len + 16
        data := mut malloc(cap)
        memcpy(data, p, len + 1)
        OwnedString { data: data, len: len, cap: cap }
    }
    fn len(&self) -> i64 = self.len
    fn push_char(&self, c: i64) -> i64 {
        I self.len >= self.cap - 1 { @.grow() } else { 0 }
        store_byte(self.data + self.len, c)
        self.len = self.len + 1
        store_byte(self.data + self.len, 0)
        self.len
    }
    fn push_cstr(&self, s: str) -> i64 {
        p := str_to_ptr(s)
        slen := strlen(s)
        I slen == 0 { return self.len }
        I self.len + slen + 1 > self.cap { @.grow() } else { 0 }
        memcpy(self.data + self.len, p, slen + 1)
        self.len = self.len + slen
        self.len
    }
    fn grow(&self) -> i64 {
        new_cap := I self.cap * 2 < 16 { 16 } else { self.cap * 2 }
        new_data := malloc(new_cap)
        memcpy(new_data, self.data, self.len + 1)
        free(self.data)
        self.data = new_data
        self.cap = new_cap
        new_cap
    }
    fn eq_cstr(&self, s: str) -> i64 {
        p := str_to_ptr(s)
        slen := strlen(s)
        I self.len != slen { return 0 }
        memcmp_rec(self.data, p, 0, self.len)
    }
    fn copy(&self) -> OwnedString {
        new_data := malloc(self.cap)
        memcpy(new_data, self.data, self.len + 1)
        OwnedString { data: new_data, len: self.len, cap: self.cap }
    }
    fn clear(&self) -> i64 {
        self.len = 0
        store_byte(self.data, 0)
        0
    }
    fn drop(&self) -> i64 {
        I self.data != 0 { free(self.data) }
        self.data = 0
        self.len = 0
        self.cap = 0
        0
    }
}

fn memcmp_rec(a: i64, b: i64, idx: i64, len: i64) -> i64 {
    I idx >= len { 1 }
    else {
        I load_byte(a + idx) != load_byte(b + idx) { 0 }
        else { memcmp_rec(a, b, idx + 1, len) }
    }
}

fn main() -> i64 {
    s := OwnedString.from_cstr("hello")
    I s.len() != 5 { return 1 }
    I s.eq_cstr("hello") != 1 { return 2 }
    I s.eq_cstr("world") != 0 { return 3 }
    s.push_char(33)
    I s.len() != 6 { return 4 }
    I s.eq_cstr("hello!") != 1 { return 5 }
    s.push_cstr(" world")
    I s.len() != 12 { return 6 }
    I s.eq_cstr("hello! world") != 1 { return 7 }
    s2 := s.copy()
    I s2.eq_cstr("hello! world") != 1 { return 8 }
    s.clear()
    I s.len() != 0 { return 9 }
    I s2.eq_cstr("hello! world") != 1 { return 10 }
    e := OwnedString.with_capacity(32)
    I e.len() != 0 { return 11 }
    e.push_cstr("test")
    I e.eq_cstr("test") != 1 { return 12 }
    s.drop()
    s2.drop()
    e.drop()
    0
}
"#;
    assert_exit_code(source, 0);
}

#[test]
fn e2e_stringmap_with_dynamic_keys() {
    // Test StringMap + OwnedString: build dynamic keys, insert, look up with literals
    let source = r#"
fn hash_str(p: i64) -> i64 { hash_str_rec(p, 5381, 0) }
fn hash_str_rec(p: i64, h: i64, i: i64) -> i64 {
    b := load_byte(p + i)
    I b == 0 { I h < 0 { 0 - h } else { h } } else { hash_str_rec(p, h * 33 + b, i + 1) }
}
fn ptr_str_eq(a: i64, b: i64) -> i64 {
    I a == b { return 1 }
    I a == 0 || b == 0 { return 0 }
    ptr_str_eq_rec(a, b, 0)
}
fn ptr_str_eq_rec(a: i64, b: i64, i: i64) -> i64 {
    ca := load_byte(a + i)
    cb := load_byte(b + i)
    I ca != cb { 0 }
    else I ca == 0 { 1 }
    else { ptr_str_eq_rec(a, b, i + 1) }
}
fn ptr_str_dup(p: i64) -> i64 {
    I p == 0 { return 0 }
    len := mut str_len_raw(p, 0)
    buf := malloc(len + 1)
    memcpy(buf, p, len + 1)
    buf
}
fn str_len_raw(p: i64, i: i64) -> i64 {
    I load_byte(p + i) == 0 { i } else { str_len_raw(p, i + 1) }
}
fn init_buckets(buckets: i64, i: i64, cap: i64) -> i64 {
    I i >= cap { 0 }
    else {
        store_i64(buckets + i * 8, 0)
        init_buckets(buckets, i + 1, cap)
    }
}

struct StringMap { buckets: i64, size: i64, cap: i64 }
impl StringMap {
    fn with_capacity(capacity: i64) -> StringMap {
        cap := mut I capacity < 8 { 8 } else { capacity }
        buckets := malloc(cap * 8)
        init_buckets(buckets, 0, cap)
        StringMap { buckets: buckets, size: 0, cap: cap }
    }
    fn len(&self) -> i64 = self.size
    fn set(&self, key: str, value: i64) -> i64 {
        kp := str_to_ptr(key)
        @.set_raw(kp, value)
    }
    fn get(&self, key: str) -> i64 {
        kp := str_to_ptr(key)
        @.get_raw(kp)
    }
    fn set_ptr(&self, kp: i64, value: i64) -> i64 {
        @.set_raw(kp, value)
    }
    fn get_ptr(&self, kp: i64) -> i64 {
        @.get_raw(kp)
    }
    fn hash_key(&self, kp: i64) -> i64 {
        h := hash_str(kp)
        h % self.cap
    }
    fn set_raw(&self, kp: i64, value: i64) -> i64 {
        idx := @.hash_key(kp)
        ep := load_i64(self.buckets + idx * 8)
        result := @.update_chain(ep, kp, value)
        I result >= 0 { result }
        else {
            kc := ptr_str_dup(kp)
            ne := malloc(24)
            store_i64(ne, kc)
            store_i64(ne + 8, value)
            store_i64(ne + 16, ep)
            store_i64(self.buckets + idx * 8, ne)
            self.size = self.size + 1
            0
        }
    }
    fn get_raw(&self, kp: i64) -> i64 {
        idx := @.hash_key(kp)
        @.get_chain(load_i64(self.buckets + idx * 8), kp)
    }
    fn get_chain(&self, ep: i64, kp: i64) -> i64 {
        I ep == 0 { 0 }
        else {
            ek := load_i64(ep)
            I ptr_str_eq(ek, kp) == 1 { load_i64(ep + 8) }
            else { @.get_chain(load_i64(ep + 16), kp) }
        }
    }
    fn update_chain(&self, ep: i64, kp: i64, value: i64) -> i64 {
        I ep == 0 { 0 - 1 }
        else {
            ek := load_i64(ep)
            I ptr_str_eq(ek, kp) == 1 {
                old := load_i64(ep + 8)
                store_i64(ep + 8, value)
                old
            } else { @.update_chain(load_i64(ep + 16), kp, value) }
        }
    }
}

struct OwnedString { data: i64, len: i64, cap: i64 }
impl OwnedString {
    fn with_capacity(capacity: i64) -> OwnedString {
        cap := mut I capacity < 16 { 16 } else { capacity }
        data := mut malloc(cap)
        store_byte(data, 0)
        OwnedString { data: data, len: 0, cap: cap }
    }
    fn as_ptr(&self) -> i64 = self.data
    fn push_cstr(&self, s: str) -> i64 {
        p := str_to_ptr(s)
        slen := strlen(s)
        I slen == 0 { return self.len }
        I self.len + slen + 1 > self.cap { @.grow() } else { 0 }
        memcpy(self.data + self.len, p, slen + 1)
        self.len = self.len + slen
        self.len
    }
    fn grow(&self) -> i64 {
        new_cap := I self.cap * 2 < 16 { 16 } else { self.cap * 2 }
        new_data := malloc(new_cap)
        memcpy(new_data, self.data, self.len + 1)
        free(self.data)
        self.data = new_data
        self.cap = new_cap
        new_cap
    }
    fn drop(&self) -> i64 {
        I self.data != 0 { free(self.data) }
        self.data = 0
        self.len = 0
        self.cap = 0
        0
    }
}

fn main() -> i64 {
    m := StringMap.with_capacity(16)
    key1 := OwnedString.with_capacity(64)
    key1.push_cstr("table_")
    key1.push_cstr("users")
    key2 := OwnedString.with_capacity(64)
    key2.push_cstr("table_")
    key2.push_cstr("orders")
    m.set_ptr(key1.as_ptr(), 42)
    m.set_ptr(key2.as_ptr(), 99)
    I m.len() != 2 { return 1 }
    I m.get("table_users") != 42 { return 2 }
    I m.get("table_orders") != 99 { return 3 }
    I m.get_ptr(key1.as_ptr()) != 42 { return 4 }
    I m.get_ptr(key2.as_ptr()) != 99 { return 5 }
    key1.drop()
    key2.drop()
    0
}
"#;
    assert_exit_code(source, 0);
}

#[test]
fn e2e_stringmap_delete_and_reinsert() {
    // Test delete + reinsert of same key
    let source = r#"
fn hash_str(p: i64) -> i64 { hash_str_rec(p, 5381, 0) }
fn hash_str_rec(p: i64, h: i64, i: i64) -> i64 {
    b := load_byte(p + i)
    I b == 0 { I h < 0 { 0 - h } else { h } } else { hash_str_rec(p, h * 33 + b, i + 1) }
}
fn ptr_str_eq(a: i64, b: i64) -> i64 {
    I a == b { return 1 }; I a == 0 || b == 0 { return 0 }; ptr_str_eq_rec(a, b, 0)
}
fn ptr_str_eq_rec(a: i64, b: i64, i: i64) -> i64 {
    ca := load_byte(a + i); cb := load_byte(b + i)
    I ca != cb { 0 } else I ca == 0 { 1 } else { ptr_str_eq_rec(a, b, i + 1) }
}
fn ptr_str_dup(p: i64) -> i64 {
    I p == 0 { return 0 }; len := str_len_raw(p, 0)
    buf := malloc(len + 1); memcpy(buf, p, len + 1); buf
}
fn str_len_raw(p: i64, i: i64) -> i64 {
    I load_byte(p + i) == 0 { i } else { str_len_raw(p, i + 1) }
}
fn init_buckets(buckets: i64, i: i64, cap: i64) -> i64 {
    I i >= cap { 0 } else { store_i64(buckets + i * 8, 0); init_buckets(buckets, i + 1, cap) }
}

struct StringMap { buckets: i64, size: i64, cap: i64 }
impl StringMap {
    fn with_capacity(capacity: i64) -> StringMap {
        cap := I capacity < 8 { 8 } else { capacity }
        buckets := malloc(cap * 8); init_buckets(buckets, 0, cap)
        StringMap { buckets: buckets, size: 0, cap: cap }
    }
    fn len(&self) -> i64 = self.size
    fn set(&self, key: str, value: i64) -> i64 { kp := str_to_ptr(key); @.set_raw(kp, value) }
    fn get(&self, key: str) -> i64 { kp := str_to_ptr(key); @.get_raw(kp) }
    fn contains(&self, key: str) -> i64 { kp := str_to_ptr(key); @.contains_raw(kp) }
    fn remove(&self, key: str) -> i64 { kp := str_to_ptr(key); @.remove_raw(kp) }
    fn hash_key(&self, kp: i64) -> i64 { h := hash_str(kp); h % self.cap }
    fn set_raw(&self, kp: i64, value: i64) -> i64 {
        idx := @.hash_key(kp); ep := load_i64(self.buckets + idx * 8)
        result := @.update_chain(ep, kp, value)
        I result >= 0 { result } else {
            kc := ptr_str_dup(kp); ne := malloc(24)
            store_i64(ne, kc); store_i64(ne + 8, value); store_i64(ne + 16, ep)
            store_i64(self.buckets + idx * 8, ne); self.size = self.size + 1; 0
        }
    }
    fn get_raw(&self, kp: i64) -> i64 {
        idx := @.hash_key(kp); @.get_chain(load_i64(self.buckets + idx * 8), kp)
    }
    fn get_chain(&self, ep: i64, kp: i64) -> i64 {
        I ep == 0 { 0 } else {
            ek := load_i64(ep)
            I ptr_str_eq(ek, kp) == 1 { load_i64(ep + 8) }
            else { @.get_chain(load_i64(ep + 16), kp) }
        }
    }
    fn contains_raw(&self, kp: i64) -> i64 {
        idx := @.hash_key(kp); @.contains_chain(load_i64(self.buckets + idx * 8), kp)
    }
    fn contains_chain(&self, ep: i64, kp: i64) -> i64 {
        I ep == 0 { 0 } else {
            ek := load_i64(ep)
            I ptr_str_eq(ek, kp) == 1 { 1 } else { @.contains_chain(load_i64(ep + 16), kp) }
        }
    }
    fn update_chain(&self, ep: i64, kp: i64, value: i64) -> i64 {
        I ep == 0 { 0 - 1 } else {
            ek := load_i64(ep)
            I ptr_str_eq(ek, kp) == 1 { old := load_i64(ep + 8); store_i64(ep + 8, value); old }
            else { @.update_chain(load_i64(ep + 16), kp, value) }
        }
    }
    fn remove_raw(&self, kp: i64) -> i64 {
        idx := @.hash_key(kp); ep := load_i64(self.buckets + idx * 8)
        @.remove_chain(idx, 0, ep, kp)
    }
    fn remove_chain(&self, bi: i64, prev: i64, ep: i64, kp: i64) -> i64 {
        I ep == 0 { 0 } else {
            ek := load_i64(ep)
            I ptr_str_eq(ek, kp) == 1 {
                val := load_i64(ep + 8); nxt := load_i64(ep + 16)
                _ := I prev == 0 { store_i64(self.buckets + bi * 8, nxt); 0 }
                     else { store_i64(prev + 16, nxt); 0 }
                free(ek); free(ep); self.size = self.size - 1; val
            } else { @.remove_chain(bi, ep, load_i64(ep + 16), kp) }
        }
    }
}

fn main() -> i64 {
    m := StringMap.with_capacity(8)
    m.set("name", 1)
    m.set("age", 2)
    m.set("city", 3)
    removed := m.remove("age")
    I removed != 2 { return 1 }
    I m.len() != 2 { return 2 }
    I m.contains("age") != 0 { return 3 }
    m.set("age", 99)
    I m.len() != 3 { return 4 }
    I m.get("age") != 99 { return 5 }
    I m.get("name") != 1 { return 6 }
    I m.get("city") != 3 { return 7 }
    m.remove("name")
    m.remove("age")
    m.remove("city")
    I m.len() != 0 { return 8 }
    0
}
"#;
    assert_exit_code(source, 0);
}

// ========== Phase 31 Stage 6: Filesystem FFI Tests ==========

#[test]
fn e2e_mkdir_rmdir() {
    let source = r#"
fn main() -> i64 {
    rmdir("/tmp/vais_e2e_mkdir_1234")
    r1 := mkdir("/tmp/vais_e2e_mkdir_1234", 493)
    I r1 != 0 { return 1 }
    d := opendir("/tmp/vais_e2e_mkdir_1234")
    I d == 0 { return 2 }
    closedir(d)
    r2 := rmdir("/tmp/vais_e2e_mkdir_1234")
    I r2 != 0 { return 3 }
    d2 := opendir("/tmp/vais_e2e_mkdir_1234")
    I d2 != 0 { closedir(d2); return 4 }
    0
}
"#;
    assert_exit_code(source, 0);
}

#[test]
fn e2e_file_rename_unlink() {
    let source = r#"
fn main() -> i64 {
    unlink("/tmp/vais_e2e_rename_old")
    unlink("/tmp/vais_e2e_rename_new")
    fp := fopen("/tmp/vais_e2e_rename_old", "w")
    I fp == 0 { return 1 }
    fputs("hello", fp)
    fclose(fp)
    r := rename_file("/tmp/vais_e2e_rename_old", "/tmp/vais_e2e_rename_new")
    I r != 0 { return 2 }
    fp2 := fopen("/tmp/vais_e2e_rename_new", "r")
    I fp2 == 0 { return 3 }
    fclose(fp2)
    fp3 := fopen("/tmp/vais_e2e_rename_old", "r")
    I fp3 != 0 { fclose(fp3); return 4 }
    unlink("/tmp/vais_e2e_rename_new")
    0
}
"#;
    assert_exit_code(source, 0);
}

#[test]
#[cfg_attr(
    not(target_os = "macos"),
    ignore = "stat struct layout differs between platforms"
)]
fn e2e_stat_file_size() {
    let source = r#"
fn main() -> i64 {
    unlink("/tmp/vais_e2e_stat_size")
    fp := fopen("/tmp/vais_e2e_stat_size", "w")
    I fp == 0 { return 1 }
    fputs("Hello, World!", fp)
    fclose(fp)
    size := stat_size("/tmp/vais_e2e_stat_size")
    unlink("/tmp/vais_e2e_stat_size")
    I size == 13 { 0 } else { return 2 }
}
"#;
    assert_exit_code(source, 0);
}

// ========== Phase 31 Stage 7: ByteBuffer + CRC32 Tests ==========

#[test]
fn e2e_bytebuffer_write_read_integers() {
    let source = r#"
fn grow_cap(cap: i64, needed: i64) -> i64 {
    I cap >= needed { cap } else { grow_cap(cap * 2, needed) }
}

struct ByteBuffer { data: i64, len: i64, cap: i64, pos: i64 }

impl ByteBuffer {
    fn with_capacity(capacity: i64) -> ByteBuffer {
        cap := mut I capacity < 16 { 16 } else { capacity }
        data := mut malloc(cap)
        ByteBuffer { data: data, len: 0, cap: cap, pos: 0 }
    }
    fn ensure_capacity(&self, needed: i64) -> i64 {
        I needed <= self.cap { return self.cap }
        new_cap := grow_cap(self.cap, needed)
        new_data := malloc(new_cap)
        memcpy(new_data, self.data, self.len)
        free(self.data)
        self.data = new_data
        self.cap = new_cap
        new_cap
    }
    fn write_u8(&self, value: i64) -> i64 {
        @.ensure_capacity(self.len + 1)
        store_byte(self.data + self.len, value & 255)
        self.len = self.len + 1
        1
    }
    fn read_u8(&self) -> i64 {
        I self.pos >= self.len { return 0 - 1 }
        val := load_byte(self.data + self.pos)
        self.pos = self.pos + 1
        val
    }
    fn write_i32_le(&self, value: i64) -> i64 {
        @.ensure_capacity(self.len + 4)
        store_byte(self.data + self.len, value & 255)
        store_byte(self.data + self.len + 1, (value >> 8) & 255)
        store_byte(self.data + self.len + 2, (value >> 16) & 255)
        store_byte(self.data + self.len + 3, (value >> 24) & 255)
        self.len = self.len + 4
        4
    }
    fn read_i32_le(&self) -> i64 {
        I self.pos + 4 > self.len { return 0 - 1 }
        b0 := load_byte(self.data + self.pos)
        b1 := load_byte(self.data + self.pos + 1)
        b2 := load_byte(self.data + self.pos + 2)
        b3 := load_byte(self.data + self.pos + 3)
        self.pos = self.pos + 4
        b0 | (b1 << 8) | (b2 << 16) | (b3 << 24)
    }
    fn write_i64_le(&self, value: i64) -> i64 {
        @.ensure_capacity(self.len + 8)
        store_byte(self.data + self.len, value & 255)
        store_byte(self.data + self.len + 1, (value >> 8) & 255)
        store_byte(self.data + self.len + 2, (value >> 16) & 255)
        store_byte(self.data + self.len + 3, (value >> 24) & 255)
        store_byte(self.data + self.len + 4, (value >> 32) & 255)
        store_byte(self.data + self.len + 5, (value >> 40) & 255)
        store_byte(self.data + self.len + 6, (value >> 48) & 255)
        store_byte(self.data + self.len + 7, (value >> 56) & 255)
        self.len = self.len + 8
        8
    }
    fn read_i64_le(&self) -> i64 {
        I self.pos + 8 > self.len { return 0 - 1 }
        b0 := load_byte(self.data + self.pos)
        b1 := load_byte(self.data + self.pos + 1)
        b2 := load_byte(self.data + self.pos + 2)
        b3 := load_byte(self.data + self.pos + 3)
        b4 := load_byte(self.data + self.pos + 4)
        b5 := load_byte(self.data + self.pos + 5)
        b6 := load_byte(self.data + self.pos + 6)
        b7 := load_byte(self.data + self.pos + 7)
        self.pos = self.pos + 8
        b0 | (b1 << 8) | (b2 << 16) | (b3 << 24) | (b4 << 32) | (b5 << 40) | (b6 << 48) | (b7 << 56)
    }
    fn rewind(&self) -> i64 {
        self.pos = 0
        0
    }
    fn drop(&self) -> i64 {
        I self.data != 0 { free(self.data) }
        self.data = 0
        self.len = 0
        self.cap = 0
        self.pos = 0
        0
    }
}

fn main() -> i64 {
    buf := ByteBuffer.with_capacity(64)
    buf.write_u8(42)
    buf.write_i32_le(12345)
    buf.write_i64_le(9876543210)
    buf.rewind()
    I buf.read_u8() != 42 { return 1 }
    I buf.read_i32_le() != 12345 { return 2 }
    I buf.read_i64_le() != 9876543210 { return 3 }
    buf.drop()
    0
}
"#;
    assert_exit_code(source, 0);
}

#[test]
fn e2e_bytebuffer_grow() {
    let source = r#"
fn grow_cap(cap: i64, needed: i64) -> i64 {
    I cap >= needed { cap } else { grow_cap(cap * 2, needed) }
}

struct ByteBuffer { data: i64, len: i64, cap: i64, pos: i64 }

impl ByteBuffer {
    fn with_capacity(capacity: i64) -> ByteBuffer {
        cap := mut I capacity < 16 { 16 } else { capacity }
        data := mut malloc(cap)
        ByteBuffer { data: data, len: 0, cap: cap, pos: 0 }
    }
    fn ensure_capacity(&self, needed: i64) -> i64 {
        I needed <= self.cap { return self.cap }
        new_cap := grow_cap(self.cap, needed)
        new_data := malloc(new_cap)
        memcpy(new_data, self.data, self.len)
        free(self.data)
        self.data = new_data
        self.cap = new_cap
        new_cap
    }
    fn write_u8(&self, value: i64) -> i64 {
        @.ensure_capacity(self.len + 1)
        store_byte(self.data + self.len, value & 255)
        self.len = self.len + 1
        1
    }
    fn write_n(&self, n: i64) -> i64 {
        @.write_n_rec(0, n)
    }
    fn write_n_rec(&self, i: i64, n: i64) -> i64 {
        I i >= n { 0 }
        else {
            @.write_u8(i & 255)
            @.write_n_rec(i + 1, n)
        }
    }
    fn drop(&self) -> i64 {
        I self.data != 0 { free(self.data) }
        self.data = 0
        0
    }
}

fn verify_buf(data: i64, i: i64, n: i64) -> i64 {
    I i >= n { 0 }
    else {
        val := load_byte(data + i)
        expected := i & 255
        I val != expected { return i + 1 }
        verify_buf(data, i + 1, n)
    }
}

fn main() -> i64 {
    buf := ByteBuffer.with_capacity(16)
    buf.write_n(100)
    I buf.len != 100 { return 1 }
    I buf.cap < 100 { return 2 }
    result := verify_buf(buf.data, 0, 100)
    I result != 0 { return result + 100 }
    buf.drop()
    0
}
"#;
    assert_exit_code(source, 0);
}

#[test]
fn e2e_crc32_known_values() {
    let source = r#"
fn crc32_update_byte(crc: i64, byte_val: i64) -> i64 {
    v := crc ^ byte_val
    masked := v & 4294967295
    crc32_update_bit(masked, 0)
}

fn crc32_update_bit(crc: i64, bit: i64) -> i64 {
    I bit >= 8 { crc & 4294967295 }
    else {
        low_bit := crc & 1
        shifted := crc >> 1
        masked_shift := shifted & 2147483647
        next := I low_bit == 1 {
            masked_shift ^ 3988292384
        } else {
            masked_shift
        }
        n := next & 4294967295
        crc32_update_bit(n, bit + 1)
    }
}

fn crc32_loop(data: i64, crc: i64, idx: i64, len: i64) -> i64 {
    I idx >= len { crc }
    else {
        byte_val := load_byte(data + idx)
        new_crc := crc32_update_byte(crc, byte_val)
        crc32_loop(data, new_crc, idx + 1, len)
    }
}

fn crc32(data: i64, len: i64) -> i64 {
    result := crc32_loop(data, 4294967295, 0, len)
    xored := result ^ 4294967295
    xored & 4294967295
}

fn crc32_str(s: str) -> i64 {
    p := str_to_ptr(s)
    len := strlen(s)
    crc32(p, len)
}

fn main() -> i64 {
    r1 := crc32_str("")
    I r1 != 0 { return 1 }
    r2 := crc32_str("123456789")
    I r2 != 3421780262 { return 2 }
    r3 := crc32_str("a")
    I r3 != 3904355907 { return 3 }
    0
}
"#;
    assert_exit_code(source, 0);
}

#[test]
fn e2e_bytebuffer_with_crc32() {
    let source = r#"
fn grow_cap(cap: i64, needed: i64) -> i64 {
    I cap >= needed { cap } else { grow_cap(cap * 2, needed) }
}

struct ByteBuffer { data: i64, len: i64, cap: i64, pos: i64 }

impl ByteBuffer {
    fn with_capacity(capacity: i64) -> ByteBuffer {
        cap := mut I capacity < 16 { 16 } else { capacity }
        data := mut malloc(cap)
        ByteBuffer { data: data, len: 0, cap: cap, pos: 0 }
    }
    fn ensure_capacity(&self, needed: i64) -> i64 {
        I needed <= self.cap { return self.cap }
        new_cap := grow_cap(self.cap, needed)
        new_data := malloc(new_cap)
        memcpy(new_data, self.data, self.len)
        free(self.data)
        self.data = new_data
        self.cap = new_cap
        new_cap
    }
    fn write_u8(&self, value: i64) -> i64 {
        @.ensure_capacity(self.len + 1)
        store_byte(self.data + self.len, value & 255)
        self.len = self.len + 1
        1
    }
    fn write_i32_le(&self, value: i64) -> i64 {
        @.ensure_capacity(self.len + 4)
        store_byte(self.data + self.len, value & 255)
        store_byte(self.data + self.len + 1, (value >> 8) & 255)
        store_byte(self.data + self.len + 2, (value >> 16) & 255)
        store_byte(self.data + self.len + 3, (value >> 24) & 255)
        self.len = self.len + 4
        4
    }
    fn drop(&self) -> i64 {
        I self.data != 0 { free(self.data) }
        self.data = 0
        0
    }
}

fn crc32_update_byte(crc: i64, byte_val: i64) -> i64 {
    v := crc ^ byte_val
    masked := v & 4294967295
    crc32_update_bit(masked, 0)
}

fn crc32_update_bit(crc: i64, bit: i64) -> i64 {
    I bit >= 8 { crc & 4294967295 }
    else {
        low_bit := crc & 1
        shifted := crc >> 1
        masked_shift := shifted & 2147483647
        next := I low_bit == 1 {
            masked_shift ^ 3988292384
        } else {
            masked_shift
        }
        n := next & 4294967295
        crc32_update_bit(n, bit + 1)
    }
}

fn crc32_loop(data: i64, crc: i64, idx: i64, len: i64) -> i64 {
    I idx >= len { crc }
    else {
        byte_val := load_byte(data + idx)
        new_crc := crc32_update_byte(crc, byte_val)
        crc32_loop(data, new_crc, idx + 1, len)
    }
}

fn crc32(data: i64, len: i64) -> i64 {
    result := crc32_loop(data, 4294967295, 0, len)
    xored := result ^ 4294967295
    xored & 4294967295
}

fn main() -> i64 {
    buf := ByteBuffer.with_capacity(64)
    buf.write_u8(1)
    buf.write_u8(2)
    buf.write_u8(3)
    buf.write_i32_le(42)
    checksum := crc32(buf.data, buf.len)

    buf2 := ByteBuffer.with_capacity(64)
    buf2.write_u8(1)
    buf2.write_u8(2)
    buf2.write_u8(3)
    buf2.write_i32_le(42)
    checksum2 := crc32(buf2.data, buf2.len)

    buf.drop()
    buf2.drop()

    I checksum != checksum2 { return 1 }
    I checksum == 0 { return 2 }
    0
}
"#;
    assert_exit_code(source, 0);
}

// ========== Phase 31 Stage 8: ? Operator + Error Propagation Tests ==========

#[test]
fn e2e_try_operator_result_ok() {
    // Test ? operator on Ok result - should extract value
    // compute(20): safe_divide(20,2)=Ok(10), ? extracts 10, Ok(10+10)=Ok(20)
    // main matches Ok(v) => v (20), then 20 - 20 = 0
    let source = r#"
enum Result {
    Ok(i64),
    Err(i64)
}

fn safe_divide(a: i64, b: i64) -> Result {
    I b == 0 { Err(1) } else { Ok(a / b) }
}

fn compute(x: i64) -> Result {
    v := safe_divide(x, 2)?
    return Ok(v + 10)
}

fn main() -> i64 {
    r := compute(20)
    v := match r {
        Ok(val) => val,
        Err(_) => 99
    }
    v - 20
}
"#;
    assert_exit_code(source, 0);
}

#[test]
fn e2e_try_operator_result_err_propagation() {
    // Test ? operator on Err result - should propagate error
    // compute() calls failing_op() which returns Err(42), ? propagates it
    // main matches Err(e) => e, so exit code = 42
    let source = r#"
enum Result {
    Ok(i64),
    Err(i64)
}

fn failing_op() -> Result {
    Err(42)
}

fn compute() -> Result {
    v := failing_op()?
    return Ok(v + 100)
}

fn main() -> i64 {
    r := compute()
    v := match r {
        Ok(_) => 1,
        Err(e) => e
    }
    v - 42
}
"#;
    assert_exit_code(source, 0);
}

#[test]
fn e2e_try_operator_chaining() {
    // Test chaining ? operators via nested function calls
    // pipeline calls step1_then_step2 which uses ? then calls step2
    // pipeline(10): step1(10)=Ok(20) -> ? -> 20 -> step2(20)=Ok(25)
    // main matches Ok(v) => v, exit code = 25
    let source = r#"
enum Result {
    Ok(i64),
    Err(i64)
}

fn step1(x: i64) -> Result {
    I x < 0 { Err(1) } else { Ok(x * 2) }
}

fn step2(x: i64) -> Result {
    I x > 100 { Err(2) } else { Ok(x + 5) }
}

fn apply_step2(a: i64) -> Result {
    step2(a)
}

fn pipeline(x: i64) -> Result {
    a := step1(x)?
    return apply_step2(a)
}

fn main() -> i64 {
    r := pipeline(10)
    v := match r { Ok(val) => val, Err(_) => 99 }
    v - 25
}
"#;
    assert_exit_code(source, 0);
}

#[test]
fn e2e_result_methods() {
    // Test Result enum with match-based helper functions
    let source = r#"
enum Result {
    Ok(i64),
    Err(i64)
}

fn is_ok(r: Result) -> i64 {
    match r { Ok(_) => 1, Err(_) => 0 }
}

fn unwrap_or(r: Result, default: i64) -> i64 {
    match r { Ok(v) => v, Err(_) => default }
}

fn main() -> i64 {
    ok := Ok(42)
    err := Err(99)
    ok_check := is_ok(ok)
    err_check := is_ok(err)
    ok_val := unwrap_or(ok, 0)
    err_val := unwrap_or(err, 0)
    ok_check - 1 + err_check + ok_val - 42 + err_val
}
"#;
    assert_exit_code(source, 0);
}

#[test]
fn e2e_while_loop() {
    let source = r#"
fn main() -> i64 {
    i := mut 0
    total := mut 0
    L i < 5 {
        total = total + i
        i = i + 1
    }
    total - 10
}
"#;
    assert_exit_code(source, 0);
}

#[test]
fn e2e_while_loop_nested() {
    let source = r#"
fn main() -> i64 {
    i := mut 0
    total := mut 0
    L i < 3 {
        j := mut 0
        L j < 3 {
            total = total + 1
            j = j + 1
        }
        i = i + 1
    }
    total - 9
}
"#;
    assert_exit_code(source, 0);
}

#[test]
fn e2e_while_loop_with_break() {
    let source = r#"
fn main() -> i64 {
    i := mut 0
    L i < 100 {
        I i == 5 { B }
        i = i + 1
    }
    i - 5
}
"#;
    assert_exit_code(source, 0);
}

#[test]
fn e2e_match_with_wildcard() {
    let source = r#"
fn main() -> i64 {
    x := 42
    match x {
        1 => 10,
        2 => 20,
        _ => 0,
    }
}
"#;
    assert_exit_code(source, 0);
}

#[test]
fn e2e_match_with_binding() {
    let source = r#"
fn main() -> i64 {
    x := 5
    match x {
        0 => 99,
        n => n - 5,
    }
}
"#;
    assert_exit_code(source, 0);
}

#[test]
fn e2e_match_with_guard() {
    let source = r#"
fn main() -> i64 {
    x := 15
    match x {
        n I n > 10 => n - 15,
        n => n,
    }
}
"#;
    assert_exit_code(source, 0);
}

#[test]
fn e2e_match_or_pattern() {
    let source = r#"
fn main() -> i64 {
    x := 2
    match x {
        1 | 2 | 3 => 0,
        _ => 99,
    }
}
"#;
    assert_exit_code(source, 0);
}

// ==================== Match Phi Node Type Tests ====================

#[test]
fn e2e_match_i64_in_function() {
    // Test match expression returning i64 from a separate function
    let source = r#"
fn classify(n: i64) -> i64 {
    match n {
        1 => 10,
        2 => 20,
        3 => 30,
        _ => 0,
    }
}
fn main() -> i64 {
    a := classify(2)
    I a == 20 { 0 } else { 1 }
}
"#;
    assert_exit_code(source, 0);
}

#[test]
fn e2e_match_enum_return_variant() {
    // Test match expression returning enum variant directly (phi node must use ptr, not i64)
    let source = r#"
enum Result { Ok(i64), Err(i64) }

fn transform(r: Result) -> Result {
    match r {
        Ok(v) => Ok(v * 2),
        Err(e) => Err(e + 1),
    }
}

fn unwrap_or(r: Result, default: i64) -> i64 {
    match r { Ok(v) => v, Err(_) => default }
}

fn main() -> i64 {
    r1 := transform(Ok(21))
    val := unwrap_or(r1, 0)
    I val == 42 { 0 } else { 1 }
}
"#;
    let result = compile_and_run(source).expect("should compile and run");
    assert_eq!(
        result.exit_code, 0,
        "match enum return variant failed: {}",
        result.stderr
    );
}

#[test]
fn e2e_match_enum_err_transform() {
    // Test match returning enum variant on error path
    let source = r#"
enum Result { Ok(i64), Err(i64) }

fn map_err(r: Result, offset: i64) -> Result {
    match r {
        Ok(v) => Ok(v),
        Err(e) => Err(e + offset),
    }
}

fn unwrap_err(r: Result) -> i64 {
    match r { Ok(_) => 0, Err(e) => e }
}

fn main() -> i64 {
    r := map_err(Err(10), 32)
    e := unwrap_err(r)
    I e == 42 { 0 } else { 1 }
}
"#;
    let result = compile_and_run(source).expect("should compile and run");
    assert_eq!(
        result.exit_code, 0,
        "match enum err transform failed: {}",
        result.stderr
    );
}

// ==================== Error Recovery E2E Tests ====================

/// Helper: parse with recovery and return (module, errors)
fn parse_recovery(source: &str) -> (vais_ast::Module, Vec<vais_parser::ParseError>) {
    vais_parser::parse_with_recovery(source)
}

#[test]
fn e2e_recovery_multiple_broken_functions() {
    // Three functions: good → broken → good. Recovery should find at least good1.
    let source = r#"
fn good1() -> i64 = 1
fn broken(
fn good2() -> i64 = 2
"#;
    let (module, errors) = parse_recovery(source);
    assert!(!errors.is_empty(), "Should report at least one error");
    // Should recover at least one valid item (good1 is parsed before error)
    let valid: Vec<_> = module
        .items
        .iter()
        .filter(|i| !matches!(i.node, vais_ast::Item::Error { .. }))
        .collect();
    assert!(
        valid.len() >= 1,
        "Should recover at least 1 valid item, got {}",
        valid.len()
    );
    // Total items (valid + error) should be more than just the error
    assert!(
        module.items.len() >= 2,
        "Should have at least 2 items (valid + error), got {}",
        module.items.len()
    );
}

#[test]
fn e2e_recovery_missing_closing_brace() {
    // Missing } after function body
    let source = r#"
fn broken() -> i64 {
    x := 1
fn good() -> i64 = 42
"#;
    let (module, errors) = parse_recovery(source);
    assert!(!errors.is_empty(), "Should report missing brace error");
    // good() should still be parsed
    let has_good = module
        .items
        .iter()
        .any(|i| matches!(&i.node, vais_ast::Item::Function(f) if f.name.node == "good"));
    assert!(has_good, "Should recover and parse 'good' function");
}

#[test]
fn e2e_recovery_invalid_top_level_token() {
    // Random token at top level
    let source = r#"
fn good1() -> i64 = 1
42
fn good2() -> i64 = 2
"#;
    let (module, errors) = parse_recovery(source);
    assert!(
        !errors.is_empty(),
        "Should report error for '42' at top level"
    );
    let valid: Vec<_> = module
        .items
        .iter()
        .filter(|i| !matches!(i.node, vais_ast::Item::Error { .. }))
        .collect();
    assert!(
        valid.len() >= 2,
        "Should recover both valid functions, got {}",
        valid.len()
    );
}

#[test]
fn e2e_recovery_broken_struct() {
    // Broken struct followed by valid function
    let source = r#"
struct Broken {
    x: i64,
    y
}
fn good() -> i64 = 0
"#;
    let (module, errors) = parse_recovery(source);
    assert!(!errors.is_empty(), "Should report struct field error");
    let has_good = module
        .items
        .iter()
        .any(|i| matches!(&i.node, vais_ast::Item::Function(f) if f.name.node == "good"));
    assert!(has_good, "Should recover and parse 'good' function");
}

#[test]
fn e2e_recovery_multiple_errors_collected() {
    // Multiple broken items — should collect multiple errors
    let source = r#"
fn broken1(
fn broken2(
fn broken3(
fn good() -> i64 = 0
"#;
    let (_module, errors) = parse_recovery(source);
    assert!(
        errors.len() >= 2,
        "Should collect at least 2 errors, got {}",
        errors.len()
    );
}

#[test]
fn e2e_recovery_error_preserves_span() {
    // Verify that errors contain span information
    let source = "fn broken(\nfn good() -> i64 = 0\n";
    let (_module, errors) = parse_recovery(source);
    assert!(!errors.is_empty(), "Should have errors");
    for error in &errors {
        let span = error.span();
        assert!(span.is_some(), "Each error should have a span");
    }
}

#[test]
fn e2e_recovery_broken_enum_then_valid() {
    // Broken enum followed by valid function
    let source = r#"
enum Broken {
    A(
}
fn good() -> i64 = 0
"#;
    let (module, errors) = parse_recovery(source);
    assert!(!errors.is_empty(), "Should report enum error");
    let has_good = module
        .items
        .iter()
        .any(|i| matches!(&i.node, vais_ast::Item::Function(f) if f.name.node == "good"));
    assert!(has_good, "Should recover and parse 'good' function");
}

#[test]
fn e2e_recovery_mixed_valid_and_broken() {
    // Interleaved valid and broken items
    let source = r#"
fn f1() -> i64 = 1
struct Broken1 { x }
fn f2() -> i64 = 2
struct Broken2 { y }
fn f3() -> i64 = 3
"#;
    let (module, errors) = parse_recovery(source);
    assert!(
        errors.len() >= 2,
        "Should report at least 2 errors, got {}",
        errors.len()
    );
    let valid_fns: Vec<_> = module
        .items
        .iter()
        .filter(|i| matches!(&i.node, vais_ast::Item::Function(_)))
        .collect();
    assert!(
        valid_fns.len() >= 3,
        "Should recover all 3 valid functions, got {}",
        valid_fns.len()
    );
}
